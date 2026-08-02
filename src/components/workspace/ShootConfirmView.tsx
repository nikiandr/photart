import { useRef, useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { validateName } from "@/lib/names";
import { cn } from "@/lib/utils";

export type ShootConfirmViewProps = {
  shootName: string;
  sourcePath: string;
  error: string | null;
  onShootNameChange: (value: string) => void;
  onCancel: () => void;
  onImport: () => void;
};

export function ShootConfirmView({
  shootName,
  sourcePath,
  error,
  onShootNameChange,
  onCancel,
  onImport,
}: ShootConfirmViewProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const [nameError, setNameError] = useState<string | null>(null);

  function handleSubmit(event: FormEvent) {
    event.preventDefault();
    const validationError = validateName(shootName);
    if (validationError) {
      setNameError(validationError);
      inputRef.current?.focus();
      return;
    }
    setNameError(null);
    onImport();
  }

  const inlineError = nameError ?? error;

  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-8">
      <form
        className="flex w-full max-w-md flex-col gap-6"
        onSubmit={handleSubmit}
        noValidate
      >
        <div className="space-y-2 text-center">
          <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
            Create a Shoot
          </h1>
          <p className="text-sm/relaxed text-muted-foreground">
            Photographs will be copied into your Catalog.
          </p>
        </div>

        <div className="space-y-4">
          <div className="space-y-2">
            <label
              htmlFor="shoot-name"
              className="block font-sans text-[0.8125rem] font-medium text-foreground"
            >
              Shoot name
            </label>
            <input
              ref={inputRef}
              id="shoot-name"
              type="text"
              value={shootName}
              onChange={(event) => {
                onShootNameChange(event.target.value);
                if (nameError) {
                  setNameError(null);
                }
              }}
              aria-invalid={nameError ? true : undefined}
              autoFocus
              className={cn(
                "h-8 w-full border border-input bg-background px-2.5 font-sans text-[0.8125rem] text-foreground outline-none",
                "focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                "aria-invalid:border-destructive aria-invalid:ring-1 aria-invalid:ring-destructive/20",
              )}
            />
          </div>

          <div className="space-y-2">
            <p className="font-sans text-[0.8125rem] font-medium text-foreground">
              Source
            </p>
            <p className="break-all font-mono text-[0.8125rem] text-muted-foreground">
              {sourcePath}
            </p>
          </div>

          {inlineError ? (
            <p className="font-sans text-xs text-destructive" role="alert">
              {inlineError}
            </p>
          ) : null}
        </div>

        <div className="flex items-center justify-between gap-3">
          <Button type="button" variant="ghost" onClick={onCancel}>
            Cancel
          </Button>
          <Button type="submit">Import photographs</Button>
        </div>
      </form>
    </div>
  );
}
