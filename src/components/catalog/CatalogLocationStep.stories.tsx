import type { Meta, StoryObj } from "@storybook/react-vite";

import { CatalogLocationStep } from "./CatalogLocationStep";

const meta = {
  title: "Catalog/CatalogLocationStep",
  component: CatalogLocationStep,
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
    catalogName: "Photart Catalog",
    parentDirectory: "/Users/alex/Pictures",
    picturesDirectory: "/Users/alex/Pictures",
    error: null,
    creating: false,
    onChooseDirectory: () => undefined,
    onBack: () => undefined,
    onCreate: () => undefined,
  },
} satisfies Meta<typeof CatalogLocationStep>;

export default meta;
type Story = StoryObj<typeof meta>;

export const WithPicturesDefault: Story = {};

export const NoFolderSelected: Story = {
  args: {
    parentDirectory: null,
  },
};

export const Creating: Story = {
  args: {
    creating: true,
  },
};

export const RecoverableError: Story = {
  args: {
    error: "A Catalog already exists at this location.",
  },
};
