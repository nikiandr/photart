import { Button } from "@/components/ui/button";

export type HomeViewProps = {
  onCreateShoot: () => void;
};

export function HomeView({ onCreateShoot }: HomeViewProps) {
  return (
    <div className="flex h-full w-full flex-col items-center justify-center px-8">
      <div className="flex max-w-md flex-col items-center gap-5 text-center">
        <div className="space-y-2">
          <h1 className="font-editorial text-3xl font-medium tracking-tight text-foreground">
            Start a new Shoot
          </h1>
          <p className="text-sm/relaxed text-muted-foreground">
            Import photographs into your Catalog when you&apos;re ready.
          </p>
        </div>
        <Button type="button" onClick={onCreateShoot}>
          Create a Shoot
        </Button>
      </div>
    </div>
  );
}
