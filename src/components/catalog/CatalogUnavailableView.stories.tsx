import type { Meta, StoryObj } from "@storybook/react-vite";

import { CatalogUnavailableView } from "./CatalogUnavailableView";

const meta = {
  title: "Catalog/CatalogUnavailableView",
  component: CatalogUnavailableView,
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
    name: "Photart Catalog",
    path: "/Users/alex/Pictures/Photart Catalog",
    reason: "The Catalog folder could not be found on this Mac.",
    onSetupNew: () => undefined,
  },
} satisfies Meta<typeof CatalogUnavailableView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};
