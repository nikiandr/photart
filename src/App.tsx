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
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
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
  return value === null ? "—" : value.toFixed(6);
}

function InfoRow({ label, value }: { label: string; value: string }) {
  return (
    <div className="grid grid-cols-[7.5rem_minmax(0,1fr)] gap-3 py-2.5 text-xs first:pt-0 last:pb-0">
      <dt className="text-muted-foreground">{label}</dt>
      <dd className="min-w-0 break-words text-right font-medium text-foreground">
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
    <Card size="sm">
      <CardHeader className="border-b">
        <CardTitle className="flex items-center gap-2">
          <span className="text-muted-foreground" aria-hidden="true">
            {icon}
          </span>
          {title}
        </CardTitle>
      </CardHeader>
      <CardContent>
        <dl className="divide-y divide-border">{children}</dl>
      </CardContent>
    </Card>
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
    const colorScheme = window.matchMedia("(prefers-color-scheme: dark)");
    const syncColorScheme = () => {
      document.documentElement.classList.toggle("dark", colorScheme.matches);
    };

    syncColorScheme();
    colorScheme.addEventListener("change", syncColorScheme);

    return () => {
      colorScheme.removeEventListener("change", syncColorScheme);
    };
  }, []);

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
    <main className="min-h-svh bg-muted/30 p-3 sm:p-5" aria-busy={loading}>
      <div className="mx-auto flex min-h-[calc(100svh-1.5rem)] max-w-[1440px] flex-col gap-4 sm:min-h-[calc(100svh-2.5rem)]">
        <header className="flex items-center justify-between gap-4">
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex size-9 shrink-0 items-center justify-center bg-foreground text-background">
              <ImageSquare className="size-5" weight="duotone" aria-hidden="true" />
            </div>
            <div className="min-w-0">
              <h1 className="font-heading text-sm font-semibold tracking-tight">
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
          <Card className="min-h-[32rem] flex-1">
            <CardContent
              className="flex flex-1 flex-col items-center justify-center gap-5 text-center"
              role="status"
              aria-live="polite"
            >
              <div className="flex size-14 items-center justify-center border bg-muted">
                <SpinnerGap
                  className="size-7 animate-spin motion-reduce:animate-none"
                  aria-hidden="true"
                />
              </div>
              <div className="space-y-1.5">
                <p className="font-heading text-sm font-medium">
                  Analyzing your photo
                </p>
                <p className="max-w-sm text-xs/relaxed text-muted-foreground">
                  Reading the image, generating a preview, and extracting its
                  metadata.
                </p>
              </div>
              <Badge variant="outline">Processing locally</Badge>
            </CardContent>
          </Card>
        ) : null}

        {showUploader ? (
          <Card
            className={cn(
              "min-h-[32rem] flex-1 border border-dashed py-0 ring-0 transition-colors",
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
              <span className="relative flex size-16 items-center justify-center border bg-background text-foreground shadow-xs">
                <UploadSimple className="size-7" weight="duotone" aria-hidden="true" />
                <span className="absolute -right-1.5 -bottom-1.5 size-3 bg-foreground" />
              </span>

              <span className="flex max-w-lg flex-col items-center gap-2">
                <Badge variant={dragActive ? "default" : "secondary"}>
                  {dragActive ? "Release to analyze" : "Local processing"}
                </Badge>
                <span className="font-heading text-lg font-medium tracking-tight">
                  {dragActive ? "Drop your photo here" : "Upload a photo"}
                </span>
                <span className="text-xs/relaxed font-normal text-muted-foreground">
                  Drag and drop anywhere in this window, or click to choose a
                  file from your Mac.
                </span>
              </span>

              <span className="flex items-center gap-2 text-[0.6875rem] font-normal text-muted-foreground">
                <span>JPEG</span>
                <span aria-hidden="true">·</span>
                <span>PNG</span>
                <span aria-hidden="true">·</span>
                <span>WEBP</span>
                <span aria-hidden="true">·</span>
                <span>Up to 100 MiB</span>
              </span>
            </Button>
          </Card>
        ) : null}

        {analysis ? (
          <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[minmax(0,1.35fr)_minmax(20rem,0.65fr)]">
            <Card className="min-h-[32rem]">
              <CardHeader className="border-b">
                <CardTitle className="truncate pr-4">
                  {analysis.fileName}
                </CardTitle>
                <CardDescription>
                  {formatFileSize(analysis.fileSizeBytes)}
                </CardDescription>
                <CardAction className="flex items-center gap-1.5">
                  <Badge variant="secondary">
                    {analysis.format.toUpperCase()}
                  </Badge>
                  <Badge variant="outline">
                    {analysis.width} × {analysis.height}
                  </Badge>
                </CardAction>
              </CardHeader>
              <CardContent className="flex min-h-0 flex-1 items-center justify-center bg-muted/40 p-3">
                <img
                  src={analysis.previewDataUrl}
                  alt={`Preview of ${analysis.fileName}`}
                  className="max-h-[calc(100svh-11rem)] max-w-full object-contain shadow-sm"
                />
              </CardContent>
            </Card>

            <aside className="flex min-h-0 flex-col gap-3 overflow-y-auto pr-px">
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

              <Button
                type="button"
                variant="outline"
                className="w-full"
                onClick={() => void handleChoosePhoto()}
              >
                <ArrowClockwise data-icon="inline-start" aria-hidden="true" />
                Analyze another photo
              </Button>
            </aside>
          </div>
        ) : null}
      </div>
    </main>
  );
}

export default App;
