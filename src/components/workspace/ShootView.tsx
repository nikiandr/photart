import { Info } from "@phosphor-icons/react/dist/csr/Info";
import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";

import { ThumbnailGrid } from "@/components/workspace/ThumbnailGrid";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import type { ShootDetails } from "@/types";

export type ShootViewProps = {
  shoot: ShootDetails;
  skippedNoticeCount: number | null;
  loading?: boolean;
};

export function ShootView({
  shoot,
  skippedNoticeCount,
  loading = false,
}: ShootViewProps) {
  const count = shoot.summary.photographCount;
  const countLabel =
    count === 1 ? "1 photograph" : `${count} photographs`;

  return (
    <div className="flex h-full min-h-0 flex-col overflow-hidden">
      <div className="shrink-0 space-y-4 px-6 pt-6 pb-4">
        <div className="space-y-1">
          <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
            {shoot.summary.name}
          </h1>
          <p className="font-mono text-[0.8125rem] text-muted-foreground">
            {countLabel}
          </p>
        </div>

        {skippedNoticeCount != null && skippedNoticeCount > 0 ? (
          <Alert>
            <Info aria-hidden="true" />
            <AlertDescription>
              {skippedNoticeCount} unsupported files were not imported.
            </AlertDescription>
          </Alert>
        ) : null}

        <Separator />
      </div>

      <div className="min-h-0 flex-1 overflow-y-auto px-6 pb-6">
        {loading ? (
          <div
            className="flex h-40 items-center justify-center"
            role="status"
            aria-live="polite"
          >
            <SpinnerGap
              className="size-6 animate-spin motion-reduce:animate-none"
              aria-hidden="true"
            />
            <span className="sr-only">Loading shoot</span>
          </div>
        ) : (
          <ThumbnailGrid photographs={shoot.photographs} />
        )}
      </div>
    </div>
  );
}
