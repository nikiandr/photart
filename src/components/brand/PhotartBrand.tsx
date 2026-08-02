import { PhotartMark } from "@/components/brand/PhotartMark";
import { cn } from "@/lib/utils";

export type PhotartBrandProps = {
  className?: string;
  /** Secondary line under the wordmark, e.g. the Catalog name. */
  detail?: string;
  /** inline: mark beside wordmark (sidebar). stack: mark above wordmark (setup). */
  layout?: "inline" | "stack";
  markClassName?: string;
};

export function PhotartBrand({
  className,
  detail,
  layout = "inline",
  markClassName,
}: PhotartBrandProps) {
  const stacked = layout === "stack";

  return (
    <div
      className={cn(
        "flex min-w-0",
        stacked
          ? "flex-col items-center gap-3 text-center"
          : "items-center gap-2.5",
        className,
      )}
    >
      <PhotartMark
        className={cn(stacked ? "size-10" : "size-8", markClassName)}
      />
      <div className={cn("min-w-0", stacked && "space-y-0.5")}>
        <p
          className={cn(
            "font-sans font-semibold tracking-tight text-foreground",
            stacked ? "text-sm" : "text-xs",
          )}
        >
          Photart
        </p>
        {detail ? (
          <p className="truncate text-xs text-muted-foreground">{detail}</p>
        ) : null}
      </div>
    </div>
  );
}
