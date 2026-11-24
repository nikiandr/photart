import { useState, type FormEvent } from "react";
import { Camera } from "@phosphor-icons/react/dist/csr/Camera";
import { House } from "@phosphor-icons/react/dist/csr/House";
import { PencilSimple } from "@phosphor-icons/react/dist/csr/PencilSimple";
import { Plus } from "@phosphor-icons/react/dist/csr/Plus";
import { Trash } from "@phosphor-icons/react/dist/csr/Trash";

import { PhotartBrand } from "@/components/brand/PhotartBrand";
import { Button } from "@/components/ui/button";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { validateName } from "@/lib/names";
import { cn } from "@/lib/utils";
import type { CatalogError, ShootMedium, ShootSummary } from "@/types";
import { ShootMediumFields } from "./ShootMediumFields";

export type SidebarActive = "home" | { shootName: string };

export type SidebarProps = {
  catalogName: string;
  shoots: ShootSummary[];
  active: SidebarActive | null;
  disabled?: boolean;
  onHome: () => void;
  onCreateShoot: () => void;
  onSelectShoot: (name: string) => void;
  onUpdateShoot: (name: string, newName: string, medium: ShootMedium | null) => Promise<void>;
  onPurgeShoot: (name: string) => Promise<void>;
};

type ShootAction =
  | { kind: "edit"; shoot: ShootSummary }
  | { kind: "purge"; shoot: ShootSummary };

