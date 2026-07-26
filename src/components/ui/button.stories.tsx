import { ArrowClockwise } from "@phosphor-icons/react/dist/csr/ArrowClockwise";
import { UploadSimple } from "@phosphor-icons/react/dist/csr/UploadSimple";
import type { Meta, StoryObj } from "@storybook/react-vite";

import { Button } from "./button";

const variants = [
  "default",
  "outline",
  "secondary",
  "ghost",
  "destructive",
  "link",
] as const;

const sizes = ["xs", "sm", "default", "lg"] as const;
const iconSizes = ["icon-xs", "icon-sm", "icon", "icon-lg"] as const;

const meta = {
  title: "UI/Button",
  component: Button,
  parameters: {
    layout: "centered",
  },
  args: {
    children: "Choose photo",
  },
  argTypes: {
    variant: {
      control: "select",
      options: variants,
    },
    size: {
      control: "select",
      options: [...sizes, ...iconSizes],
    },
  },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Playground: Story = {};

export const WithIcon: Story = {
  args: {
    children: (
      <>
        <UploadSimple data-icon="inline-start" aria-hidden="true" />
        Choose photo
      </>
    ),
  },
};

export const KeyboardFocus: Story = {
  args: {
    autoFocus: true,
    children: (
      <>
        <UploadSimple data-icon="inline-start" aria-hidden="true" />
        Choose photo
      </>
    ),
  },
  parameters: {
    docs: {
      description: {
        story: "Tab to the button to confirm the focus ring is visible in both atmospheres.",
      },
    },
  },
};

export const Disabled: Story = {
  args: {
    disabled: true,
  },
};

export const Variants: Story = {
  render: () => (
    <div className="flex flex-wrap items-center gap-3">
      {variants.map((variant) => (
        <Button key={variant} variant={variant}>
          {variant === "default"
            ? "Analyze"
            : variant === "outline"
              ? "Analyze another"
              : variant === "secondary"
                ? "Compare"
                : variant === "ghost"
                  ? "Dismiss"
                  : variant === "destructive"
                    ? "Purge"
                    : "Open in Finder"}
        </Button>
      ))}
    </div>
  ),
};

export const Sizes: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      <div className="flex items-center gap-3">
        {sizes.map((size) => (
          <Button key={size} size={size}>
            {size === "default" ? "Choose photo" : size}
          </Button>
        ))}
      </div>
      <div className="flex items-center gap-3">
        {iconSizes.map((size) => (
          <Button key={size} size={size} aria-label="Choose photo">
            <UploadSimple aria-hidden="true" />
          </Button>
        ))}
        <Button variant="outline" size="icon" aria-label="Analyze another">
          <ArrowClockwise aria-hidden="true" />
        </Button>
      </div>
    </div>
  ),
};
