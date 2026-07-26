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
        <p className="font-sans text-sm font-medium">Exposure</p>
        <p className="text-sm text-muted-foreground">Read from the file</p>
      </div>
      <Separator />
      <p className="font-mono text-[0.8125rem]">
        1/120 s · ƒ/1.78 · ISO 80 · 24 mm
      </p>
    </div>
  ),
};

export const Vertical: Story = {
  render: () => (
    <div className="flex h-5 items-center gap-3 font-mono text-[0.8125rem]">
      <span>JPEG</span>
      <Separator orientation="vertical" />
      <span>4.8 MB</span>
      <Separator orientation="vertical" />
      <span>4032 × 3024</span>
    </div>
  ),
};
