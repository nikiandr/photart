import type { Meta, StoryObj } from "@storybook/react-vite";
import { useState } from "react";

import type { ShootMedium } from "@/types";
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
    medium: null,
    sourceHint: "noneFound",
    onShootNameChange: () => undefined,
    onChooseSource: () => undefined,
    onMediumChange: () => undefined,
    onCancel: () => undefined,
    onImport: () => undefined,
  },
} satisfies Meta<typeof ShootConfirmView>;

export default meta;
type Story = StoryObj<typeof meta>;

const interactive: Story["render"] = function Render(args) {
  const [shootName, setShootName] = useState(args.shootName);
  const [mediumOverride, setMediumOverride] = useState<ShootMedium | null>();
  return (
    <ShootConfirmView
      {...args}
      shootName={shootName}
      onShootNameChange={setShootName}
      medium={mediumOverride === undefined ? args.medium : mediumOverride}
      sourceHint={mediumOverride === undefined ? args.sourceHint : "noneFound"}
      onMediumChange={setMediumOverride}
    />
  );
};

export const Default: Story = { render: interactive };

export const NoSourceSelected: Story = {
  args: { shootName: "", sourcePath: null },
  render: interactive,
};

export const FilmDetected: Story = {
  args: { sourceHint: "noritsu", medium: { kind: "film", filmStock: null } },
  render: interactive,
};

export const PossibleFilmRoll: Story = {
  args: { sourceHint: "possibleFilmRoll" },
  render: interactive,
};

export const WithError: Story = {
  args: {
    shootName: "Harbor Walk",
    sourcePath: "/Users/alex/Downloads/Harbor Walk",
    error: "A Shoot with this name already exists.",
  },
  render: interactive,
};
