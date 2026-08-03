use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use chrono::{SecondsFormat, Utc};
use rayon::prelude::*;

use crate::image_processing::{self, prepare_photograph};

use super::error::CatalogError;
use super::manifest::{self, PhotographRecord, ShootRecord};
use super::names::validate_single_path_component_name;
use super::paths::{self, normalize_relative_path, thumbnail_file_name};
use super::state::shoot_details_from_record;
use super::{ImportProgress, ImportShootResult};

static IMPORT_SEQUENCE: AtomicU64 = AtomicU64::new(1);

pub struct ImportCoordinator {
    active: Mutex<Option<ActiveImport>>,
}

struct ActiveImport {
    id: String,
    cancel: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct ImportPermit {
    pub id: String,
    pub cancel: Arc<AtomicBool>,
}

impl ImportCoordinator {
    pub fn new() -> Self {
        Self {
            active: Mutex::new(None),
        }
    }

    /// Begin a single active import with the opaque frontend import ID.
    /// Returns an error if one is already running.
    pub fn begin(&self, id: String) -> Result<ImportPermit, CatalogError> {
        if id.trim().is_empty() {
            return Err(CatalogError::new(
                "import_id_invalid",
                "An import id is required.",
            ));
        }

        let mut guard = self
            .active
            .lock()
            .map_err(|_| CatalogError::new("import_lock", "Import coordinator is unavailable."))?;
        if guard.is_some() {
            return Err(CatalogError::new(
                "import_busy",
                "Another Shoot import is already in progress.",
            ));
        }

        let cancel = Arc::new(AtomicBool::new(false));
        *guard = Some(ActiveImport {
            id: id.clone(),
            cancel: Arc::clone(&cancel),
        });
        Ok(ImportPermit { id, cancel })
    }

    /// Request cancellation of the active import matching `id`.
    /// Safe to call more than once, including after the import finishes.
    pub fn cancel(&self, id: &str) -> Result<(), CatalogError> {
        let guard = self
            .active
            .lock()
            .map_err(|_| CatalogError::new("import_lock", "Import coordinator is unavailable."))?;
        if let Some(active) = guard.as_ref() {
            if active.id == id {
                active.cancel.store(true, Ordering::SeqCst);
            }
        }
        Ok(())
    }

