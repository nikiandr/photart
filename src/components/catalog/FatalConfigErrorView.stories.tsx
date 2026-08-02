import type { Meta, StoryObj } from "@storybook/react-vite";

import { FatalConfigErrorView } from "./FatalConfigErrorView";

const meta = {
  title: "Catalog/FatalConfigErrorView",
  component: FatalConfigErrorView,
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
    error: {
      code: "config_parse_error",
      message:
        "Photart couldn’t read its configuration file. The file may be damaged or from an unsupported version.",
    },
  },
} satisfies Meta<typeof FatalConfigErrorView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const UnsupportedVersion: Story = {
  args: {
    error: {
      code: "config_unsupported_version",
      message: "This Catalog was created with a newer version of Photart.",
    },
  },
};
