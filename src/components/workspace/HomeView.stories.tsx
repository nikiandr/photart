import type { Meta, StoryObj } from "@storybook/react-vite";

import { HomeView } from "./HomeView";

const meta = {
  title: "Workspace/HomeView",
  component: HomeView,
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
    onCreateShoot: () => undefined,
  },
} satisfies Meta<typeof HomeView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
