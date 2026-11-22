import { Info } from "@phosphor-icons/react/dist/csr/Info";
import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";
import { useCallback, useLayoutEffect, useRef, useState } from "react";

import { PhotographDetailView } from "@/components/workspace/PhotographDetailView";
import { ThumbnailGrid } from "@/components/workspace/ThumbnailGrid";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import { loadPhotographCapture } from "@/lib/catalog";
import { isDevEdition } from "@/lib/edition";
import type { ShootDetails } from "@/types";

export type ShootViewProps = {
  shoot: ShootDetails;
  skippedNoticeCount: number | null;
  loading?: boolean;
  loadCapture?: typeof loadPhotographCapture;
};

export function ShootView({
  shoot,
  skippedNoticeCount,
  loading = false,
  loadCapture = loadPhotographCapture,
}: ShootViewProps) {
  const [active, setActive] = useState<{
    path: string;
    direction: -1 | 1;
    openedAt: number | null;
  } | null>(null);
  const grid = useRef<HTMLDivElement>(null);
  const returnFocusPath = useRef<string | null>(null);
  const activeIndex = shoot.photographs.findIndex((photo) => photo.relativePath === active?.path);
  const detailOpen = !loading && active !== null && activeIndex >= 0;
  const openPhotograph = useCallback((path: string, direction: -1 | 1 = 1) => {
    const openedAt = isDevEdition() ? performance.now() : null;
    setActive({ path, direction, openedAt });
  }, []);

  useLayoutEffect(() => {
    const path = returnFocusPath.current;
    if (!detailOpen && path !== null) {
      const thumbnail = grid.current?.querySelector<HTMLButtonElement>(
        `[data-photograph-path="${CSS.escape(path)}"]`,
      );
      thumbnail?.focus({ preventScroll: true });
      thumbnail?.scrollIntoView({ block: "nearest" });
      returnFocusPath.current = null;
    }
  }, [detailOpen]);

  const count = shoot.summary.photographCount;
  const countLabel =
    count === 1 ? "1 photograph" : `${count} photographs`;

  return (
    <div className="relative h-full min-h-0 overflow-hidden">
      <div
        className={`flex h-full min-h-0 flex-col overflow-hidden${detailOpen ? " invisible" : ""}`}
        inert={detailOpen}
      >
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

        <div ref={grid} className="min-h-0 flex-1 overflow-y-auto px-6 pb-6">
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
            <ThumbnailGrid photographs={shoot.photographs} onOpenPhotograph={openPhotograph} />
          )}
        </div>
      </div>
      {detailOpen ? (
        <PhotographDetailView
          shootName={shoot.summary.name}
          photographs={shoot.photographs}
          activeIndex={activeIndex}
          direction={active.direction}
          openedAt={active.openedAt}
          onOpenPhotograph={openPhotograph}
          onClose={() => {
            returnFocusPath.current = active.path;
            setActive(null);
          }}
          loadCapture={loadCapture}
        />
      ) : null}
    </div>
  );
}
