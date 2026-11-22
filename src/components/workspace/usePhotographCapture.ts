import { useEffect, useRef, useState } from "react";

import type { loadPhotographCapture } from "@/lib/catalog";
import type { PhotographCapture } from "@/types";

const MAX_CACHED_CAPTURES = 5;

export function usePhotographCapture(
  loadCapture: typeof loadPhotographCapture,
  shootName: string,
  relativePath: string,
  originalAvailable: boolean,
) {
  const cache = useRef(new Map<string, PhotographCapture>());
  const [result, setResult] = useState<{
    path: string;
    capture: PhotographCapture;
  }>();

  useEffect(() => {
    let cancelled = false;
    const captures = cache.current;

    if (originalAvailable) {
      const cached = captures.get(relativePath);
      if (cached) {
        captures.delete(relativePath);
        captures.set(relativePath, cached);
        setResult({ path: relativePath, capture: cached });
      } else {
        void loadCapture(shootName, relativePath).then((capture) => {
          if (!cancelled) {
            captures.set(relativePath, capture);
            const oldest = captures.keys().next().value;
            if (captures.size > MAX_CACHED_CAPTURES && oldest !== undefined) {
              captures.delete(oldest);
            }
            setResult({ path: relativePath, capture });
          }
        }).catch(() => {
          // Capture details are optional, including when a file becomes unavailable.
        });
      }
    }

    return () => {
      cancelled = true;
    };
  }, [loadCapture, shootName, relativePath, originalAvailable]);

  return originalAvailable && result?.path === relativePath ? result.capture : undefined;
}