    /// Clear the active import slot after completion or failure.
    pub fn finish(&self, id: &str) -> Result<(), CatalogError> {
        let mut guard = self
            .active
            .lock()
            .map_err(|_| CatalogError::new("import_lock", "Import coordinator is unavailable."))?;
        match guard.as_ref() {
            Some(active) if active.id == id => {
                *guard = None;
                Ok(())
            }
            Some(_) => Err(CatalogError::new(
                "import_not_found",
                "No active import matches the given id.",
            )),
            None => Ok(()),
        }
    }
}

impl Default for ImportCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

fn next_import_id() -> String {
    let sequence = IMPORT_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let millis = Utc::now().timestamp_millis();
    format!("import-{millis}-{sequence}")
}

#[derive(Default)]
pub(crate) struct ImportFaults {
    /// Fail after N photographs begin prepare (0 = disabled).
    pub fail_after_prepared: Option<u32>,
    /// Fail after N photographs begin the staged original write (0 = disabled).
    pub fail_after_copied: Option<u32>,
    /// Fail during commit after files are moved but before manifest write.
    pub fail_before_manifest: bool,
}

pub fn import_shoot(
    catalog_path: &Path,
    source_path: &Path,
    shoot_name: &str,
    mut progress: impl FnMut(ImportProgress),
    cancel: &AtomicBool,
) -> Result<ImportShootResult, CatalogError> {
    import_shoot_inner(
        catalog_path,
        source_path,
        shoot_name,
        &mut progress,
        cancel,
        &ImportFaults::default(),
    )
}

pub(crate) fn import_shoot_inner(
    catalog_path: &Path,
    source_path: &Path,
    shoot_name: &str,
    progress: &mut dyn FnMut(ImportProgress),
    cancel: &AtomicBool,
    faults: &ImportFaults,
) -> Result<ImportShootResult, CatalogError> {
    check_cancel(cancel)?;

    let shoot_name = validate_single_path_component_name(shoot_name)?;
    let catalog_path = paths::canonicalize_existing(catalog_path).map_err(|error| {
        CatalogError::new(
            "catalog_path",
            format!(
                "Could not resolve Catalog path {}: {error}",
                catalog_path.display()
            ),
        )
    })?;
    let source_path = paths::canonicalize_existing(source_path).map_err(|error| {
        CatalogError::new(
            "source_path",
            format!(
                "Could not resolve Source path {}: {error}",
                source_path.display()
            ),
        )
    })?;

    if !source_path.is_dir() {
        return Err(CatalogError::new(
            "source_not_directory",
            format!("Source is not a directory: {}.", source_path.display()),
        ));
    }

    // Ensure Source is readable.
    fs::read_dir(&source_path).map_err(|error| {
        CatalogError::new(
            "source_unreadable",
            format!(
                "Source directory could not be read at {}: {error}",
                source_path.display()
            ),
        )
    })?;

    if paths::paths_overlap(&source_path, &catalog_path)? {
        return Err(CatalogError::new(
            "source_catalog_overlap",
            "Source and Catalog paths overlap; import is not allowed.",
        ));
    }

    let mut manifest = manifest::read_manifest(&catalog_path)?;
    if manifest.shoots.iter().any(|shoot| shoot.name == shoot_name) {
        return Err(CatalogError::new(
            "shoot_exists",
            format!("A Shoot named \"{shoot_name}\" already exists in the Catalog."),
        ));
    }

    let destination_shoot = catalog_path.join(&shoot_name);
    if destination_shoot.exists() {
        return Err(CatalogError::new(
            "shoot_folder_exists",
            format!(
                "A folder already exists at the Shoot destination: {}.",
                destination_shoot.display()
            ),
        ));
    }

    let import_id = next_import_id();
    let staging_root = paths::imports_dir(&catalog_path).join(&import_id);
    let staging_shoot = staging_root.join("shoot");
    let staging_thumbnails = staging_root.join("thumbnails");

    let mut rollback = RollbackState {
        staging_root: Some(staging_root.clone()),
        visible_shoot: None,
        visible_thumbnails: None,
    };

    let result = (|| {
        fs::create_dir_all(&staging_shoot).map_err(|error| {
            CatalogError::new(
                "import_stage",
                format!(
                    "Could not create import staging at {}: {error}",
                    staging_shoot.display()
                ),
            )
        })?;
        fs::create_dir_all(&staging_thumbnails).map_err(|error| {
            CatalogError::new(
                "import_stage",
                format!(
                    "Could not create thumbnail staging at {}: {error}",
                    staging_thumbnails.display()
                ),
            )
        })?;

        progress(ImportProgress {
            phase: "scanning".into(),
            total: None,
            prepared: None,
        });

        let scan = scan_source(&source_path, cancel)?;
        check_cancel(cancel)?;

        progress(ImportProgress {
            phase: "preparing".into(),
            total: Some(scan.candidates.len() as u32),
            prepared: Some(0),
        });

        // Single-pass ingest: read each source file once, build the thumbnail,
        // and write the original bytes to staging (no second source read).
        // Because copying is no longer a separate pass, "preparing" is the only
        // per-photograph phase the UI reports.
        let prepared = ingest_candidates(
            &IngestContext {
                source_path: &source_path,
                staging_shoot: &staging_shoot,
                staging_thumbnails: &staging_thumbnails,
                shoot_name: &shoot_name,
            },
            &scan.candidates,
            progress,
            cancel,
            faults,
        )?;

        if prepared.is_empty() {
            return Err(CatalogError::new(
                "no_photographs",
                "No supported photographs were found in the Source.",
            ));
        }

        let total = prepared.len() as u32;
        progress(ImportProgress {
            phase: "finalizing".into(),
            total: Some(total),
            prepared: Some(total),
        });

        // Commit: move staged originals and thumbnails into place, then update manifest.
        move_dir(&staging_shoot, &destination_shoot).map_err(|error| {
            CatalogError::new(
                "import_commit",
                format!(
                    "Could not commit Shoot folder to {}: {error}",
                    destination_shoot.display()
                ),
            )
        })?;
        rollback.visible_shoot = Some(destination_shoot.clone());
        rollback.staging_root = Some(staging_root.clone()); // still has thumbnails staging

        let final_thumbnails = paths::shoot_thumbnails_dir(&catalog_path, &shoot_name);
        if let Some(parent) = final_thumbnails.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                CatalogError::new(
                    "import_commit",
                    format!(
                        "Could not create thumbnails directory at {}: {error}",
                        parent.display()
                    ),
                )
            })?;
        }
        move_dir(&staging_thumbnails, &final_thumbnails).map_err(|error| {
            CatalogError::new(
                "import_commit",
                format!(
                    "Could not commit thumbnails to {}: {error}",
                    final_thumbnails.display()
                ),
            )
        })?;
        rollback.visible_thumbnails = Some(final_thumbnails);

