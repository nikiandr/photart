import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";

import { Button } from "@/components/ui/button";
import type { ImportProgress } from "@/types";

export type ImportProgressViewProps = {
  shootName: string;
  progress: ImportProgress | null;
  onCancel: () => void;
};

type ProgressPresentation = {
  label: string;
  detail: string | null;
  indeterminate: boolean;
  value: number;
  max: number;
  percent: number | null;
};

function presentProgress(progress: ImportProgress | null): ProgressPresentation {
  if (!progress) {
    return {
      label: "Starting import…",
      detail: null,
      indeterminate: true,
      value: 0,
      max: 1,
      percent: null,
    };
  }

  switch (progress.phase) {
    case "scanning":
      return {
        label: "Scanning photographs…",
        detail: "Looking through the Source folder.",
        indeterminate: true,
        value: 0,
        max: 1,
        percent: null,
      };
    case "preparing": {
      const prepared = progress.prepared ?? 0;
      const total = progress.total ?? 0;
      if (total > 0) {
        return {
          label: "Preparing photographs…",
          detail: `Prepared ${prepared} of ${total}`,
          indeterminate: false,
          value: prepared,
          max: total,
          percent: Math.round((prepared / total) * 100),
        };
      }
      return {
        label: "Preparing photographs…",
        detail: "Checking image files and building thumbnails.",
        indeterminate: true,
        value: 0,
        max: 1,
        percent: null,
      };
    }
    case "copying": {
      const copied = progress.copied ?? 0;
      const total = progress.total ?? 0;
      if (total > 0) {
        return {
          label: "Copying photographs…",
          detail: `Copied ${copied} of ${total} photographs`,
          indeterminate: false,
          value: copied,
          max: total,
          percent: Math.round((copied / total) * 100),
        };
      }
      return {
        label: "Copying photographs…",
        detail: null,
        indeterminate: true,
        value: 0,
        max: 1,
        percent: null,
      };
    }
    case "finalizing":
      return {
        label: "Finalizing Shoot…",
        detail: "Writing the Catalog index.",
        indeterminate: false,
        value: 1,
        max: 1,
        percent: 100,
      };
    default:
      return {
        label: "Importing photographs…",
        detail: null,
        indeterminate: true,
        value: 0,
        max: 1,
        percent: null,
      };
  }
}

export function ImportProgressView({
  shootName,
  progress,
  onCancel,
}: ImportProgressViewProps) {
  const presentation = presentProgress(progress);

  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-8">
      <div
        className="flex w-full max-w-md flex-col items-center gap-6 text-center"
        aria-live="polite"
        aria-busy="true"
      >
        <div className="space-y-2">
          <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
            Importing
          </h1>
          <p className="font-sans text-sm text-muted-foreground">{shootName}</p>
        </div>

        <div className="flex w-full flex-col items-center gap-3">
          {presentation.indeterminate ? (
            <SpinnerGap
              className="size-7 animate-spin text-foreground motion-reduce:animate-none"
              aria-hidden="true"
            />
          ) : (
            <p className="font-mono text-xs tabular-nums text-muted-foreground">
              {presentation.percent}%
            </p>
          )}

          {presentation.indeterminate ? (
            <div
              className="relative h-1.5 w-full overflow-hidden border border-border bg-muted"
              role="progressbar"
              aria-label={presentation.label}
            >
              <div className="absolute inset-y-0 w-1/3 bg-foreground motion-safe:animate-[import-indeterminate_1.4s_ease-in-out_infinite] motion-reduce:left-1/3" />
            </div>
          ) : (
            <progress
              className="h-1.5 w-full overflow-hidden border border-border bg-muted accent-foreground [&::-webkit-progress-bar]:bg-muted [&::-webkit-progress-value]:bg-foreground [&::-moz-progress-bar]:bg-foreground"
              value={presentation.value}
              max={presentation.max}
              aria-label={presentation.label}
            />
          )}

          <div className="space-y-1" role="status">
            <p className="font-sans text-sm font-medium text-foreground">
              {presentation.label}
            </p>
            {presentation.detail ? (
              <p className="font-mono text-[0.8125rem] text-muted-foreground">
                {presentation.detail}
              </p>
            ) : null}
          </div>
        </div>

        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel Import
        </Button>
      </div>
    </div>
  );
}