export function Sidebar({
  catalogName,
  shoots,
  active,
  disabled = false,
  onHome,
  onCreateShoot,
  onSelectShoot,
  onUpdateShoot,
  onPurgeShoot,
}: SidebarProps) {
  const homeActive = active === "home";
  const [action, setAction] = useState<ShootAction | null>(null);
  const [nameDraft, setNameDraft] = useState("");
  const [mediumDraft, setMediumDraft] = useState<ShootMedium | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [working, setWorking] = useState(false);

  function beginAction(nextAction: ShootAction) {
    setAction(nextAction);
    setNameDraft(nextAction.shoot.name);
    setMediumDraft(nextAction.shoot.medium);
    setActionError(null);
  }

  function closeAction() {
    if (!working) {
      setAction(null);
      setActionError(null);
    }
  }

  async function submitEdit(event: FormEvent) {
    event.preventDefault();
    if (action?.kind === "edit") {
      const validationError = validateName(nameDraft);
      if (validationError) {
        setActionError(validationError);
      } else if (nameDraft.trim() === action.shoot.name && mediumDraft === action.shoot.medium) {
        closeAction();
      } else {
        setWorking(true);
        setActionError(null);
        try {
          await onUpdateShoot(action.shoot.name, nameDraft.trim(), mediumDraft);
          setAction(null);
        } catch (error) {
          setActionError((error as CatalogError).message);
        } finally {
          setWorking(false);
        }
      }
    }
  }

  async function confirmPurge() {
    if (action?.kind === "purge") {
      setWorking(true);
      setActionError(null);
      try {
        await onPurgeShoot(action.shoot.name);
        setAction(null);
      } catch (error) {
        setActionError((error as CatalogError).message);
      } finally {
        setWorking(false);
      }
    }
  }

  return (
    <div
      className={cn(
        "flex h-full min-h-0 flex-col",
        disabled && "pointer-events-none opacity-60",
      )}
      aria-disabled={disabled || undefined}
    >
      <div className="flex h-14 shrink-0 items-center border-b border-sidebar-border px-3">
        <PhotartBrand detail={catalogName} />
      </div>

      <div className="shrink-0 px-2 pt-2">
        <button
          type="button"
          onClick={onHome}
          disabled={disabled}
          aria-current={homeActive ? "page" : undefined}
          className={cn(
            "flex h-8 w-full items-center gap-2 px-2 font-sans text-[0.8125rem] font-medium outline-none transition-colors motion-reduce:transition-none",
            "focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-sidebar-ring",
            homeActive
              ? "bg-sidebar-accent text-sidebar-accent-foreground"
              : "text-sidebar-foreground hover:bg-sidebar-accent/70",
          )}
        >
          <House className="size-4 shrink-0" aria-hidden="true" />
          Home
        </button>
      </div>

      <div className="mt-3 flex min-h-0 flex-1 flex-col">
        <div className="flex shrink-0 items-center justify-between gap-2 px-4 pb-1">
          <div className="flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <Camera className="size-3.5" aria-hidden="true" />
            Shoots
          </div>
          <Button
            type="button"
            variant="ghost"
            size="icon-xs"
            aria-label="Create a Shoot"
            onClick={onCreateShoot}
            disabled={disabled}
          >
            <Plus aria-hidden="true" />
          </Button>
        </div>

        <nav
          className="min-h-0 flex-1 overflow-y-auto px-2 pb-3"
          aria-label="Shoots"
        >
          {shoots.length === 0 ? (
            <p className="px-2 py-2 text-xs text-muted-foreground">
              No shoots yet
            </p>
          ) : (
            <ul className="flex flex-col gap-0.5">
              {shoots.map((shoot) => {
                const isActive =
                  active !== null &&
                  active !== "home" &&
                  active.shootName === shoot.name;

                return (
                  <li key={shoot.name}>
                    <ContextMenu disabled={disabled}>
                      <ContextMenuTrigger>
                        <button
                          type="button"
                          onClick={() => onSelectShoot(shoot.name)}
                          disabled={disabled}
                          aria-current={isActive ? "page" : undefined}
                          className={cn(
                            "flex h-8 w-full min-w-0 items-center gap-2 px-2 text-left outline-none transition-colors motion-reduce:transition-none",
                            "focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-sidebar-ring",
                            isActive
                              ? "bg-sidebar-accent text-sidebar-accent-foreground"
                              : "text-sidebar-foreground hover:bg-sidebar-accent/70",
                          )}
                        >
                          <span className="min-w-0 flex-1 truncate font-sans text-[0.8125rem] font-medium">
                            {shoot.name}
                          </span>
                          <span className="shrink-0 font-mono text-[0.6875rem] text-muted-foreground">
                            {shoot.photographCount}
                          </span>
                        </button>
                      </ContextMenuTrigger>
                      <ContextMenuContent>
                        <ContextMenuItem
                          onClick={() =>
                            beginAction({ kind: "edit", shoot })
                          }
                        >
                          <PencilSimple aria-hidden="true" />
                          Edit Shoot…
                        </ContextMenuItem>
                        <ContextMenuSeparator />
                        <ContextMenuItem
                          variant="destructive"
                          onClick={() =>
                            beginAction({ kind: "purge", shoot })
                          }
                        >
                          <Trash aria-hidden="true" />
                          Purge Shoot…
                        </ContextMenuItem>
                      </ContextMenuContent>
                    </ContextMenu>
                  </li>
                );
              })}
            </ul>
          )}
        </nav>
      </div>

      <Dialog open={action !== null} onOpenChange={(open) => !open && closeAction()}>
        <DialogContent showCloseButton={false}>
          {action?.kind === "edit" ? (
            <form className="contents" onSubmit={submitEdit} noValidate>
              <DialogHeader>
                <DialogTitle>Edit Shoot</DialogTitle>
                <DialogDescription>
                  Change the name or medium of this Shoot.
                </DialogDescription>
              </DialogHeader>
              <div className="space-y-2">
                <label
                  htmlFor="edit-shoot-name"
                  className="block font-sans text-[0.8125rem] font-medium text-foreground"
                >
                  Shoot name
                </label>
                <input
                  id="edit-shoot-name"
                  value={nameDraft}
                  onChange={(event) => {
                    setNameDraft(event.target.value);
                    setActionError(null);
                  }}
                  disabled={working}
                  autoFocus
                  aria-invalid={actionError ? true : undefined}
                  aria-describedby={actionError ? "shoot-action-error" : undefined}
                  className={cn(
                    "h-8 w-full border border-input bg-background px-2.5 font-sans text-[0.8125rem] text-foreground outline-none",
                    "focus-visible:border-ring focus-visible:ring-1 focus-visible:ring-ring/50",
                    "aria-invalid:border-destructive aria-invalid:ring-1 aria-invalid:ring-destructive/20",
                  )}
                />
                {actionError ? (
                  <p
                    id="shoot-action-error"
                    className="font-sans text-xs text-destructive"
                    role="alert"
                  >
                    {actionError}
                  </p>
                ) : null}
              </div>
              <ShootMediumFields
                medium={mediumDraft}
                onChange={setMediumDraft}
                disabled={working}
              />
              <DialogFooter>
                <DialogClose
                  render={<Button variant="outline" />}
                  disabled={working}
                >
                  Cancel
                </DialogClose>
                <Button type="submit" disabled={working}>
                  {working ? "Applying…" : "Apply changes"}
                </Button>
              </DialogFooter>
            </form>
          ) : null}

          {action?.kind === "purge" ? (
            <>
              <DialogHeader>
                <DialogTitle>Purge “{action.shoot.name}”?</DialogTitle>
                <DialogDescription>
                  This permanently removes {action.shoot.photographCount}{" "}
                  {action.shoot.photographCount === 1
                    ? "photograph"
                    : "photographs"}{" "}
                  and its cached thumbnails from the Catalog. This cannot be
                  undone.
                </DialogDescription>
              </DialogHeader>
              {actionError ? (
                <p
                  id="shoot-action-error"
                  className="border-l-2 border-l-destructive pl-3 font-sans text-xs text-destructive"
                  role="alert"
                >
                  {actionError}
                </p>
              ) : null}
              <DialogFooter>
                <DialogClose
                  render={<Button variant="outline" />}
                  disabled={working}
                >
                  Cancel
                </DialogClose>
                <Button
                  type="button"
                  variant="destructive"
                  onClick={() => void confirmPurge()}
                  disabled={working}
                >
                  {working ? "Purging…" : "Purge Shoot"}
                </Button>
              </DialogFooter>
            </>
          ) : null}
        </DialogContent>
      </Dialog>
    </div>
  );
}
