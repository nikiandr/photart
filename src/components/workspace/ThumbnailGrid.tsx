import { Tooltip } from "@base-ui/react/tooltip";
import { memo, useCallback, useLayoutEffect, useMemo, useRef, useState } from "react";

import type { PhotographEntry } from "@/types";

const targetRowHeight = 120;

export type ThumbnailGridProps = {
  photographs: PhotographEntry[];
  jpegCompanions?: ReadonlyMap<string, PhotographEntry>;
  onOpenPhotograph: (relativePath: string) => void;
};

function layoutRows(photographs: PhotographEntry[], width: number, gap: number) {
  const rows = [];
  let items: { photograph: PhotographEntry; aspectRatio: number }[] = [];
  let totalRatio = 0;

  for (const photograph of photographs) {
    const aspectRatio = photograph.width / photograph.height;
    const remainingWidth = width - items.length * gap;
    const heightWithPhoto = remainingWidth / (totalRatio + aspectRatio);

    if (items.length > 0 && heightWithPhoto < targetRowHeight) {
      const heightWithoutPhoto = (remainingWidth + gap) / totalRatio;

      if (
        remainingWidth <= 0 ||
        heightWithoutPhoto - targetRowHeight < targetRowHeight - heightWithPhoto
      ) {
        rows.push({ items, height: heightWithoutPhoto });
        items = [];
        totalRatio = 0;
      }
    }

    items.push({ photograph, aspectRatio });
    totalRatio += aspectRatio;

    const height = (width - (items.length - 1) * gap) / totalRatio;
    if (height <= targetRowHeight) {
      rows.push({ items, height });
      items = [];
      totalRatio = 0;
    }
  }

  if (items.length > 0) {
    rows.push({ items, height: targetRowHeight });
  }

  return rows;
}

export const ThumbnailGrid = memo(function ThumbnailGrid({
  photographs,
  jpegCompanions,
  onOpenPhotograph,
}: ThumbnailGridProps) {
  const [size, setSize] = useState({ width: 0, gap: 0 });
  const [filenameTooltip] = useState(() => Tooltip.createHandle<string>());
  const [expandedPairs, setExpandedPairs] = useState<ReadonlySet<string>>(() => new Set());
  const gridElement = useRef<HTMLDivElement | null>(null);
  const returnToggleFocus = useRef<string | null>(null);

  const measureGrid = useCallback((grid: HTMLDivElement | null) => {
    if (grid) {
      gridElement.current = grid;
      const updateSize = (width: number) => {
        const gap = parseFloat(getComputedStyle(grid).columnGap);
        setSize((previous) =>
          previous.width === width && previous.gap === gap ? previous : { width, gap },
        );
      };

      updateSize(grid.getBoundingClientRect().width);
      const observer = new ResizeObserver(([entry]) => {
        updateSize(entry.contentRect.width);
      });
      observer.observe(grid);
      return () => {
        gridElement.current = null;
        observer.disconnect();
      };
    }
  }, []);

  const visible = useMemo(() => {
    const jpegPaths = new Set<string>();
    const items = photographs.flatMap((photo) => {
      const companion = jpegCompanions?.get(photo.relativePath);
      if (companion && expandedPairs.has(photo.relativePath)) {
        jpegPaths.add(companion.relativePath);
        return [photo, companion];
      }
      return [photo];
    });
    return { items, jpegPaths };
  }, [photographs, jpegCompanions, expandedPairs]);
  const rows = useMemo(() => size.width > 0 ? layoutRows(visible.items, size.width, size.gap) : [], [visible, size]);

  useLayoutEffect(() => {
    const path = returnToggleFocus.current;
    if (path !== null) {
      gridElement.current?.querySelector<HTMLButtonElement>(
        `[data-pair-path="${CSS.escape(path)}"]`,
      )?.focus({ preventScroll: true });
      returnToggleFocus.current = null;
    }
  }, [expandedPairs]);

  function togglePair(path: string) {
    returnToggleFocus.current = path;
    setExpandedPairs((current) => {
      const updated = new Set(current);
      if (updated.has(path)) updated.delete(path);
      else updated.add(path);
      return updated;
    });
  }

  return photographs.length === 0 ? (
    <p className="py-8 text-center text-sm text-muted-foreground">
      No photographs in this Shoot.
    </p>
  ) : (
    <>
      <div ref={measureGrid} role="list" className="grid min-w-0 gap-4">
        {rows.map(({ items, height }) => (
          <div
            key={items[0].photograph.relativePath}
            role="presentation"
            className="flex min-w-0 gap-4"
          >
            {items.map(({ photograph, aspectRatio }) => {
              const path = photograph.relativePath;
              const paired = jpegCompanions?.has(path);
              const expanded = paired && expandedPairs.has(path);
              const format = expanded ? "RAW" : visible.jpegPaths.has(path) ? "JPEG" : null;
              return (
                <div
                  key={path}
                  role="listitem"
                  className="relative min-w-0"
                  style={{ width: height * aspectRatio, height }}
                >
                  <Tooltip.Trigger
                    handle={filenameTooltip}
                    payload={photograph.fileName}
                    render={
                      <button
                        type="button"
                        data-photograph-path={path}
                        onClick={() => onOpenPhotograph(path)}
                      >
                        <img
                          src={photograph.thumbnailUrl}
                          alt={photograph.fileName}
                          width={photograph.width}
                          height={photograph.height}
                          loading="lazy"
                          decoding="async"
                          className="size-full object-contain"
                        />
                      </button>
                    }
                    className="block size-full cursor-pointer bg-muted focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring"
                  />
                  {paired ? (
                    <button
                      type="button"
                      data-pair-path={path}
                      aria-expanded={expanded}
                      aria-label={`RAW + JPEG: ${photograph.fileName}`}
                      onClick={() => togglePair(path)}
                      className="absolute top-1 right-1 cursor-pointer border border-border bg-popover px-1.5 py-1 font-mono text-[0.6875rem] text-popover-foreground focus-visible:outline-2 focus-visible:outline-ring"
                    >
                      RAW + JPEG
                    </button>
                  ) : null}
                  {format ? (
                    <span className="pointer-events-none absolute bottom-1 left-1 bg-popover px-1.5 py-0.5 font-mono text-[0.6875rem] text-popover-foreground">
                      {format}
                    </span>
                  ) : null}
                </div>
              );
            })}
          </div>
        ))}
      </div>
      <Tooltip.Root handle={filenameTooltip}>
        {({ payload }) => (
          <Tooltip.Portal>
            <Tooltip.Positioner side="bottom" sideOffset={4} className="z-10">
              <Tooltip.Popup
                role="tooltip"
                className="max-w-(--available-width) rounded-sm border border-border bg-popover px-2 py-1 font-mono text-[0.6875rem] wrap-anywhere text-popover-foreground"
              >
                {payload}
              </Tooltip.Popup>
            </Tooltip.Positioner>
          </Tooltip.Portal>
        )}
      </Tooltip.Root>
    </>
  );
});
