import { useRef, useState, type FormEvent } from "react";

import { Button } from "@/components/ui/button";
import { validateName } from "@/lib/names";
import { cn } from "@/lib/utils";
import type { ShootMedium, SourceMediumHint } from "@/types";
import { ShootMediumFields } from "./ShootMediumFields";

const hintText = {
  checking: "Checking Source…",
  noritsu: "Film detected",
  possibleFilmRoll: "Possible film roll",
  noneFound: "",
  unavailable: "Source check unavailable",
};

export type ShootConfirmViewProps = {
  shootName: string;
  sourcePath: string | null;
  error: string | null;
  medium: ShootMedium | null;
  sourceHint: SourceMediumHint | "checking" | "unavailable";
  onShootNameChange: (value: string) => void;
  onChooseSource: () => void;
  onMediumChange: (medium: ShootMedium | null) => void;
  onCancel: () => void;
  onImport: () => void;
};

export function ShootConfirmView({
  shootName,
  sourcePath,
  error,
  medium,
  sourceHint,
  onShootNameChange,
  onChooseSource,
  onMediumChange,
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
    if (sourcePath) {
      onImport();
    }
  }

  const inlineError = nameError ?? error;

  return (
    <div className="flex h-full w-full flex-col items-center overflow-y-auto px-8 py-6">
      <form
        className="my-auto flex w-full max-w-md flex-col gap-6"
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
            <div className="flex items-center justify-between gap-3">
              <p className="font-sans text-[0.8125rem] font-medium text-foreground">
                Source
              </p>
              <Button type="button" variant="outline" size="sm" onClick={onChooseSource} autoFocus={!sourcePath}>
                {sourcePath ? "Change folder…" : "Choose folder…"}
              </Button>
            </div>
            <p className="break-all font-mono text-[0.8125rem] text-muted-foreground">
              {sourcePath ?? "No folder selected"}
            </p>
          </div>

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
              className={cn(
                "h-8 w-full border border-input bg-background px-2.5 font-sans text-[0.8125rem] text-foreground outline-none",
                "focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                "aria-invalid:border-destructive aria-invalid:ring-1 aria-invalid:ring-destructive/20",
              )}
            />
          </div>

          <ShootMediumFields
            medium={medium}
            onChange={onMediumChange}
            status={hintText[sourceHint]}
          />

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
          <Button type="submit" disabled={!sourcePath}>Import photographs</Button>
        </div>
      </form>
    </div>
  );
}
