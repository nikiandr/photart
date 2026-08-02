import type { Meta, StoryObj } from "@storybook/react-vite";

import { ImportProgressView } from "./ImportProgressView";

const meta = {
  title: "Workspace/ImportProgressView",
  component: ImportProgressView,
  parameters: {
    layout: "fullscreen",
  },
  decorators: [
    (Story) => (
      <div className="h-svh w-full bg-background">
        <Story />
      </div>
    ),
  ],
  args: {
    shootName: "Tel Aviv Morning",
    onCancel: () => undefined,
  },
} satisfies Meta<typeof ImportProgressView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Scanning: Story = {
  args: {
    progress: {
      phase: "scanning",
      copied: null,
      total: null,
      prepared: null,
    },
  },
};

export const Preparing: Story = {
  args: {
    progress: {
      phase: "preparing",
      copied: null,
      total: 128,
      prepared: 37,
    },
  },
};

export const Copying: Story = {
  args: {
    progress: {
      phase: "copying",
      copied: 42,
      total: 128,
      prepared: 128,
    },
  },
};

export const Finalizing: Story = {
  args: {
    progress: {
      phase: "finalizing",
      copied: 128,
      total: 128,
      prepared: 128,
    },
  },
};
