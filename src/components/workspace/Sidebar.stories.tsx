import type { Meta, StoryObj } from "@storybook/react-vite";

import { Sidebar } from "./Sidebar";
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
];

const meta = {
  title: "Workspace/Sidebar",
  component: Sidebar,
  parameters: {
    layout: "fullscreen",
  },
  decorators: [
    (Story) => (
      <div className="flex h-svh w-60 flex-col border-r border-sidebar-border bg-sidebar text-sidebar-foreground">
        <Story />
      </div>
    ),
  ],
  args: {
    catalogName: "Photart Catalog",
    shoots: sampleShoots,
    active: "home" as const,
    disabled: false,
    onHome: () => undefined,
    onCreateShoot: () => undefined,
    onSelectShoot: () => undefined,
    onRenameShoot: async () => undefined,
    onPurgeShoot: async () => undefined,
  },
} satisfies Meta<typeof Sidebar>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Empty: Story = {
  args: {
    shoots: [],
    active: "home",
  },
};

export const PopulatedHome: Story = {};

export const ActiveShoot: Story = {
  args: {
    active: { shootName: "Tel Aviv Morning" },
  },
};

export const Disabled: Story = {
  args: {
    disabled: true,
    active: null,
  },
};
