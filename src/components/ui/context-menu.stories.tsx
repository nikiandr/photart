import { PencilSimple } from "@phosphor-icons/react/dist/csr/PencilSimple";
import { Trash } from "@phosphor-icons/react/dist/csr/Trash";
import type { Meta, StoryObj } from "@storybook/react-vite";

import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "./context-menu";

const meta = {
  title: "UI/Context Menu",
  component: ContextMenu,
  parameters: { layout: "centered" },
} satisfies Meta<typeof ContextMenu>;

export default meta;
type Story = StoryObj<typeof meta>;

export const ShootActions: Story = {
  render: () => (
    <ContextMenu>
      <ContextMenuTrigger className="flex h-24 w-64 items-center justify-center border border-border bg-sidebar text-[0.8125rem] text-muted-foreground">
        Right-click this Shoot
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem>
          <PencilSimple aria-hidden="true" />
          Rename Shoot
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem variant="destructive">
          <Trash aria-hidden="true" />
          Purge Shoot…
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  ),
};
