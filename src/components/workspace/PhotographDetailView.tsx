import { CaretLeft } from "@phosphor-icons/react/dist/csr/CaretLeft";
import { CaretRight } from "@phosphor-icons/react/dist/csr/CaretRight";
import { useLayoutEffect, useRef, useState } from "react";
import type { KeyboardEvent, SyntheticEvent } from "react";

import { Button } from "@/components/ui/button";
import type { loadPhotographCapture } from "@/lib/catalog";
import type { PhotographEntry } from "@/types";
import { usePhotographCapture } from "./usePhotographCapture";

export type PhotographDetailViewProps = {
  shootName: string;
  photographs: PhotographEntry[];
  activeIndex: number;
  direction: -1 | 1;
  openedAt: number | null;
  onOpenPhotograph: (relativePath: string, direction: -1 | 1) => void;
  onClose: () => void;
  loadCapture: typeof loadPhotographCapture;
};

type OriginalStatus = "ready" | "unavailable";

function cancelRemovedImage(image: HTMLImageElement | null) {
  if (image) {
    return () => {
      queueMicrotask(() => {
        if (!image.isConnected) image.removeAttribute("src");
      });
    };
  }
}

export function PhotographDetailView({
  shootName,
  photographs,
  activeIndex,
  direction,
  openedAt,
  onOpenPhotograph,
  onClose,
  loadCapture,
}: PhotographDetailViewProps) {
  const active = photographs[activeIndex];
  const previous = photographs[activeIndex - 1];
  const next = photographs[activeIndex + 1];
  const backButton = useRef<HTMLButtonElement>(null);
  const filmstrip = useRef<HTMLUListElement>(null);
  const activeThumbnail = useRef<HTMLButtonElement>(null);
  const capture = usePhotographCapture(
    loadCapture,
    shootName,
    active.relativePath,
    active.originalUrl,
  );
  const [originalLoads, setOriginalLoads] = useState(() => new Map<string, OriginalStatus>());
  const [timing, setTiming] = useState<{ openedAt: number; milliseconds: number }>();
  const activeStatus = active.originalUrl === null
    ? "unavailable"
    : originalLoads.get(active.relativePath);
  const sharp = activeStatus === "ready";
  const neighbors = direction === 1 ? [next, previous] : [previous, next];
  // Keep decoded neighbors mounted. Start at most one nearby preload after the stage settles.
  const preload = activeStatus === undefined ? undefined : neighbors.find(
    (photo) => photo?.originalUrl && !originalLoads.has(photo.relativePath),
  );
  const originalImages = [
    active,
    ...neighbors.filter((photo) => photo && (
      originalLoads.get(photo.relativePath) === "ready" || photo === preload
    )),
  ];

  useLayoutEffect(() => {
    backButton.current?.focus({ preventScroll: true });
  }, []);

  useLayoutEffect(() => {
    const button = activeThumbnail.current;
    if (button) {
      if (filmstrip.current?.contains(document.activeElement)) {
        button.focus({ preventScroll: true });
      }
      button.scrollIntoView({ block: "nearest", inline: "nearest" });
    }
  }, [activeIndex]);

  useLayoutEffect(() => {
    if (openedAt !== null && sharp) {
      let afterPaint: number | undefined;
      // The first frame can paint the revealed original. Publish the time on the following frame.
      const beforePaint = requestAnimationFrame(() => {
        const milliseconds = performance.now() - openedAt;
        afterPaint = requestAnimationFrame(() => setTiming({ openedAt, milliseconds }));
      });
      return () => {
        cancelAnimationFrame(beforePaint);
        if (afterPaint !== undefined) cancelAnimationFrame(afterPaint);
      };
    }
  }, [openedAt, sharp]);

  function openPhotograph(index: number) {
    const photograph = photographs[index];
    if (photograph && index !== activeIndex) {
      const neighbors = photographs.slice(Math.max(0, index - 1), index + 2);
      setOriginalLoads((current) => {
        const retained = new Map<string, OriginalStatus>();
        for (const neighbor of neighbors) {
          const status = current.get(neighbor.relativePath);
          if (status && (status === "ready" || neighbor !== photograph)) {
            retained.set(neighbor.relativePath, status);
          }
        }
        return retained;
      });
      onOpenPhotograph(photograph.relativePath, index < activeIndex ? -1 : 1);
    }
  }

  function updateOriginal(image: HTMLImageElement, path: string, status: OriginalStatus) {
    if (image.isConnected) {
      setOriginalLoads((current) => new Map(current).set(path, status));
    }
  }

  function decodeOriginal(event: SyntheticEvent<HTMLImageElement>, path: string) {
    const image = event.currentTarget;
    void image.decode().then(
      () => updateOriginal(image, path, "ready"),
      () => updateOriginal(image, path, "unavailable"),
    );
  }

  function handleKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (!event.defaultPrevented && !event.altKey && !event.ctrlKey && !event.metaKey && !event.shiftKey) {
      switch (event.key) {
        case "ArrowLeft":
          event.preventDefault();
          openPhotograph(activeIndex - 1);
          break;
        case "ArrowRight":
          event.preventDefault();
          openPhotograph(activeIndex + 1);
          break;
        case "Escape":
          event.preventDefault();
          onClose();
          break;
      }
    }
  }

  const rows = [
    ["Dimensions", `${active.width} × ${active.height} px`],
    ["Captured", capture?.capturedAt],
    ["Camera", capture?.camera],
    ["Lens", capture?.lens],
    ["Aperture", capture?.aperture],
    ["Shutter speed", capture?.shutterSpeed],
    ["ISO", capture?.iso],
  ];

  return (
    <section
      aria-label="Photograph"
      tabIndex={-1}
      className="absolute inset-0 flex min-h-0 min-w-0 flex-col bg-background text-foreground outline-none"
      onKeyDown={handleKeyDown}
    >
      <div className="flex shrink-0 flex-wrap items-center justify-between gap-x-3 gap-y-1 border-b border-border px-3 py-2">
        <Button ref={backButton} variant="ghost" onClick={onClose} aria-keyshortcuts="Escape">
          Back to grid <kbd aria-hidden="true" className="font-mono text-xs text-muted-foreground">Esc</kbd>
        </Button>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="icon"
            disabled={!previous}
            focusableWhenDisabled
            aria-label="Previous photograph"
            aria-keyshortcuts="ArrowLeft"
            title="Previous photograph (←)"
            onClick={() => openPhotograph(activeIndex - 1)}
          >
            <CaretLeft aria-hidden="true" />
          </Button>
          <p role="status" className="px-1 text-center font-mono text-xs whitespace-nowrap text-muted-foreground">
            <span className="sr-only">{active.fileName}, </span>{activeIndex + 1} / {photographs.length}
          </p>
          <Button
            variant="ghost"
            size="icon"
            disabled={!next}
            focusableWhenDisabled
            aria-label="Next photograph"
            aria-keyshortcuts="ArrowRight"
            title="Next photograph (→)"
            onClick={() => openPhotograph(activeIndex + 1)}
          >
            <CaretRight aria-hidden="true" />
          </Button>
        </div>
      </div>

      <div className="flex min-h-0 flex-1">
        <div className="min-h-0 min-w-0 flex-1 bg-stage p-3">
          <div className="relative size-full">
            <img
              key={`thumbnail:${active.relativePath}`}
              src={active.thumbnailUrl}
              alt={sharp ? "" : active.fileName}
              className={`block size-full object-contain${sharp ? " invisible" : ""}`}
            />
            {originalImages.map((photo) => {
              const isActive = photo === active;
              const visible = isActive && sharp;
              return photo?.originalUrl ? (
                <img
                  key={photo.relativePath}
                  ref={cancelRemovedImage}
                  alt={visible ? photo.fileName : ""}
                  fetchPriority={isActive ? "high" : "low"}
                  src={photo.originalUrl}
                  onLoad={(event) => decodeOriginal(event, photo.relativePath)}
                  onError={(event) => updateOriginal(event.currentTarget, photo.relativePath, "unavailable")}
                  className={`absolute inset-0 size-full object-contain${visible ? "" : " invisible"}`}
                />
              ) : null;
            })}
          </div>
        </div>
        <aside aria-label="Photograph details" className="w-[36%] max-w-64 shrink-0 overflow-y-auto border-l border-border p-3">
          <h2 className="font-mono text-sm font-medium wrap-anywhere">{active.fileName}</h2>
          <dl className="mt-3">
            {rows.map(([label, value]) => value != null ? (
              <div key={label} className="border-t border-border py-2 text-xs">
                <dt className="text-muted-foreground">{label}</dt>
                <dd className="mt-0.5 font-mono wrap-anywhere">{value}</dd>
              </div>
            ) : null)}
            {openedAt !== null ? (
              <div className="border-t border-border py-2 text-xs">
                <dt className="text-muted-foreground">Time to sharp</dt>
                <dd className="mt-0.5 font-mono">
                  {activeStatus === "unavailable" ? "Unavailable" : timing?.openedAt === openedAt
                    ? `${Math.round(timing.milliseconds)} ms`
                    : "Loading…"}
                </dd>
              </div>
            ) : null}
          </dl>
          {activeStatus === "unavailable" ? (
            <p role="status" className="border-t border-border pt-2 text-xs text-muted-foreground">
              Original unavailable.
            </p>
          ) : null}
        </aside>
      </div>

      <ul ref={filmstrip} aria-label="Filmstrip" role="list" className="flex shrink-0 gap-2 overflow-x-auto border-t border-border px-3 py-2">
        {photographs.map((photograph, index) => {
          const isActive = index === activeIndex;
          return (
            <li key={photograph.relativePath} className="shrink-0">
              <button
                ref={isActive ? activeThumbnail : undefined}
                type="button"
                aria-current={isActive ? "true" : undefined}
                tabIndex={isActive ? 0 : -1}
                title={photograph.fileName}
                onClick={() => openPhotograph(index)}
                className="block h-16 w-20 cursor-pointer border border-transparent bg-stage p-1 aria-[current=true]:border-foreground focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-ring"
              >
                <img
                  src={photograph.thumbnailUrl}
                  alt={photograph.fileName}
                  loading="lazy"
                  className="block size-full object-contain"
                />
              </button>
            </li>
          );
        })}
      </ul>
    </section>
  );
}
