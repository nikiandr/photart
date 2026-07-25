import type { Meta, StoryObj } from "@storybook/react-vite";

import { Separator } from "./separator";

const meta = {
  title: "UI/Separator",
  component: Separator,
  parameters: {
    layout: "centered",
  },
} satisfies Meta<typeof Separator>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Horizontal: Story = {
  render: () => (
    <div className="w-72 space-y-3">
      <div>
        <p className="font-medium">Photo metadata</p>
        <p className="text-muted-foreground">Extracted locally</p>
      </div>
      <Separator />
      <p>4032 × 3024 · 4.8 MB</p>
    </div>
  ),
};

export const Vertical: Story = {
  render: () => (
    <div className="flex h-5 items-center gap-3">
      <span>JPEG</span>
      <Separator orientation="vertical" />
      <span>4.8 MB</span>
      <Separator orientation="vertical" />
      <span>sRGB</span>
    </div>
  ),
};
