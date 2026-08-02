import type { Meta, StoryObj } from "@storybook/react-vite";

import { PhotartBrand } from "./PhotartBrand";
import { PhotartMark } from "./PhotartMark";

const meta = {
  title: "Brand/PhotartBrand",
  component: PhotartBrand,
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof PhotartBrand>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Mark: Story = {
  render: () => <PhotartMark className="size-16" />,
};

export const SidebarLockup: Story = {
  args: {
    layout: "inline",
    detail: "Photart Catalog",
  },
  decorators: [
    (Story) => (
      <div className="w-56 border border-sidebar-border bg-sidebar p-3 text-sidebar-foreground">
        <Story />
      </div>
    ),
  ],
};

export const StackedLockup: Story = {
  args: {
    layout: "stack",
  },
};
