import { useCallback, useEffect, useRef, useState, type ReactNode } from "react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { Aperture } from "@phosphor-icons/react/dist/csr/Aperture";
import { ArrowClockwise } from "@phosphor-icons/react/dist/csr/ArrowClockwise";
import { Camera } from "@phosphor-icons/react/dist/csr/Camera";
import { FileImage } from "@phosphor-icons/react/dist/csr/FileImage";
import { ImageSquare } from "@phosphor-icons/react/dist/csr/ImageSquare";
import { MapPin } from "@phosphor-icons/react/dist/csr/MapPin";
import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";
import { UploadSimple } from "@phosphor-icons/react/dist/csr/UploadSimple";
import { WarningCircle } from "@phosphor-icons/react/dist/csr/WarningCircle";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import type { PhotoAnalysis, PhotoAnalysisError } from "./types";

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
  return value === null ? "—" : value.toFixed(6);
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[7.5rem_minmax(0,1fr)] gap-3 py-2.5 text-sm first:pt-0 last:pb-0">
      <dt className="font-sans text-muted-foreground">{label}</dt>
      <dd className="min-w-0 break-words text-right font-mono text-[0.8125rem] font-medium text-foreground">
        {value}
      </dd>
    </div>
  );
}

function MetadataSection({
  title,
  icon,
  children,
}: {
  title: string;
  icon: ReactNode;
  children: ReactNode;
}) {
  return (
    <section className="border-t border-border pt-4">
      <h2 className="mb-2 flex items-center gap-2 font-sans text-sm font-medium text-foreground">
        <span className="text-muted-foreground" aria-hidden="true">
          {icon}
        </span>
        {title}
      </h2>
      <dl className="divide-y divide-border">{children}</dl>
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

    if (selected !== null) {
      await analyzePath(selected);
    }
  }, [analyzePath]);

  const handlePaths = useCallback(
    async (paths: string[]) => {
      if (paths.length > 1) {
        setError("Please drop only one photo at a time.");
      } else if (paths.length === 1) {
        await analyzePath(paths[0]);
      }
    },
    [analyzePath],
  );

  useEffect(() => {
    const atmosphere = analysis ? "review" : "gallery";
    document.documentElement.setAttribute("data-atmosphere", atmosphere);

    return () => {
      document.documentElement.setAttribute("data-atmosphere", "gallery");
    };
  }, [analysis]);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;

    void getCurrentWebview()
      .onDragDropEvent((event) => {
        const payload = event.payload;

        if (payload.type === "enter" || payload.type === "over") {
          setDragActive(true);
        } else if (payload.type === "leave") {
          setDragActive(false);
        } else if (payload.type === "drop") {
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

  const showUploader = !analysis && !loading;

  return (
    <main className="min-h-svh bg-background p-3 sm:p-5" aria-busy={loading}>
      <div className="mx-auto flex min-h-[calc(100svh-1.5rem)] max-w-[1440px] flex-col gap-4 sm:min-h-[calc(100svh-2.5rem)]">
        <header className="flex items-center justify-between gap-4">
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex size-9 shrink-0 items-center justify-center bg-foreground text-background">
              <ImageSquare className="size-5" weight="duotone" aria-hidden="true" />
            </div>
            <div className="min-w-0">
              <h1 className="font-sans text-sm font-semibold tracking-tight">
                photart
              </h1>
              <p className="truncate text-xs text-muted-foreground">
                Local photo inspector
              </p>
            </div>
          </div>

          <Button
            type="button"
            variant="outline"
            onClick={() => void handleChoosePhoto()}
            disabled={loading}
          >
            {loading ? (
              <SpinnerGap
                data-icon="inline-start"
                className="animate-spin motion-reduce:animate-none"
                aria-hidden="true"
              />
            ) : (
              <UploadSimple data-icon="inline-start" aria-hidden="true" />
            )}
            {loading ? "Analyzing…" : "Choose photo"}
          </Button>
        </header>

        <Separator />

        {error ? (
          <Alert variant="destructive">
            <WarningCircle weight="fill" aria-hidden="true" />
            <AlertTitle>Couldn’t analyze that photo</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        ) : null}

        {loading ? (
          <div
            className="flex min-h-[32rem] flex-1 flex-col items-center justify-center gap-5 text-center"
            role="status"
            aria-live="polite"
          >
            <div className="flex size-14 items-center justify-center border border-border bg-muted">
              <SpinnerGap
                className="size-7 animate-spin motion-reduce:animate-none"
                aria-hidden="true"
              />
            </div>
            <div className="space-y-1.5">
              <p className="font-sans text-sm font-medium">
                Analyzing your photo
              </p>
              <p className="max-w-sm text-sm/relaxed text-muted-foreground">
                Reading the image, generating a preview, and extracting its
                metadata.
              </p>
            </div>
          </div>
        ) : null}

        {showUploader ? (
          <div
            className={cn(
              "flex min-h-[32rem] flex-1 border border-dashed transition-colors motion-reduce:transition-none",
              dragActive
                ? "border-foreground bg-accent"
                : "border-border bg-card",
            )}
          >
            <Button
              type="button"
              variant="ghost"
              className="h-full min-h-[32rem] w-full flex-1 flex-col gap-5 whitespace-normal rounded-none p-8 text-center hover:bg-muted/40"
              onClick={() => void handleChoosePhoto()}
            >
              <span className="relative flex size-16 items-center justify-center border border-border bg-background text-foreground">
                <UploadSimple className="size-7" weight="duotone" aria-hidden="true" />
                <span className="absolute -right-1.5 -bottom-1.5 size-3 bg-foreground" />
              </span>

              <span className="flex max-w-lg flex-col items-center gap-2">
                <span className="font-sans text-lg font-medium tracking-tight">
                  {dragActive ? "Drop your photo here" : "Upload a photo"}
                </span>
                <span className="text-sm/relaxed font-normal text-muted-foreground">
                  {dragActive
                    ? "Release to analyze this file."
                    : "Drag and drop anywhere in this window, or click to choose a file from your Mac."}
                </span>
              </span>

              <span className="flex items-center gap-2 font-mono text-[0.6875rem] font-normal text-muted-foreground">
                <span>JPEG</span>
                <span aria-hidden="true">·</span>
                <span>PNG</span>
                <span aria-hidden="true">·</span>
                <span>WEBP</span>
                <span aria-hidden="true">·</span>
                <span>Up to 100 MiB</span>
              </span>
            </Button>
          </div>
        ) : null}

        {analysis ? (
          <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1.35fr)_minmax(20rem,0.65fr)]">
            <section className="flex min-h-[32rem] flex-col bg-stage text-stage-foreground">
              <div className="flex items-start justify-between gap-3 border-b border-border px-4 py-3">
                <div className="min-w-0">
                  <p className="truncate font-mono text-[0.8125rem] font-medium">
                    {analysis.fileName}
                  </p>
                  <p className="mt-0.5 font-mono text-xs text-muted-foreground">
                    {formatFileSize(analysis.fileSizeBytes)}
                    <span aria-hidden="true"> · </span>
                    {analysis.format.toUpperCase()}
                    <span aria-hidden="true"> · </span>
                    {analysis.width} × {analysis.height}
                  </p>
                </div>
              </div>
              <div className="flex min-h-0 flex-1 items-center justify-center p-3">
                <img
                  src={analysis.previewDataUrl}
                  alt={`Preview of ${analysis.fileName}`}
                  className="max-h-[calc(100svh-11rem)] max-w-full rounded-none object-contain"
                />
              </div>
            </section>

            <aside className="flex min-h-0 flex-col gap-1 overflow-y-auto pr-px">
              <MetadataSection
                title="File"
                icon={<FileImage className="size-4" />}
              >
                <InfoRow label="Name" value={analysis.fileName} />
                <InfoRow
                  label="Size"
                  value={formatFileSize(analysis.fileSizeBytes)}
                />
                <InfoRow
                  label="Format"
                  value={analysis.format.toUpperCase()}
                />
                <InfoRow label="MIME" value={analysis.mimeType} />
                <InfoRow
                  label="Dimensions"
                  value={`${analysis.width} × ${analysis.height}`}
                />
                <InfoRow label="Color" value={analysis.colorType} />
              </MetadataSection>

              <MetadataSection
                title="Camera"
                icon={<Camera className="size-4" />}
              >
                <InfoRow label="Make" value={analysis.exif.cameraMake ?? "—"} />
                <InfoRow
                  label="Model"
                  value={analysis.exif.cameraModel ?? "—"}
                />
                <InfoRow label="Lens" value={analysis.exif.lensModel ?? "—"} />
                <InfoRow
                  label="Captured"
                  value={analysis.exif.capturedAt ?? "—"}
                />
              </MetadataSection>

              <MetadataSection
                title="Exposure"
                icon={<Aperture className="size-4" />}
              >
                <InfoRow
                  label="Shutter"
                  value={analysis.exif.exposureTime ?? "—"}
                />
                <InfoRow
                  label="Aperture"
                  value={analysis.exif.aperture ?? "—"}
                />
                <InfoRow label="ISO" value={analysis.exif.iso ?? "—"} />
                <InfoRow
                  label="Focal length"
                  value={analysis.exif.focalLength ?? "—"}
                />
              </MetadataSection>

              <MetadataSection
                title="Location"
                icon={<MapPin className="size-4" />}
              >
                <InfoRow
                  label="Latitude"
                  value={formatCoordinate(analysis.exif.latitude)}
                />
                <InfoRow
                  label="Longitude"
                  value={formatCoordinate(analysis.exif.longitude)}
                />
              </MetadataSection>

              <div className="pt-4">
                <Button
                  type="button"
                  variant="outline"
                  className="w-full"
                  onClick={() => void handleChoosePhoto()}
                >
                  <ArrowClockwise data-icon="inline-start" aria-hidden="true" />
                  Analyze another photo
                </Button>
              </div>
            </aside>
          </div>
        ) : null}
      </div>
    </main>
  );
}

export default App;
