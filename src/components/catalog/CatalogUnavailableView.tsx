import { WarningCircle } from "@phosphor-icons/react/dist/csr/WarningCircle";

import { Button } from "@/components/ui/button";
import { shortenAbsolutePath } from "@/lib/catalog";

export type CatalogUnavailableViewProps = {
  name: string;
  path: string;
  reason: string;
  onSetupNew: () => void;
};

export function CatalogUnavailableView({
  name,
  path,
  reason,
  onSetupNew,
}: CatalogUnavailableViewProps) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-6">
      <div className="flex w-full max-w-lg flex-col items-center gap-6 text-center">
        <div className="space-y-3">
          <p className="font-sans text-xs font-medium tracking-wide text-muted-foreground lowercase">
            photart
          </p>
          <div className="flex flex-col items-center gap-3">
            <WarningCircle
              className="size-8 text-muted-foreground"
              weight="duotone"
              aria-hidden="true"
            />
            <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
              Catalog unavailable
            </h1>
          </div>
          <p className="text-sm/relaxed text-muted-foreground">
            Photart couldn&apos;t open{" "}
            <span className="text-foreground">{name}</span>.
          </p>
          <p className="break-all font-mono text-[0.8125rem] text-foreground">
            {shortenAbsolutePath(path)}
          </p>
          <p className="text-sm/relaxed text-muted-foreground">{reason}</p>
        </div>

        <Button type="button" onClick={onSetupNew}>
          Set Up a New Catalog
        </Button>
      </div>
    </div>
  );
}
