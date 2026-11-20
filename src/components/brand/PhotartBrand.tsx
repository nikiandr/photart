import { PhotartMark } from "@/components/brand/PhotartMark";
import { editionName, isDevEdition } from "@/lib/edition";
import { cn } from "@/lib/utils";

export type PhotartBrandProps = {
  className?: string;
  /** Secondary line under the wordmark, e.g. the Catalog name. */
  detail?: string;
  /** Storybook override. The running app reads the build channel. */
  dev?: boolean;
  /** inline: mark beside wordmark (sidebar). stack: mark above wordmark (setup). */
  layout?: "inline" | "stack";
  markClassName?: string;
};

export function PhotartBrand({
  className,
  detail,
  dev,
  layout = "inline",
  markClassName,
}: PhotartBrandProps) {
  const stacked = layout === "stack";
  const showDev = dev ?? isDevEdition();

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
        appearance={stacked ? "icon" : "sidebar"}
        className={cn(stacked ? "size-10" : "size-8", markClassName)}
        title={editionName(showDev)}
      />
      <div className={cn("min-w-0", stacked && "space-y-0.5")}>
        <p
          className={cn(
            "font-sans font-semibold tracking-tight text-foreground",
            stacked ? "text-sm" : "text-xs",
          )}
        >
          Photart
          {showDev ? (
            <span className="font-mono text-xs font-normal text-muted-foreground">
              {" "}
              Dev
            </span>
          ) : null}
        </p>
        {detail ? (
          <p className="truncate text-xs text-muted-foreground">{detail}</p>
        ) : null}
      </div>
    </div>
  );
}
