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
      <div className="w-[min(32rem,calc(100vw-2rem))] space-y-4">
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
      <AlertTitle>Shoot imported</AlertTitle>
      <AlertDescription>
        248 photographs are ready for Technical Check. Everything stayed on this
        device.
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
        Select a JPEG, PNG, or WebP image under 100 MiB and try again.
      </AlertDescription>
    </Alert>
  ),
};

export const Hierarchy: Story = {
  name: "Default vs destructive",
  render: () => (
    <>
      <Alert>
        <Info aria-hidden="true" />
        <AlertTitle>Metadata is ready</AlertTitle>
        <AlertDescription>
          Filename, exposure, and location were read without leaving this Mac.
        </AlertDescription>
      </Alert>
      <Alert variant="destructive">
        <WarningCircle weight="fill" aria-hidden="true" />
        <AlertTitle>Couldn&apos;t analyze that photo</AlertTitle>
        <AlertDescription>
          The file may be unsupported or larger than 100 MiB. Description text
          stays muted; the leading rule and title carry the warning.
        </AlertDescription>
      </Alert>
    </>
  ),
};
