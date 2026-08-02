import type { Meta, StoryObj } from "@storybook/react-vite";

import { Sidebar } from "./Sidebar";
import { WorkspaceShell } from "./WorkspaceShell";
import type { ShootSummary } from "@/types";

const sampleShoots: ShootSummary[] = [
  {
    name: "Tel Aviv Morning",
    relativePath: "Shoots/Tel Aviv Morning",
    importedAt: "2026-08-01T10:00:00Z",
    photographCount: 128,
    skippedFileCount: 2,
  },
  {
    name: "Harbor Walk",
    relativePath: "Shoots/Harbor Walk",
    importedAt: "2026-08-01T14:00:00Z",
    photographCount: 54,
    skippedFileCount: 0,
  },
  {
    name: "Very Long Shoot Name That Should Truncate Nicely",
    relativePath: "Shoots/Very Long Shoot Name That Should Truncate Nicely",
    importedAt: "2026-08-01T18:00:00Z",
    photographCount: 9,
    skippedFileCount: 0,
  },
];

const meta = {
  title: "Workspace/WorkspaceShell",
  component: WorkspaceShell,
  parameters: {
    layout: "fullscreen",
  },
} satisfies Meta<typeof WorkspaceShell>;

export default meta;
type Story = StoryObj<typeof meta>;

export const EmptyCatalog: Story = {
  args: {
    sidebar: (
      <Sidebar
        catalogName="Photart Catalog"
        shoots={[]}
        active="home"
        onHome={() => undefined}
        onCreateShoot={() => undefined}
        onSelectShoot={() => undefined}
        onRenameShoot={async () => undefined}
        onPurgeShoot={async () => undefined}
      />
    ),
    children: (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Main workspace
      </div>
    ),
  },
};

export const Populated: Story = {
  args: {
    sidebar: (
      <Sidebar
        catalogName="Photart Catalog"
        shoots={sampleShoots}
        active={{ shootName: "Harbor Walk" }}
        onHome={() => undefined}
        onCreateShoot={() => undefined}
        onSelectShoot={() => undefined}
        onRenameShoot={async () => undefined}
        onPurgeShoot={async () => undefined}
      />
    ),
    children: (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Harbor Walk
      </div>
    ),
  },
};

export const ImportDisabled: Story = {
  args: {
    sidebar: (
      <Sidebar
        catalogName="Photart Catalog"
        shoots={sampleShoots}
        active={null}
        disabled
        onHome={() => undefined}
        onCreateShoot={() => undefined}
        onSelectShoot={() => undefined}
        onRenameShoot={async () => undefined}
        onPurgeShoot={async () => undefined}
      />
    ),
    children: (
      <div className="flex h-full items-center justify-center text-sm text-muted-foreground">
        Import in progress
      </div>
    ),
  },
};
