use std::path::{Component, Path, PathBuf};

use super::error::CatalogError;

pub const PHOTART_DIR: &str = ".photart";
pub const MANIFEST_FILE: &str = "catalog.json";
pub const THUMBNAILS_DIR: &str = "thumbnails";
pub const IMPORTS_DIR: &str = "imports";

pub fn photart_dir(catalog_path: &Path) -> PathBuf {
    catalog_path.join(PHOTART_DIR)
}

pub fn manifest_path(catalog_path: &Path) -> PathBuf {
    photart_dir(catalog_path).join(MANIFEST_FILE)
}

pub fn thumbnails_dir(catalog_path: &Path) -> PathBuf {
    photart_dir(catalog_path).join(THUMBNAILS_DIR)
}

pub fn imports_dir(catalog_path: &Path) -> PathBuf {
    photart_dir(catalog_path).join(IMPORTS_DIR)
}

pub fn shoot_thumbnails_dir(catalog_path: &Path, shoot_name: &str) -> PathBuf {
    thumbnails_dir(catalog_path).join(shoot_name)
}

/// Normalize a relative path to platform-neutral `/` separators.
pub fn normalize_relative_path(path: &str) -> String {
    path.replace('\\', "/")
}

/// Validate a stored relative path: non-empty, relative, no traversal, resolves inside catalog.
pub fn validate_relative_path(relative: &str) -> Result<String, CatalogError> {
    let normalized = normalize_relative_path(relative.trim());
    if normalized.is_empty() {
        return Err(CatalogError::new(
            "invalid_path",
            "Relative path cannot be empty.",
        ));
    }

    let path = Path::new(&normalized);
    if path.is_absolute() {
        return Err(CatalogError::new(
            "invalid_path",
            format!("Path must be relative: {normalized}"),
        ));
    }

    // Split manually so `.` segments are rejected (Path::components skips them).
    for segment in normalized.split('/') {
        if segment.is_empty() {
            return Err(CatalogError::new(
                "invalid_path",
                format!("Path must not contain empty segments: {normalized}"),
            ));
        }
        if segment == "." {
            return Err(CatalogError::new(
                "invalid_path",
                format!("Path must not contain \".\": {normalized}"),
            ));
        }
        if segment == ".." {
            return Err(CatalogError::new(
                "invalid_path",
                format!("Path must not contain \"..\": {normalized}"),
            ));
        }
        if segment.contains('\0') {
            return Err(CatalogError::new(
                "invalid_path",
                format!("Path must not contain null characters: {normalized}"),
            ));
        }
    }

    for component in path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir | Component::ParentDir => {
                return Err(CatalogError::new(
                    "invalid_path",
                    format!("Path must not contain \".\" or \"..\": {normalized}"),
                ));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(CatalogError::new(
                    "invalid_path",
                    format!("Path must be relative: {normalized}"),
                ));
            }
        }
    }

    Ok(normalized)
}

/// Returns true when `candidate` resolves strictly inside `catalog_root` (or is the root itself).
pub fn is_path_inside_catalog(catalog_root: &Path, candidate: &Path) -> bool {
    match (
        canonicalize_existing(catalog_root),
        canonicalize_existing(candidate),
    ) {
        (Ok(root), Ok(path)) => path_is_inside(&root, &path),
        _ => {
            // If candidate does not exist yet, compare normalized lexical paths.
            let Ok(root) = canonicalize_existing(catalog_root) else {
                return false;
            };
            let candidate_normalized = normalize_lexically(candidate);
            path_is_inside(&root, &candidate_normalized)
        }
    }
}

/// Authorize a path for asset serving: must resolve inside the Catalog.
/// Returns the canonical absolute path on success.
pub fn authorize_catalog_path(
    catalog_root: &Path,
    candidate: &Path,
) -> Result<PathBuf, CatalogError> {
    let root = canonicalize_existing(catalog_root).map_err(|error| {
        CatalogError::new(
            "catalog_path",
            format!(
                "Catalog path could not be resolved at {}: {error}",
                catalog_root.display()
            ),
        )
    })?;

    let path = canonicalize_existing(candidate).map_err(|error| {
        CatalogError::new(
            "unauthorized_path",
            format!(
                "Path could not be resolved at {}: {error}",
                candidate.display()
            ),
        )
    })?;

    if !path_is_inside(&root, &path) {
        return Err(CatalogError::new(
            "unauthorized_path",
            format!(
                "Path {} is outside the Catalog at {}.",
                path.display(),
                root.display()
            ),
        ));
    }

    Ok(path)
}