        // Staging root should now be empty or nearly empty; remove it.
        let _ = fs::remove_dir_all(&staging_root);
        rollback.staging_root = None;

        if faults.fail_before_manifest {
            return Err(CatalogError::new(
                "import_fault",
                "Injected failure before manifest update.",
            ));
        }

        let mut photographs: Vec<PhotographRecord> = prepared
            .iter()
            .map(|item| PhotographRecord {
                relative_path: item.relative_path.clone(),
                file_name: item.file_name.clone(),
                width: item.width,
                height: item.height,
                thumbnail_relative_path: item.thumbnail_relative_path.clone(),
            })
            .collect();
        photographs.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

        let shoot_record = ShootRecord {
            name: shoot_name.clone(),
            relative_path: shoot_name.clone(),
            imported_at: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            photograph_count: photographs.len() as u32,
            skipped_file_count: scan.skipped_file_count + prepared_skip_count(&scan, &prepared),
            photographs,
        };

        // Prepend so manifest order is newest-first.
        manifest.shoots.insert(0, shoot_record.clone());
        manifest::write_manifest(&catalog_path, &manifest)?;

        // Success: clear rollback targets.
        rollback.visible_shoot = None;
        rollback.visible_thumbnails = None;

        Ok(ImportShootResult {
            shoot: shoot_details_from_record(&catalog_path, &shoot_record)?,
            skipped_file_count: shoot_record.skipped_file_count,
        })
    })();

    match result {
        Ok(value) => Ok(value),
        Err(error) => {
            let cleanup_error = rollback.cleanup();
            if let Some(cleanup_error) = cleanup_error {
                return Err(CatalogError::new(
                    "import_rollback",
                    format!(
                        "{}; additionally, cleanup failed and left: {}",
                        error.message, cleanup_error
                    ),
                ));
            }
            Err(error)
        }
    }
}

fn prepared_skip_count(scan: &ScanResult, prepared: &[PreparedItem]) -> u32 {
    let prepared_paths: std::collections::HashSet<&str> = prepared
        .iter()
        .map(|item| item.relative_path.as_str())
        .collect();
    scan.candidates
        .iter()
        .filter(|path| !prepared_paths.contains(path.as_str()))
        .count() as u32
}

struct RollbackState {
    staging_root: Option<PathBuf>,
    visible_shoot: Option<PathBuf>,
    visible_thumbnails: Option<PathBuf>,
}

impl RollbackState {
    fn cleanup(&self) -> Option<String> {
        let mut remaining = Vec::new();

        if let Some(path) = &self.staging_root {
            if path.exists() {
                if let Err(error) = fs::remove_dir_all(path) {
                    remaining.push(format!("{} ({error})", path.display()));
                }
            }
        }
        if let Some(path) = &self.visible_shoot {
            if path.exists() {
                if let Err(error) = fs::remove_dir_all(path) {
                    remaining.push(format!("{} ({error})", path.display()));
                }
            }
        }
        if let Some(path) = &self.visible_thumbnails {
            if path.exists() {
                if let Err(error) = fs::remove_dir_all(path) {
                    remaining.push(format!("{} ({error})", path.display()));
                }
            }
        }

        if remaining.is_empty() {
            None
        } else {
            Some(remaining.join(", "))
        }
    }
}

struct ScanResult {
    candidates: Vec<String>,
    skipped_file_count: u32,
}

fn scan_source(source_path: &Path, cancel: &AtomicBool) -> Result<ScanResult, CatalogError> {
    let mut candidates = Vec::new();
    let mut skipped_file_count = 0u32;
    walk_source(
        source_path,
        source_path,
        &mut candidates,
        &mut skipped_file_count,
        cancel,
    )?;
    candidates.sort();
    Ok(ScanResult {
        candidates,
        skipped_file_count,
    })
}

