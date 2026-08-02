import type { PhotographEntry } from "@/types";

export type ThumbnailGridProps = {
  photographs: PhotographEntry[];
};

export function ThumbnailGrid({ photographs }: ThumbnailGridProps) {
  if (photographs.length === 0) {
    return (
      <p className="py-8 text-center text-sm text-muted-foreground">
        No photographs in this Shoot.
      </p>
    );
  }

  return (
    <ul className="grid grid-cols-[repeat(auto-fill,minmax(10rem,1fr))] gap-4">
      {photographs.map((photograph) => (
        <li key={photograph.relativePath} className="min-w-0">
          <figure className="flex flex-col gap-1.5">
            <img
              src={photograph.thumbnailUrl}
              alt={photograph.fileName}
              width={photograph.width}
              height={photograph.height}
              loading="lazy"
              decoding="async"
              className="h-auto w-full bg-muted"
            />
            <figcaption className="truncate font-mono text-[0.6875rem] text-muted-foreground">
              {photograph.fileName}
            </figcaption>
          </figure>
        </li>
      ))}
    </ul>
  );
}
