import { useId } from "react";

import { cn } from "@/lib/utils";
import type { ShootMedium } from "@/types";

type ShootMediumFieldsProps = {
  medium: ShootMedium | null;
  onChange: (medium: ShootMedium | null) => void;
  disabled?: boolean;
  status?: string;
};

export function ShootMediumFields({ medium, onChange, disabled, status }: ShootMediumFieldsProps) {
  const id = useId();

  return (
    <fieldset className="min-w-0 space-y-2" disabled={disabled}>
      <legend className="font-sans text-[0.8125rem] font-medium text-foreground">
        Medium <span className="font-normal text-muted-foreground">(optional)</span>
        <span role="status" className="text-xs font-medium text-muted-foreground">
          {status ? <span className="ml-2 border-l border-border pl-2">{status}</span> : null}
        </span>
      </legend>
      <div className="flex flex-wrap items-center gap-x-3 gap-y-2">
        <div className="inline-flex divide-x divide-input border border-input">
          {(["film", "digital"] as const).map((kind) => (
            <label key={kind} className="relative">
              <input
                type="radio"
                name={`${id}-medium`}
                value={kind}
                checked={medium?.kind === kind}
                onClick={() => {
                  if (status && medium?.kind === kind) {
                    onChange(medium);
                  }
                }}
                onChange={() => onChange(kind === "film" ? { kind, filmStock: null } : { kind })}
                className="peer sr-only"
              />
              <span className={cn(
                "flex h-8 cursor-pointer items-center px-3 font-sans text-[0.8125rem] text-muted-foreground",
                "hover:text-foreground peer-checked:bg-secondary peer-checked:font-medium peer-checked:text-secondary-foreground",
                "peer-focus-visible:outline-2 peer-focus-visible:-outline-offset-2 peer-focus-visible:outline-ring peer-disabled:cursor-default",
              )}>
                {kind === "film" ? "Film" : "Digital"}
              </span>
            </label>
          ))}
        </div>
        {medium ? (
          <button
            type="button"
            onClick={() => onChange(null)}
            className="text-xs text-muted-foreground underline underline-offset-2 hover:text-foreground focus-visible:outline-2 focus-visible:outline-ring"
          >
            Clear medium
          </button>
        ) : null}
      </div>
      {medium?.kind === "film" ? (
        <div className="space-y-2 pt-2">
          <label htmlFor={`${id}-stock`} className="block font-sans text-[0.8125rem] font-medium text-foreground">
            Film stock <span className="font-normal text-muted-foreground">(optional)</span>
          </label>
          <input
            id={`${id}-stock`}
            type="text"
            value={medium.filmStock ?? ""}
            onChange={(event) => onChange({ kind: "film", filmStock: event.target.value })}
            placeholder="e.g. Kodak Gold 200"
            className="h-8 w-full border border-input bg-background px-2.5 font-sans text-[0.8125rem] text-foreground outline-none focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50"
          />
        </div>
      ) : null}
    </fieldset>
  );
}
