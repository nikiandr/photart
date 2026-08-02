import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";

import { ShootConfirmView } from "./ShootConfirmView";

const meta = {
  title: "Workspace/ShootConfirmView",
  component: ShootConfirmView,
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
    shootName: "Tel Aviv Morning",
    sourcePath: "/Users/alex/Downloads/Tel Aviv Morning",
    error: null,
    onShootNameChange: () => undefined,
    onCancel: () => undefined,
    onImport: () => undefined,
  },
} satisfies Meta<typeof ShootConfirmView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: function Render(args) {
    const [shootName, setShootName] = useState(args.shootName);
    return (
      <ShootConfirmView
        {...args}
        shootName={shootName}
        onShootNameChange={setShootName}
      />
    );
  },
};

export const WithError: Story = {
  args: {
    shootName: "Harbor Walk",
    sourcePath: "/Users/alex/Downloads/Harbor Walk",
    error: "A Shoot with this name already exists.",
  },
  render: function Render(args) {
    const [shootName, setShootName] = useState(args.shootName);
    return (
      <ShootConfirmView
        {...args}
        shootName={shootName}
        onShootNameChange={setShootName}
      />
    );
  },
};
