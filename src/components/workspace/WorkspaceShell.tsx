import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

export type WorkspaceShellProps = {
  sidebar: ReactNode;
  children: ReactNode;
  className?: string;
};

export function WorkspaceShell({
  sidebar,
  children,
  className,
}: WorkspaceShellProps) {
  return (
    <div
      className={cn(
        "flex h-svh w-full overflow-hidden bg-background text-foreground",
        className,
      )}
    >
      <aside className="flex w-60 shrink-0 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
        {sidebar}
      </aside>
      <main className="min-w-0 flex-1 overflow-hidden">{children}</main>
    </div>
  );
}
