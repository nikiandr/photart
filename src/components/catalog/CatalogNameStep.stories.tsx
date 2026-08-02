import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";

import { CatalogNameStep } from "./CatalogNameStep";

const meta = {
  title: "Catalog/CatalogNameStep",
  component: CatalogNameStep,
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
    onCatalogNameChange: () => undefined,
    onContinue: () => undefined,
  },
} satisfies Meta<typeof CatalogNameStep>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: function Render(args) {
    const [catalogName, setCatalogName] = useState(args.catalogName);
    return (
      <CatalogNameStep
        catalogName={catalogName}
        onCatalogNameChange={setCatalogName}
        onContinue={args.onContinue}
      />
    );
  },
};

export const EmptyName: Story = {
  args: {
    catalogName: "",
  },
  render: function Render(args) {
    const [catalogName, setCatalogName] = useState(args.catalogName);
    return (
      <CatalogNameStep
        catalogName={catalogName}
        onCatalogNameChange={setCatalogName}
        onContinue={args.onContinue}
      />
    );
  },
};
