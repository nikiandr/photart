import type { Meta, StoryObj } from "@storybook/react-vite";

import { Badge } from "./badge";

const variants = [
  "default",
  "secondary",
  "destructive",
  "outline",
  "ghost",
  "link",
] as const;

const meta = {
  title: "UI/Badge",
  component: Badge,
  parameters: {
    layout: "centered",
  },
  args: {
    children: "Suggested frame",
  },
  argTypes: {
    variant: {
      control: "select",
      options: variants,
    },
  },
} satisfies Meta<typeof Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Playground: Story = {};

export const QuietDefault: Story = {
  name: "Quiet default",
  render: () => (
    <div className="flex flex-col items-start gap-4">
      <p className="max-w-sm text-sm text-muted-foreground">
        Default badges are quiet status marks, not primary actions.
      </p>
      <div className="flex flex-wrap items-center gap-3">
        <Badge>Suggested frame</Badge>
        <Badge variant="outline">Minor issues</Badge>
        <Badge variant="secondary">Burst · 5</Badge>
      </div>
    </div>
  ),
};

export const Variants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      {variants.map((variant) => (
        <Badge key={variant} variant={variant}>
          {variant === "default"
            ? "Suggested frame"
            : variant === "secondary"
              ? "In catalog"
              : variant === "destructive"
                ? "Unsupported"
                : variant === "outline"
                  ? "All good"
                  : variant === "ghost"
                    ? "Nearby"
                    : "Open file"}
        </Badge>
      ))}
    </div>
  ),
};

export const WithMonoMetadata: Story = {
  name: "With mono metadata",
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      <Badge variant="outline" className="font-mono text-[0.6875rem]">
        JPEG
      </Badge>
      <Badge variant="outline" className="font-mono text-[0.6875rem]">
        4032 × 3024
      </Badge>
      <Badge variant="outline" className="font-mono text-[0.6875rem]">
        ISO 80
      </Badge>
    </div>
  ),
};
