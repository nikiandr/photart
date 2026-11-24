import { Info } from "@phosphor-icons/react/dist/csr/Info";
import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";
import { useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";

import { PhotographDetailView } from "@/components/workspace/PhotographDetailView";
import { ThumbnailGrid } from "@/components/workspace/ThumbnailGrid";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { loadPhotographCapture } from "@/lib/catalog";
import { isDevEdition } from "@/lib/edition";
import type { PhotographEntry, ShootDetails } from "@/types";

export type ShootViewProps = {
  shoot: ShootDetails;
  skippedNoticeCount: number | null;
  loading?: boolean;
  loadCapture?: typeof loadPhotographCapture;
  onSetPairLinked?: (rawPath: string, linked: boolean) => Promise<void>;
};

export function ShootView({
  shoot,
  skippedNoticeCount,
  loading = false,
  loadCapture = loadPhotographCapture,
  onSetPairLinked,
}: ShootViewProps) {
  const [active, setActive] = useState<{
    path: string;
    showJpeg: boolean;
    direction: -1 | 1;
    openedAt: number | null;
  } | null>(null);
  const [undoPath, setUndoPath] = useState<string | null>(null);
  const [pairBusy, setPairBusy] = useState(false);
  const [pairError, setPairError] = useState(false);
  const grid = useRef<HTMLDivElement>(null);
  const returnFocusPath = useRef<string | null>(null);
  const { photographs, jpegCompanions, rawPaths } = useMemo(() => {
    const files = new Map(shoot.photographs.map((photo) => [photo.relativePath, photo]));
    const jpegCompanions = new Map<string, PhotographEntry>();
    const rawPaths = new Map<string, string>();
    for (const photo of shoot.photographs) {
      const jpeg = photo.jpegCompanionPath ? files.get(photo.jpegCompanionPath) : undefined;
      if (jpeg) {
        jpegCompanions.set(photo.relativePath, jpeg);
        rawPaths.set(jpeg.relativePath, photo.relativePath);
      }
    }
    return {
      photographs: shoot.photographs.filter((photo) => !rawPaths.has(photo.relativePath)),
      jpegCompanions,
      rawPaths,
    };
  }, [shoot.photographs]);
  const activeIndex = photographs.findIndex((photo) => photo.relativePath === active?.path);
  const detailOpen = !loading && active !== null && activeIndex >= 0;
  const activeCompanion = active ? jpegCompanions.get(active.path) : undefined;
  const canRelinkPair = photographs[activeIndex]?.canRelinkJpegCompanion;
  const openPhotograph = useCallback((path: string, direction: -1 | 1 = 1) => {
    const openedAt = isDevEdition() ? performance.now() : null;
    const rawPath = rawPaths.get(path);
    setActive({ path: rawPath ?? path, showJpeg: rawPath !== undefined, direction, openedAt });
  }, [rawPaths]);

  async function changePair(rawPath: string, linked: boolean) {
    if (onSetPairLinked && !pairBusy) {
      setPairBusy(true);
      setPairError(false);
      try {
        await onSetPairLinked(rawPath, linked);
        setUndoPath(linked ? null : rawPath);
        if (!linked || !detailOpen) {
          returnFocusPath.current = rawPath;
        }
        if (!linked) {
          setActive((current) => current?.path === rawPath ? null : current);
        }
      } catch {
        setPairError(true);
      } finally {
        setPairBusy(false);
      }
    }
  }

  useLayoutEffect(() => {
    const path = returnFocusPath.current;
    if (!detailOpen && path !== null) {
      const findThumbnail = (relativePath: string) => grid.current?.querySelector<HTMLButtonElement>(
        `[data-photograph-path="${CSS.escape(relativePath)}"]`,
      );
      const thumbnail = findThumbnail(path) ?? findThumbnail(rawPaths.get(path) ?? path);
      thumbnail?.focus({ preventScroll: true });
      thumbnail?.scrollIntoView({ block: "nearest" });
      returnFocusPath.current = null;
    }
  }, [detailOpen, rawPaths]);

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
          {undoPath ? (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <span role="status">RAW and JPEG unlinked.</span>
              <Button variant="ghost" size="sm" disabled={pairBusy} onClick={() => void changePair(undoPath, true)}>
                Undo
              </Button>
            </div>
          ) : null}
          {pairError && !detailOpen ? <p role="alert" className="text-sm">Could not update the RAW + JPEG pair.</p> : null}

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
            <ThumbnailGrid photographs={photographs} jpegCompanions={jpegCompanions} onOpenPhotograph={openPhotograph} />
          )}
        </div>
      </div>
      {detailOpen ? (
        <PhotographDetailView
          shootName={shoot.summary.name}
          photographs={photographs}
          activeIndex={activeIndex}
          jpegCompanion={activeCompanion}
          showJpeg={active.showJpeg}
          onShowJpeg={(showJpeg) => {
            const openedAt = isDevEdition() ? performance.now() : null;
            setActive((current) => current && current.showJpeg !== showJpeg
              ? { ...current, showJpeg, openedAt }
              : current);
          }}
          onTogglePairLink={onSetPairLinked && (activeCompanion || canRelinkPair)
            ? () => void changePair(active.path, !activeCompanion)
            : undefined}
          pairBusy={pairBusy}
          pairError={pairError}
          direction={active.direction}
          openedAt={active.openedAt}
          onOpenPhotograph={openPhotograph}
          onClose={() => {
            returnFocusPath.current = (active.showJpeg && activeCompanion?.relativePath) || active.path;
            setActive(null);
          }}
          loadCapture={loadCapture}
        />
      ) : null}
    </div>
  );
}
