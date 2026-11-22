import { Channel, convertFileSrc, invoke } from "@tauri-apps/api/core";

import type {
  CatalogError,
  CatalogSnapshot,
  CatalogStateResponse,
  CreateCatalogInput,
  ImportProgress,
  ImportShootInput,
  ImportShootResult,
  PhotographCapture,
  PhotographEntry,
  ShootDetails,
  ShootMedium,
  SourceMediumHint,
} from "@/types";

function normalizeError(error: unknown): CatalogError {
  if (
    error &&
    typeof error === "object" &&
    "message" in error &&
    typeof (error as CatalogError).message === "string"
  ) {
    const catalogError = error as CatalogError;
    return {
      code: catalogError.code || "unknown",
      message: catalogError.message,
    };
  }

  if (error instanceof Error) {
    return { code: "unknown", message: error.message };
  }

  return {
    code: "unknown",
    message: "Something went wrong while talking to Photart.",
  };
}

type RawShootDetails = Omit<ShootDetails, "photographs"> & {
  photographs: (Omit<PhotographEntry, "thumbnailUrl" | "originalUrl"> & {
    originalAbsolutePath: string | null;
  })[];
};

function withAssetUrls(shoot: RawShootDetails): ShootDetails {
  return {
    ...shoot,
    photographs: shoot.photographs.map(({ originalAbsolutePath, ...photograph }) => ({
      ...photograph,
      thumbnailUrl: convertFileSrc(photograph.thumbnailAbsolutePath),
      originalUrl: originalAbsolutePath === null ? null : convertFileSrc(originalAbsolutePath),
    })),
  };
}

export async function loadCatalogState(): Promise<CatalogStateResponse> {
  try {
    return await invoke<CatalogStateResponse>("get_catalog_state");
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function createCatalog(
  input: CreateCatalogInput,
): Promise<CatalogSnapshot> {
  try {
    return await invoke<CatalogSnapshot>("create_catalog", {
      name: input.name,
      parentDirectory: input.parentDirectory,
    });
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function loadShoot(name: string): Promise<ShootDetails> {
  try {
    const shoot = await invoke<RawShootDetails>("get_shoot", { shootName: name });
    return withAssetUrls(shoot);
  } catch (error) {
    throw normalizeError(error);
  }
}

export function loadPhotographCapture(
  shootName: string,
  relativePath: string,
): Promise<PhotographCapture> {
  return invoke("get_photograph_capture", { shootName, relativePath });
}

export function inspectSourceMedium(sourcePath: string): Promise<SourceMediumHint> {
  return invoke("inspect_source_medium", { sourcePath });
}

export async function updateShoot(
  shootName: string,
  newName: string,
  medium: ShootMedium | null,
): Promise<ShootDetails> {
  try {
    const shoot = await invoke<RawShootDetails>("update_shoot", {
      shootName,
      newName,
      medium,
    });
    return withAssetUrls(shoot);
  } catch (error) {
    throw normalizeError(error);
  }
}

export async function purgeShoot(shootName: string): Promise<void> {
  try {
    await invoke("purge_shoot", { shootName });
  } catch (error) {
    throw normalizeError(error);
  }
}

export function importShoot(
  input: ImportShootInput,
  onProgress: (progress: ImportProgress) => void,
  existingImportId?: string,
): {
  result: Promise<ImportShootResult>;
  cancel: () => Promise<void>;
  importId: string;
} {
  const importId =
    existingImportId ??
    (typeof crypto !== "undefined" && "randomUUID" in crypto
      ? crypto.randomUUID()
      : `import-${Date.now()}`);

  const channel = new Channel<ImportProgress>();
  channel.onmessage = onProgress;

  const result = (async () => {
    try {
      const imported = await invoke<Omit<ImportShootResult, "shoot"> & { shoot: RawShootDetails }>("import_shoot", {
        importId,
        sourcePath: input.sourcePath,
        shootName: input.shootName,
        medium: input.medium,
        onProgress: channel,
      });
      return {
        ...imported,
        shoot: withAssetUrls(imported.shoot),
      };
    } catch (error) {
      throw normalizeError(error);
    }
  })();

  return {
    importId,
    result,
    cancel: async () => {
      try {
        await invoke("cancel_shoot_import", { importId });
      } catch (error) {
        throw normalizeError(error);
      }
    },
  };
}

export function isConfigFatalError(error: CatalogError): boolean {
  return error.code.startsWith("config_");
}

export function joinCatalogPath(
  parentDirectory: string,
  catalogName: string,
): string {
  return parentDirectory.endsWith("/")
    ? `${parentDirectory}${catalogName}`
    : `${parentDirectory}/${catalogName}`;
}

/** Prefer a concise ~/Pictures or ~/… preview when the path is under home. */
export function homePathDisplay(
  parentDirectory: string,
  catalogName: string,
  picturesDirectory?: string | null,
): string {
  const joined = joinCatalogPath(parentDirectory, catalogName);

  if (picturesDirectory) {
    const pics = picturesDirectory.replace(/\/+$/, "");
    if (joined === `${pics}/${catalogName}` || joined.startsWith(`${pics}/`)) {
      return `~/Pictures${joined.slice(pics.length)}`;
    }
  }

  const usersMatch = joined.match(/^\/Users\/[^/]+(\/.*)$/);
  if (usersMatch) {
    return `~${usersMatch[1]}`;
  }

  return joined;
}

export function shortenAbsolutePath(
  absolutePath: string,
  picturesDirectory?: string | null,
): string {
  if (picturesDirectory) {
    const pics = picturesDirectory.replace(/\/+$/, "");
    if (absolutePath === pics || absolutePath.startsWith(`${pics}/`)) {
      return `~/Pictures${absolutePath.slice(pics.length)}`;
    }
  }

  const usersMatch = absolutePath.match(/^\/Users\/[^/]+(\/.*)$/);
  if (usersMatch) {
    return `~${usersMatch[1]}`;
  }

  return absolutePath;
}
