import { FolderOpen } from "@phosphor-icons/react/dist/csr/FolderOpen";
import { SpinnerGap } from "@phosphor-icons/react/dist/csr/SpinnerGap";

import { PhotartBrand } from "@/components/brand/PhotartBrand";
import { Button } from "@/components/ui/button";
import { homePathDisplay } from "@/lib/catalog";

export type CatalogLocationStepProps = {
  catalogName: string;
  parentDirectory: string | null;
  picturesDirectory: string | null;
  error: string | null;
  creating: boolean;
  onChooseDirectory: () => void;
  onBack: () => void;
  onCreate: () => void;
};

export function CatalogLocationStep({
  catalogName,
  parentDirectory,
  picturesDirectory,
  error,
  creating,
  onChooseDirectory,
  onBack,
  onCreate,
}: CatalogLocationStepProps) {
  const preview =
    parentDirectory != null
      ? homePathDisplay(parentDirectory, catalogName, picturesDirectory)
      : null;

  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-6">
      <div className="flex w-full max-w-md flex-col gap-6">
        <div className="space-y-4 text-center">
          <PhotartBrand layout="stack" />
          <div className="space-y-2">
            <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
              Choose a location
            </h1>
            <p className="text-sm/relaxed text-muted-foreground">
              Photart will create a folder for your Catalog here.
            </p>
          </div>
        </div>

        <div className="space-y-3">
          <Button
            type="button"
            variant="outline"
            className="w-full justify-start"
            onClick={onChooseDirectory}
            disabled={creating}
          >
            <FolderOpen data-icon="inline-start" aria-hidden="true" />
            {parentDirectory ? "Change folder" : "Choose folder"}
          </Button>

          {preview ? (
            <p className="break-all font-mono text-[0.8125rem] text-foreground">
              {preview}
            </p>
          ) : (
            <p className="font-mono text-[0.8125rem] text-muted-foreground">
              No folder selected
            </p>
          )}

          {error ? (
            <p className="font-sans text-xs text-destructive" role="alert">
              {error}
            </p>
          ) : null}
        </div>

        <div className="flex items-center justify-between gap-3">
          <Button
            type="button"
            variant="ghost"
            onClick={onBack}
            disabled={creating}
          >
            Back
          </Button>
          <Button
            type="button"
            onClick={onCreate}
            disabled={creating || parentDirectory == null}
          >
            {creating ? (
              <SpinnerGap
                data-icon="inline-start"
                className="animate-spin motion-reduce:animate-none"
                aria-hidden="true"
              />
            ) : null}
            {creating ? "Creating…" : "Create Catalog"}
          </Button>
        </div>
      </div>
    </div>
  );
}