fn walk_source(
    source_root: &Path,
    current: &Path,
    candidates: &mut Vec<String>,
    skipped_file_count: &mut u32,
    cancel: &AtomicBool,
) -> Result<(), CatalogError> {
    check_cancel(cancel)?;

    let entries = fs::read_dir(current).map_err(|error| {
        CatalogError::new(
            "source_scan",
            format!("Could not read directory {}: {error}", current.display()),
        )
    })?;

    let mut entries: Vec<_> = entries.collect::<Result<Vec<_>, _>>().map_err(|error| {
        CatalogError::new(
            "source_scan",
            format!("Could not read directory {}: {error}", current.display()),
        )
    })?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        check_cancel(cancel)?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            CatalogError::new(
                "source_scan",
                format!("Could not read metadata for {}: {error}", path.display()),
            )
        })?;
        let file_type = metadata.file_type();

        if file_type.is_symlink() {
            // Do not follow directory symlinks; skip file symlinks (not regular files).
            *skipped_file_count += 1;
            continue;
        }

        if file_type.is_dir() {
            walk_source(source_root, &path, candidates, skipped_file_count, cancel)?;
            continue;
        }

        if !file_type.is_file() {
            *skipped_file_count += 1;
            continue;
        }

        let relative = path.strip_prefix(source_root).map_err(|_| {
            CatalogError::new(
                "source_scan",
                format!("Path {} is outside the Source root.", path.display()),
            )
        })?;
        let relative = normalize_relative_path(&relative.to_string_lossy());

        if image_processing::is_supported_extension(&path) {
            candidates.push(relative);
        } else {
            *skipped_file_count += 1;
        }
    }

    Ok(())
}

struct PreparedItem {
    relative_path: String,
    file_name: String,
    width: u32,
    height: u32,
    thumbnail_relative_path: String,
}

enum IngestWorkerResult {
    Ready(PreparedItem),
    Skipped,
    Failed(CatalogError),
}

/// The fixed locations an ingest works against, bundled so the worker closure
/// (and `ingest_candidates` itself) stays to a readable argument list.
struct IngestContext<'a> {
    source_path: &'a Path,
    staging_shoot: &'a Path,
    staging_thumbnails: &'a Path,
    shoot_name: &'a str,
}

fn ingest_chunk_size() -> usize {
    std::thread::available_parallelism()
        .map(|parallelism| parallelism.get().saturating_mul(2))
        .unwrap_or(8)
        .clamp(4, 32)
}

