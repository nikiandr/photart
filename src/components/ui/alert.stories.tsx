import { Info } from "@phosphor-icons/react/dist/csr/Info";
import { WarningCircle } from "@phosphor-icons/react/dist/csr/WarningCircle";
import type { Meta, StoryObj } from "@storybook/react-vite";

import { Alert, AlertDescription, AlertTitle } from "./alert";

const meta = {
  title: "UI/Alert",
  component: Alert,
  parameters: {
    layout: "centered",
  },
  decorators: [
    (Story) => (
      <div className="w-[min(32rem,calc(100vw-2rem))]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Alert>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <Alert>
      <Info aria-hidden="true" />
      <AlertTitle>Photo ready</AlertTitle>
      <AlertDescription>
        Metadata was read successfully and remains on this device.
      </AlertDescription>
    </Alert>
  ),
};

export const Destructive: Story = {
  render: () => (
    <Alert variant="destructive">
      <WarningCircle weight="fill" aria-hidden="true" />
      <AlertTitle>Couldn&apos;t analyze that photo</AlertTitle>
      <AlertDescription>
        Select a JPEG, PNG, or WebP image and try again.
      </AlertDescription>
    </Alert>
  ),
};