pub fn canonicalize_existing(path: &Path) -> std::io::Result<PathBuf> {
    fs_canonicalize(path)
}

fn fs_canonicalize(path: &Path) -> std::io::Result<PathBuf> {
    dunce_canonicalize(path)
}

/// Prefer a dunce-like canonicalize that strips Windows UNC prefixes when present.
/// Falls back to std::fs::canonicalize.
fn dunce_canonicalize(path: &Path) -> std::io::Result<PathBuf> {
    let canonical = std::fs::canonicalize(path)?;
    Ok(strip_unc_prefix(canonical))
}

fn strip_unc_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let text = path.to_string_lossy();
        if let Some(stripped) = text.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped);
        }
    }
    path
}

fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    strip_unc_prefix(normalized)
}

fn path_is_inside(root: &Path, candidate: &Path) -> bool {
    if candidate == root {
        return true;
    }
    candidate.starts_with(root)
}

/// Deterministic, unique thumbnail file name derived from a relative original path.
///
/// Nested sources that share a filename get distinct cache names by encoding the
/// relative path (`/` → `__`). Collisions with literal `__` in path segments are
/// avoided by hashing the full relative path into the file stem.
pub fn thumbnail_file_name(relative_path: &str) -> String {
    let normalized = normalize_relative_path(relative_path);
    let encoded = normalized.replace('/', "__");
    let stem = Path::new(&encoded)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("photo");
    let hash = simple_hash(&normalized);
    format!("{stem}_{hash:08x}.jpg")
}

fn simple_hash(value: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for byte in value.as_bytes() {
        hash ^= u32::from(*byte);
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

/// Returns true when `a` and `b` overlap (either is inside the other), after canonicalization.
pub fn paths_overlap(a: &Path, b: &Path) -> Result<bool, CatalogError> {
    let a = canonicalize_existing(a).map_err(|error| {
        CatalogError::new(
            "path_resolve",
            format!("Could not resolve path {}: {error}", a.display()),
        )
    })?;
    let b = canonicalize_existing(b).map_err(|error| {
        CatalogError::new(
            "path_resolve",
            format!("Could not resolve path {}: {error}", b.display()),
        )
    })?;
    Ok(path_is_inside(&a, &b) || path_is_inside(&b, &a))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn relative_path_validation_rejects_traversal() {
        assert!(validate_relative_path("a/b.jpg").is_ok());
        assert_eq!(
            validate_relative_path("../x").unwrap_err().code,
            "invalid_path"
        );
        assert_eq!(
            validate_relative_path("/abs").unwrap_err().code,
            "invalid_path"
        );
        assert_eq!(
            validate_relative_path("a/./b").unwrap_err().code,
            "invalid_path"
        );
    }

    #[test]
    fn thumbnail_names_are_unique_for_nested_duplicates() {
        let a = thumbnail_file_name("day1/IMG_001.jpg");
        let b = thumbnail_file_name("day2/IMG_001.jpg");
        assert_ne!(a, b);
        assert!(a.ends_with(".jpg"));
        assert!(b.ends_with(".jpg"));
    }

    #[test]
    fn authorize_only_allows_catalog_descendants() {
        let root = std::env::temp_dir().join(format!(
            "photart-paths-auth-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(root.join("inside")).unwrap();
        let outside = root
            .parent()
            .unwrap()
            .join(format!("photart-paths-outside-{}", std::process::id()));
        fs::create_dir_all(&outside).unwrap();

        let inside = root.join("inside").join("photo.jpg");
        fs::write(&inside, b"x").unwrap();
        let outside_file = outside.join("photo.jpg");
        fs::write(&outside_file, b"x").unwrap();

        assert!(authorize_catalog_path(&root, &inside).is_ok());
        assert_eq!(
            authorize_catalog_path(&root, &outside_file)
                .unwrap_err()
                .code,
            "unauthorized_path"
        );
        assert!(is_path_inside_catalog(&root, &inside));
        assert!(!is_path_inside_catalog(&root, &outside_file));

        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_dir_all(&outside);
    }
}
