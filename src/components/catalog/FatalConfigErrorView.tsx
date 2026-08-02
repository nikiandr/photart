import { PhotartBrand } from "@/components/brand/PhotartBrand";
import type { CatalogError } from "@/types";

export type FatalConfigErrorViewProps = {
  error: CatalogError;
};

export function FatalConfigErrorView({ error }: FatalConfigErrorViewProps) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-6">
      <div className="flex w-full max-w-md flex-col items-center gap-4 text-center">
        <PhotartBrand layout="stack" />
        <h1 className="font-editorial text-2xl font-medium tracking-tight text-foreground">
          Photart can&apos;t start
        </h1>
        <p className="text-sm/relaxed text-muted-foreground">{error.message}</p>
        {error.code ? (
          <p className="font-mono text-xs text-muted-foreground">{error.code}</p>
        ) : null}
      </div>
    </div>
  );
}
