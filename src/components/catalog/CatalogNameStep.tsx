import { useRef, useState, type FormEvent } from "react";

import { PhotartBrand } from "@/components/brand/PhotartBrand";
import { Button } from "@/components/ui/button";
import { validateName } from "@/lib/names";
import { cn } from "@/lib/utils";

export type CatalogNameStepProps = {
  catalogName: string;
  onCatalogNameChange: (value: string) => void;
  onContinue: () => void;
};

export function CatalogNameStep({
  catalogName,
  onCatalogNameChange,
  onContinue,
}: CatalogNameStepProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [error, setError] = useState<string | null>(null);

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const validationError = validateName(catalogName);
    if (validationError) {
      setError(validationError);
      inputRef.current?.focus();
      return;
    }
    setError(null);
    onContinue();
  }

  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-6">
      <form
        className="flex w-full max-w-md flex-col gap-6"
        onSubmit={handleSubmit}
        noValidate
      >
        <div className="space-y-4 text-center">
          <PhotartBrand layout="stack" />
          <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
            Name your Catalog
          </h1>
        </div>

        <div className="space-y-2">
          <label
            htmlFor="catalog-name"
            className="block font-sans text-[0.8125rem] font-medium text-foreground"
          >
            Catalog name
          </label>
          <input
            ref={inputRef}
            id="catalog-name"
            type="text"
            value={catalogName}
            onChange={(event) => {
              onCatalogNameChange(event.target.value);
              if (error) {
                setError(null);
              }
            }}
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? "catalog-name-error" : undefined}
            autoFocus
            className={cn(
              "h-8 w-full border border-input bg-background px-2.5 font-sans text-[0.8125rem] text-foreground outline-none",
              "focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
              "aria-invalid:border-destructive aria-invalid:ring-1 aria-invalid:ring-destructive/20",
            )}
          />
          {error ? (
            <p
              id="catalog-name-error"
              className="font-sans text-xs text-destructive"
              role="alert"
            >
              {error}
            </p>
          ) : null}
        </div>

        <div className="flex justify-end">
          <Button type="submit">Continue</Button>
        </div>
      </form>
    </div>
  );
}
