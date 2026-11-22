export type CatalogError = {
  code: string;
  message: string;
};

export type ShootSummary = {
  name: string;
  relativePath: string;
  importedAt: string;
  photographCount: number;
  skippedFileCount: number;
};

export type PhotographEntry = {
  relativePath: string;
  fileName: string;
  width: number;
  height: number;
  thumbnailRelativePath: string;
  thumbnailAbsolutePath: string;
  /** Authorized asset URL for the cached thumbnail. */
  thumbnailUrl: string;
  originalUrl: string | null;
};

export type PhotographCapture = {
  camera: string | null;
  lens: string | null;
  capturedAt: string | null;
  aperture: string | null;
  shutterSpeed: string | null;
  iso: number | null;
};

export type CatalogSnapshot = {
  name: string;
  path: string;
  shoots: ShootSummary[];
};

export type ShootDetails = {
  summary: ShootSummary;
  photographs: PhotographEntry[];
};

export type CatalogStateResponse =
  | {
      status: "setupRequired";
      picturesDirectory: string | null;
    }
  | {
      status: "unavailable";
      name: string;
      path: string;
      reason: string;
      picturesDirectory: string | null;
    }
  | {
      status: "ready";
      catalog: CatalogSnapshot;
    };

export type ImportProgress = {
  phase: "scanning" | "preparing" | "finalizing" | string;
  total: number | null;
  prepared: number | null;
};

export type ImportShootResult = {
  shoot: ShootDetails;
  skippedFileCount: number;
};

export type CreateCatalogInput = {
  name: string;
  parentDirectory: string;
};

export type ImportShootInput = {
  sourcePath: string;
  shootName: string;
};
