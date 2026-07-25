import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import type { PhotoAnalysis, PhotoAnalysisError } from "./types";
import "./App.css";

function formatFileSize(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function formatCoordinate(value: number | null): string {
  if (value === null) {
    return "—";
  }
  return value.toFixed(6);
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="info-row">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function MetadataSection({
  title,
  children,
}: {
  title: string;
  children: ReactNode;
}) {
  return (
    <section className="metadata-section">
      <h2>{title}</h2>
      <dl>{children}</dl>
    </section>
  );
}

function App() {
  const [analysis, setAnalysis] = useState<PhotoAnalysis | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [dragActive, setDragActive] = useState(false);
  const requestIdRef = useRef(0);

  const analyzePath = useCallback(async (path: string) => {
    const requestId = ++requestIdRef.current;
    setLoading(true);
    setError(null);
    setAnalysis(null);

    try {
      const result = await invoke<PhotoAnalysis>("analyze_photo", { path });
      if (requestId === requestIdRef.current) {
        setAnalysis(result);
        setError(null);
      }
    } catch (err) {
      if (requestId === requestIdRef.current) {
        const analysisError = err as PhotoAnalysisError;
        const message =
          analysisError.message ??
          "Something went wrong while analyzing the photo.";
        setAnalysis(null);
        setError(message);
      }
    } finally {
      if (requestId === requestIdRef.current) {
        setLoading(false);
      }
    }
  }, []);

  const handleChoosePhoto = useCallback(async () => {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Images",
          extensions: ["jpg", "jpeg", "png", "webp"],
        },
      ],
    });

    if (selected === null) {
      return;
    }

    await analyzePath(selected);
  }, [analyzePath]);

  const handlePaths = useCallback(
    async (paths: string[]) => {
      if (paths.length === 0) {
        return;
      }

      if (paths.length > 1) {
        setError("Please drop only one photo at a time.");
        return;
      }

      await analyzePath(paths[0]);
    },
    [analyzePath],
  );

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;

        if (payload.type === "enter" || payload.type === "over") {
          setDragActive(true);
          return;
        }

        if (payload.type === "leave") {
          setDragActive(false);
          return;
        }

        if (payload.type === "drop") {
          setDragActive(false);
          void handlePaths(payload.paths);
        }
      })
      .then((cleanup) => {
        if (disposed) {
          cleanup();
        } else {
          unlisten = cleanup;
        }
      });

    return () => {
      disposed = true;
      unlisten?.();
    };
  }, [handlePaths]);

  const showEmpty = !analysis && !loading && !error;

  return (
    <main className="app" aria-busy={loading}>
      <header className="app-header">
        <h1>photart</h1>
        <button
          type="button"
          onClick={() => void handleChoosePhoto()}
          disabled={loading}
        >
          {loading ? "Loading…" : "Choose Photo"}
        </button>
      </header>

      {error && (
        <div className="banner banner-error" role="alert">
          {error}
        </div>
      )}

      {loading && (
        <div className="loading-state" role="status" aria-live="polite">
          <span className="loading-spinner" aria-hidden="true" />
          <div>
            <p className="loading-title">Loading your photo</p>
            <p className="loading-subtitle">
              Reading the image and extracting its details…
            </p>
          </div>
        </div>
      )}

      {showEmpty && (
        <div
          className={`drop-zone ${dragActive ? "drop-zone-active" : ""}`}
          onClick={() => void handleChoosePhoto()}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") {
              event.preventDefault();
              void handleChoosePhoto();
            }
          }}
          role="button"
          tabIndex={0}
        >
          <p className="drop-zone-title">Drop a photo here</p>
          <p className="drop-zone-subtitle">
            JPEG, PNG, or WebP · up to 100 MiB
          </p>
        </div>
      )}

      {analysis && (
        <div className="analysis-layout">
          <div className="preview-panel">
            <img
              src={analysis.previewDataUrl}
              alt={`Preview of ${analysis.fileName}`}
              className="preview-image"
            />
          </div>

          <div className="metadata-panel">
            <MetadataSection title="File">
              <InfoRow label="Name" value={analysis.fileName} />
              <InfoRow label="Size" value={formatFileSize(analysis.fileSizeBytes)} />
              <InfoRow label="Format" value={analysis.format.toUpperCase()} />
              <InfoRow label="MIME" value={analysis.mimeType} />
              <InfoRow
                label="Dimensions"
                value={`${analysis.width} × ${analysis.height}`}
              />
              <InfoRow label="Color" value={analysis.colorType} />
            </MetadataSection>

            <MetadataSection title="Camera">
              <InfoRow label="Make" value={analysis.exif.cameraMake ?? "—"} />
              <InfoRow label="Model" value={analysis.exif.cameraModel ?? "—"} />
              <InfoRow label="Lens" value={analysis.exif.lensModel ?? "—"} />
              <InfoRow
                label="Captured"
                value={analysis.exif.capturedAt ?? "—"}
              />
            </MetadataSection>

            <MetadataSection title="Exposure">
              <InfoRow
                label="Shutter"
                value={analysis.exif.exposureTime ?? "—"}
              />
              <InfoRow label="Aperture" value={analysis.exif.aperture ?? "—"} />
              <InfoRow label="ISO" value={analysis.exif.iso ?? "—"} />
              <InfoRow
                label="Focal length"
                value={analysis.exif.focalLength ?? "—"}
              />
            </MetadataSection>

            <MetadataSection title="Location">
              <InfoRow
                label="Latitude"
                value={formatCoordinate(analysis.exif.latitude)}
              />
              <InfoRow
                label="Longitude"
                value={formatCoordinate(analysis.exif.longitude)}
              />
            </MetadataSection>
          </div>
        </div>
      )}
    </main>
  );
}

export default App;
