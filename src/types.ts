export interface PhotoExif {
  capturedAt: string | null;
  cameraMake: string | null;
  cameraModel: string | null;
  lensModel: string | null;
  exposureTime: string | null;
  aperture: string | null;
  iso: string | null;
  focalLength: string | null;
  latitude: number | null;
  longitude: number | null;
}

export interface PhotoAnalysis {
  fileName: string;
  fileSizeBytes: number;
  format: string;
  mimeType: string;
  width: number;
  height: number;
  colorType: string;
  previewDataUrl: string;
  exif: PhotoExif;
}

export interface PhotoAnalysisError {
  code: string;
  message: string;
}
