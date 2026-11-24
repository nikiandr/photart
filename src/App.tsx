import { useEffect, useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";

import { PhotartMark } from "@/components/brand/PhotartMark";
import { CatalogLocationStep } from "@/components/catalog/CatalogLocationStep";
import { CatalogNameStep } from "@/components/catalog/CatalogNameStep";
import { CatalogUnavailableView } from "@/components/catalog/CatalogUnavailableView";
import { FatalConfigErrorView } from "@/components/catalog/FatalConfigErrorView";
import { HomeView } from "@/components/workspace/HomeView";
import { ImportProgressView } from "@/components/workspace/ImportProgressView";
import { ShootConfirmView } from "@/components/workspace/ShootConfirmView";
import { ShootView } from "@/components/workspace/ShootView";
import { Sidebar, type SidebarActive } from "@/components/workspace/Sidebar";
import { WorkspaceShell } from "@/components/workspace/WorkspaceShell";
import {
  createCatalog,
  importShoot,
  inspectSourceMedium,
  isConfigFatalError,
  loadCatalogState,
  loadShoot,
  purgeShoot,
  setJpegCompanionLink,
  updateShoot,
} from "@/lib/catalog";
import { editionName } from "@/lib/edition";
import { pathBasename } from "@/lib/names";
import type {
  CatalogError,
  CatalogSnapshot,
  ImportProgress,
  ShootDetails,
  ShootMedium,
  SourceMediumHint,
} from "@/types";

const DEFAULT_CATALOG_NAME = "Photart Catalog";

type AppView =
  | { kind: "booting" }
  | { kind: "setup-name"; catalogName: string }
  | {
      kind: "setup-location";
      catalogName: string;
      parentDirectory: string | null;
      picturesDirectory: string | null;
      error: string | null;
      creating: boolean;
    }
  | { kind: "unavailable"; name: string; path: string; reason: string }
  | { kind: "fatal-config"; error: CatalogError }
  | { kind: "home"; catalog: CatalogSnapshot }
  | {
      kind: "confirm-shoot";
      catalog: CatalogSnapshot;
      sourcePath: string | null;
      shootName: string;
      error: string | null;
      previous: "home" | { shootName: string };
      inspectionId: number;
      sourceHint: SourceMediumHint | "checking" | "unavailable";
      mediumOverride?: ShootMedium | null;
    }
  | {
      kind: "importing";
      catalog: CatalogSnapshot;
      sourcePath: string;
      shootName: string;
      progress: ImportProgress | null;
      importId: string;
      previous: "home" | { shootName: string };
    }
  | {
      kind: "shoot";
      catalog: CatalogSnapshot;
      shoot: ShootDetails;
      skippedNoticeCount: number | null;
      loading?: boolean;
    };

function draftMedium(draft: Extract<AppView, { kind: "confirm-shoot" }>): ShootMedium | null {
  return draft.mediumOverride !== undefined
    ? draft.mediumOverride
    : draft.sourceHint === "noritsu" ? { kind: "film", filmStock: null } : null;
}

function upsertShoot(
  catalog: CatalogSnapshot,
  shoot: ShootDetails,
): CatalogSnapshot {
  const summary = shoot.summary;
  const shoots = [
    summary,
    ...catalog.shoots.filter((entry) => entry.name !== summary.name),
  ];
  return { ...catalog, shoots };
}

function replaceShootSummary(
  catalog: CatalogSnapshot,
  previousName: string,
  shoot: ShootDetails,
): CatalogSnapshot {
  return {
    ...catalog,
    shoots: catalog.shoots.map((entry) =>
      entry.name === previousName ? shoot.summary : entry,
    ),
  };
}

function removeShootSummary(
  catalog: CatalogSnapshot,
  shootName: string,
): CatalogSnapshot {
  return {
    ...catalog,
    shoots: catalog.shoots.filter((entry) => entry.name !== shootName),
  };
}

function App() {
  const [view, setView] = useState<AppView>({ kind: "booting" });
  const picturesDirectoryRef = useRef<string | null>(null);
  const importCancelRef = useRef<(() => Promise<void>) | null>(null);
  const shootRequestIdRef = useRef(0);
  const inspectionIdRef = useRef(0);

  useEffect(() => {
    document.documentElement.setAttribute("data-atmosphere", "gallery");
  }, []);

  useEffect(() => {
    let cancelled = false;

    void (async () => {
      try {
        const state = await loadCatalogState();
        if (cancelled) {
          return;
        }

        if (state.status === "setupRequired") {
          picturesDirectoryRef.current = state.picturesDirectory;
          setView({
            kind: "setup-name",
            catalogName: DEFAULT_CATALOG_NAME,
          });
          return;
        }

        if (state.status === "unavailable") {
          picturesDirectoryRef.current = state.picturesDirectory;
          setView({
            kind: "unavailable",
            name: state.name,
            path: state.path,
            reason: state.reason,
          });
          return;
        }

        setView({ kind: "home", catalog: state.catalog });
      } catch (error) {
        if (cancelled) {
          return;
        }
        const catalogError = error as CatalogError;
        if (isConfigFatalError(catalogError)) {
          setView({ kind: "fatal-config", error: catalogError });
        } else {
          setView({
            kind: "fatal-config",
            error: {
              code: catalogError.code || "unknown",
              message:
                catalogError.message ||
                "Photart couldn’t load your Catalog.",
            },
          });
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, []);

  async function pickDirectory(): Promise<string | null> {
    const selected = await open({ directory: true, multiple: false });
    if (selected === null) {
      return null;
    }
    return typeof selected === "string" ? selected : selected[0] ?? null;
  }

  function beginCreateShoot(
    catalog: CatalogSnapshot,
    previous: "home" | { shootName: string },
  ) {
    shootRequestIdRef.current += 1;
    setView({
      kind: "confirm-shoot",
      catalog,
      sourcePath: null,
      shootName: "",
      error: null,
      previous,
      inspectionId: ++inspectionIdRef.current,
      sourceHint: "noneFound",
    });
  }

  async function chooseShootSource(draft: Extract<AppView, { kind: "confirm-shoot" }>) {
    try {
      const sourcePath = await pickDirectory();
      if (sourcePath && sourcePath !== draft.sourcePath) {
        const inspectionId = ++inspectionIdRef.current;
        setView((current) => {
          if (current.kind !== "confirm-shoot" || current.inspectionId !== draft.inspectionId) {
            return current;
          }
          const previousDefault = current.sourcePath ? pathBasename(current.sourcePath) : "";
          return {
            ...current,
            sourcePath,
            shootName: !current.shootName.trim() || current.shootName === previousDefault
              ? pathBasename(sourcePath)
              : current.shootName,
            error: null,
            inspectionId,
            sourceHint: "checking",
          };
        });
        const sourceHint = await inspectSourceMedium(sourcePath).catch(() => "unavailable" as const);
        setView((current) =>
          current.kind === "confirm-shoot" && current.inspectionId === inspectionId
            ? { ...current, sourceHint }
            : current,
        );
      }
    } catch {
      setView((current) =>
        current.kind === "confirm-shoot" && current.inspectionId === draft.inspectionId
          ? { ...current, error: "Could not open the folder picker. Try again." }
          : current,
      );
    }
  }

  async function openShoot(catalog: CatalogSnapshot, shootName: string) {
    const requestId = ++shootRequestIdRef.current;
    setView({
      kind: "shoot",
      catalog,
      shoot: {
        summary: {
          name: shootName,
          relativePath: shootName,
          importedAt: "",
          photographCount: 0,
          skippedFileCount: 0,
          medium: null,
        },
        photographs: [],
      },
      skippedNoticeCount: null,
      loading: true,
    });

    try {
      const shoot = await loadShoot(shootName);
      if (requestId !== shootRequestIdRef.current) {
        return;
      }
      setView((current) => ({
        kind: "shoot",
        catalog: current.kind === "shoot" ? replaceShootSummary(current.catalog, shootName, shoot) : catalog,
        shoot,
        skippedNoticeCount: null,
        loading: false,
      }));
    } catch {
      if (requestId !== shootRequestIdRef.current) {
        return;
      }
      setView({ kind: "home", catalog });
    }
  }

  function startImport(args: Extract<AppView, { kind: "confirm-shoot" }>) {
    if (!args.sourcePath) {
      return;
    }
    const importId =
      typeof crypto !== "undefined" && "randomUUID" in crypto
        ? crypto.randomUUID()
        : `import-${Date.now()}`;
    const shootName = args.shootName.trim();

    // Commit the importing view before invoke so progress events have a home.
    setView({
      kind: "importing",
      catalog: args.catalog,
      sourcePath: args.sourcePath,
      shootName,
      progress: {
        phase: "scanning",
        total: null,
        prepared: null,
        remainingSeconds: null,
      },
      importId,
      previous: args.previous,
    });

    const { result, cancel } = importShoot(
      {
        sourcePath: args.sourcePath,
        shootName,
        medium: draftMedium(args),
      },
      (progress) => {
        setView((current) =>
          current.kind === "importing" && current.importId === importId
            ? { ...current, progress }
            : current,
        );
      },
      importId,
    );

    importCancelRef.current = cancel;

    void result
      .then((imported) => {
        importCancelRef.current = null;
        const catalog = upsertShoot(args.catalog, imported.shoot);
        setView({
          kind: "shoot",
          catalog,
          shoot: imported.shoot,
          skippedNoticeCount:
            imported.skippedFileCount > 0 ? imported.skippedFileCount : null,
          loading: false,
        });
      })
      .catch((error: unknown) => {
        importCancelRef.current = null;
        const catalogError = error as CatalogError;
        const cancelled =
          catalogError.code === "import_cancelled" ||
          /cancel/i.test(catalogError.message);

        if (cancelled) {
          setView({ kind: "home", catalog: args.catalog });
          return;
        }

        setView({
          ...args,
          sourceHint: args.sourceHint === "checking" ? "noneFound" : args.sourceHint,
          error: catalogError.message,
        });
      });
  }

  async function handleUpdateShoot(shootName: string, newName: string, medium: ShootMedium | null) {
    const renamed = await updateShoot(shootName, newName, medium);
    shootRequestIdRef.current += 1;
    applyShootUpdate(shootName, renamed);
  }

  function applyShootUpdate(shootName: string, updated: ShootDetails) {
    setView((current) => {
      if (current.kind === "home") {
        return {
          ...current,
          catalog: replaceShootSummary(current.catalog, shootName, updated),
        };
      }
      if (current.kind === "confirm-shoot") {
        return {
          ...current,
          catalog: replaceShootSummary(current.catalog, shootName, updated),
          previous:
            current.previous !== "home" &&
            current.previous.shootName === shootName
              ? { shootName: updated.summary.name }
              : current.previous,
        };
      }
      if (current.kind === "importing") {
        return {
          ...current,
          catalog: replaceShootSummary(current.catalog, shootName, updated),
          previous:
            current.previous !== "home" &&
            current.previous.shootName === shootName
              ? { shootName: updated.summary.name }
              : current.previous,
        };
      }
      if (current.kind === "shoot") {
        return {
          ...current,
          catalog: replaceShootSummary(current.catalog, shootName, updated),
          shoot:
            current.shoot.summary.name === shootName
              ? updated
              : current.shoot,
          loading:
            current.shoot.summary.name === shootName
              ? false
              : current.loading,
        };
      }
      return current;
    });
  }

  async function handlePurgeShoot(shootName: string) {
    await purgeShoot(shootName);
    shootRequestIdRef.current += 1;

    setView((current) => {
      if (current.kind === "home") {
        return {
          ...current,
          catalog: removeShootSummary(current.catalog, shootName),
        };
      }
      if (current.kind === "confirm-shoot") {
        return {
          ...current,
          catalog: removeShootSummary(current.catalog, shootName),
          previous:
            current.previous !== "home" &&
            current.previous.shootName === shootName
              ? "home"
              : current.previous,
        };
      }
      if (current.kind === "importing") {
        return {
          ...current,
          catalog: removeShootSummary(current.catalog, shootName),
          previous:
            current.previous !== "home" &&
            current.previous.shootName === shootName
              ? "home"
              : current.previous,
        };
      }
      if (current.kind === "shoot") {
        const catalog = removeShootSummary(current.catalog, shootName);
        if (current.shoot.summary.name === shootName) {
          return { kind: "home", catalog };
        }
        return { ...current, catalog };
      }
      return current;
    });
  }

  function sidebarActive(current: AppView): SidebarActive | null {
    if (current.kind === "home") {
      return "home";
    }
    if (current.kind === "shoot") {
      return { shootName: current.shoot.summary.name };
    }
    if (current.kind === "confirm-shoot") {
      return current.previous === "home"
        ? "home"
        : { shootName: current.previous.shootName };
    }
    return null;
  }

  function catalogFromView(current: AppView): CatalogSnapshot | null {
    if (
      current.kind === "home" ||
      current.kind === "confirm-shoot" ||
      current.kind === "importing" ||
      current.kind === "shoot"
    ) {
      return current.catalog;
    }
    return null;
  }

  if (view.kind === "booting") {
    return (
      <div className="flex h-svh w-full items-center justify-center overflow-hidden bg-background">
        <div role="status" aria-live="polite" className="flex flex-col items-center gap-4">
          <PhotartMark className="size-12" title={editionName()} />
          <div className="flex flex-col items-center gap-3">
            <SpinnerGap
              className="size-5 animate-spin motion-reduce:animate-none text-muted-foreground"
              aria-hidden="true"
            />
            <p className="font-sans text-sm text-muted-foreground">
              Opening {editionName()}…
            </p>
          </div>
        </div>
      </div>
    );
  }

  if (view.kind === "setup-name") {
    return (
      <div className="h-svh w-full overflow-hidden bg-background">
        <CatalogNameStep
          catalogName={view.catalogName}
          onCatalogNameChange={(catalogName) =>
            setView({ kind: "setup-name", catalogName })
          }
          onContinue={() =>
            setView({
              kind: "setup-location",
              catalogName: view.catalogName.trim(),
              parentDirectory: picturesDirectoryRef.current,
              picturesDirectory: picturesDirectoryRef.current,
              error: null,
              creating: false,
            })
          }
        />
      </div>
    );
  }

  if (view.kind === "setup-location") {
    return (
      <div className="h-svh w-full overflow-hidden bg-background">
        <CatalogLocationStep
          catalogName={view.catalogName}
          parentDirectory={view.parentDirectory}
          picturesDirectory={view.picturesDirectory}
          error={view.error}
          creating={view.creating}
          onChooseDirectory={() => {
            void (async () => {
              const parentDirectory = await pickDirectory();
              if (!parentDirectory) {
                return;
              }
              setView((current) =>
                current.kind === "setup-location"
                  ? { ...current, parentDirectory, error: null }
                  : current,
              );
            })();
          }}
          onBack={() =>
            setView({
              kind: "setup-name",
              catalogName: view.catalogName,
            })
          }
          onCreate={() => {
            if (!view.parentDirectory || view.creating) {
              return;
            }
            const parentDirectory = view.parentDirectory;
            const catalogName = view.catalogName;
            setView({ ...view, creating: true, error: null });
            void createCatalog({ name: catalogName, parentDirectory })
              .then((catalog) => {
                setView({ kind: "home", catalog });
              })
              .catch((error: unknown) => {
                const catalogError = error as CatalogError;
                setView((current) =>
                  current.kind === "setup-location"
                    ? {
                        ...current,
                        creating: false,
                        error: catalogError.message,
                      }
                    : current,
                );
              });
          }}
        />
      </div>
    );
  }

  if (view.kind === "unavailable") {
    return (
      <div className="h-svh w-full overflow-hidden bg-background">
        <CatalogUnavailableView
          name={view.name}
          path={view.path}
          reason={view.reason}
          onSetupNew={() =>
            setView({
              kind: "setup-name",
              catalogName: DEFAULT_CATALOG_NAME,
            })
          }
        />
      </div>
    );
  }

  if (view.kind === "fatal-config") {
    return (
      <div className="h-svh w-full overflow-hidden bg-background">
        <FatalConfigErrorView error={view.error} />
      </div>
    );
  }

  const catalog = catalogFromView(view);
  if (!catalog) {
    return null;
  }

  const importing = view.kind === "importing";
  const active = sidebarActive(view);

  return (
    <WorkspaceShell
      sidebar={
        <Sidebar
          catalogName={catalog.name}
          shoots={catalog.shoots}
          active={active}
          disabled={importing}
          onHome={() => {
            if (importing) {
              return;
            }
            shootRequestIdRef.current += 1;
            setView({ kind: "home", catalog });
          }}
          onCreateShoot={() => {
            if (importing) {
              return;
            }
            const previous: "home" | { shootName: string } =
              view.kind === "shoot"
                ? { shootName: view.shoot.summary.name }
                : "home";
            beginCreateShoot(catalog, previous);
          }}
          onSelectShoot={(shootName) => {
            if (importing) {
              return;
            }
            void openShoot(catalog, shootName);
          }}
          onUpdateShoot={handleUpdateShoot}
          onPurgeShoot={handlePurgeShoot}
        />
      }
    >
      {view.kind === "home" ? (
        <HomeView
          onCreateShoot={() => beginCreateShoot(view.catalog, "home")}
        />
      ) : null}

      {view.kind === "confirm-shoot" ? (
        <ShootConfirmView
          shootName={view.shootName}
          sourcePath={view.sourcePath}
          onChooseSource={() => void chooseShootSource(view)}
          error={view.error}
          medium={draftMedium(view)}
          sourceHint={view.mediumOverride === undefined ? view.sourceHint : "noneFound"}
          onMediumChange={(mediumOverride) =>
            setView((current) => current.kind === "confirm-shoot" && current.inspectionId === view.inspectionId
              ? { ...current, mediumOverride }
              : current)
          }
          onShootNameChange={(shootName) =>
            setView((current) => current.kind === "confirm-shoot" && current.inspectionId === view.inspectionId
              ? { ...current, shootName, error: null }
              : current)
          }
          onCancel={() => {
            if (view.previous === "home") {
              setView({ kind: "home", catalog: view.catalog });
              return;
            }
            void openShoot(view.catalog, view.previous.shootName);
          }}
          onImport={() => startImport(view)}
        />
      ) : null}

      {view.kind === "importing" ? (
        <ImportProgressView
          shootName={view.shootName}
          progress={view.progress}
          onCancel={() => {
            void importCancelRef.current?.();
          }}
        />
      ) : null}

      {view.kind === "shoot" ? (
        <ShootView
          key={view.shoot.summary.relativePath}
          shoot={view.shoot}
          skippedNoticeCount={view.skippedNoticeCount}
          loading={view.loading}
          onSetPairLinked={async (rawPath, linked) => {
            const shootName = view.shoot.summary.name;
            const updated = await setJpegCompanionLink(shootName, rawPath, linked);
            applyShootUpdate(shootName, updated);
          }}
        />
      ) : null}
    </WorkspaceShell>
  );
}

export default App;