/// Read each candidate once, generate its thumbnail, and write the original
/// bytes to staging.
///
/// Work is parallelized in bounded chunks so progress reporting and cancellation
/// checkpoints stay on the main thread between chunks. Chunking does not bound
/// memory: each worker drops its `bytes` before returning, so peak usage is
/// roughly `rayon threads × largest file`, independent of `chunk_size`.
///
/// Note this deliberately writes the bytes already in memory rather than calling
/// `fs::copy`. That trades away `fs::copy`'s macOS `fclonefileat` fast path and
/// its permission-bit preservation in exchange for never reading the source
/// twice — the right side of the trade when importing from a slow SD card, which
/// is the common case, and where the source is a different volume anyway.
fn ingest_candidates(
    ctx: &IngestContext,
    candidates: &[String],
    progress: &mut dyn FnMut(ImportProgress),
    cancel: &AtomicBool,
    faults: &ImportFaults,
) -> Result<Vec<PreparedItem>, CatalogError> {
    let total = candidates.len() as u32;
    let chunk_size = ingest_chunk_size();
    let prepared_slots = AtomicU32::new(0);
    let copied_slots = AtomicU32::new(0);
    let mut prepared = Vec::with_capacity(candidates.len());

    for chunk in candidates.chunks(chunk_size) {
        check_cancel(cancel)?;

        let results: Vec<IngestWorkerResult> = chunk
            .par_iter()
            .map(|relative| {
                if cancel.load(Ordering::SeqCst) {
                    return IngestWorkerResult::Failed(CatalogError::new(
                        "import_cancelled",
                        "Shoot import was cancelled.",
                    ));
                }

                if let Some(limit) = faults.fail_after_prepared {
                    let slot = prepared_slots.fetch_add(1, Ordering::SeqCst);
                    if slot >= limit {
                        return IngestWorkerResult::Failed(CatalogError::new(
                            "import_fault",
                            "Injected failure during prepare.",
                        ));
                    }
                }

                let source_absolute = ctx.source_path.join(Path::new(relative));
                let bytes = match fs::read(&source_absolute) {
                    Ok(bytes) => bytes,
                    Err(_) => return IngestWorkerResult::Skipped,
                };

                let photograph = match prepare_photograph(&bytes) {
                    Ok(photograph) => photograph,
                    Err(_) => return IngestWorkerResult::Skipped,
                };

                let thumb_name = thumbnail_file_name(relative);
                let thumb_staging_path = ctx.staging_thumbnails.join(&thumb_name);
                if let Err(error) = fs::write(&thumb_staging_path, &photograph.thumbnail_jpeg) {
                    return IngestWorkerResult::Failed(CatalogError::new(
                        "thumbnail_write",
                        format!(
                            "Could not write staged thumbnail at {}: {error}",
                            thumb_staging_path.display()
                        ),
                    ));
                }

                if let Some(limit) = faults.fail_after_copied {
                    let slot = copied_slots.fetch_add(1, Ordering::SeqCst);
                    if slot >= limit {
                        return IngestWorkerResult::Failed(CatalogError::new(
                            "import_fault",
                            "Injected failure during copy.",
                        ));
                    }
                }

                let destination = ctx.staging_shoot.join(Path::new(relative));
                // Created lazily, per photograph that actually survived prepare, so
                // a source folder whose files all skipped never reaches the Catalog
                // as an empty directory. `create_dir_all` tolerates the races
                // between workers sharing a parent.
                if let Some(parent) = destination.parent() {
                    if let Err(error) = fs::create_dir_all(parent) {
                        return IngestWorkerResult::Failed(CatalogError::new(
                            "import_copy",
                            format!(
                                "Could not create destination folder at {}: {error}",
                                parent.display()
                            ),
                        ));
                    }
                }

                // Exact original-byte write from the in-memory read (no second source I/O).
                if let Err(error) = fs::write(&destination, &bytes) {
                    return IngestWorkerResult::Failed(CatalogError::new(
                        "import_copy",
                        format!(
                            "Could not write {} to {}: {error}",
                            source_absolute.display(),
                            destination.display()
                        ),
                    ));
                }

                let file_name = Path::new(relative)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(relative)
                    .to_string();

                let thumbnail_relative_path = format!(
                    "{}/{}/{}/{}",
                    paths::PHOTART_DIR,
                    paths::THUMBNAILS_DIR,
                    ctx.shoot_name,
                    thumb_name
                );

                IngestWorkerResult::Ready(PreparedItem {
                    relative_path: relative.clone(),
                    file_name,
                    width: photograph.width,
                    height: photograph.height,
                    thumbnail_relative_path,
                })
            })
            .collect();

        for result in results {
            match result {
                IngestWorkerResult::Ready(item) => {
                    prepared.push(item);
                    progress(ImportProgress {
                        phase: "preparing".into(),
                        total: Some(total),
                        prepared: Some(prepared.len() as u32),
                    });
                }
                IngestWorkerResult::Skipped => {}
                IngestWorkerResult::Failed(error) => return Err(error),
            }
        }
    }

    prepared.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(prepared)
}

fn move_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(_) => {
            // Cross-device fallback: recursive copy then remove.
            copy_dir_recursive(from, to)?;
            fs::remove_dir_all(from)?;
            Ok(())
        }
    }
}

fn copy_dir_recursive(from: &Path, to: &Path) -> std::io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let source = entry.path();
        let destination = to.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_recursive(&source, &destination)?;
        } else {
            fs::copy(&source, &destination)?;
        }
    }
    Ok(())
}

fn check_cancel(cancel: &AtomicBool) -> Result<(), CatalogError> {
    if cancel.load(Ordering::SeqCst) {
        Err(CatalogError::new(
            "import_cancelled",
            "Shoot import was cancelled.",
        ))
    } else {
        Ok(())
    }
}
