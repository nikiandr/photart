import type { Meta, StoryObj } from "@storybook/react-vite";

import { Badge } from "./badge";
import { Button } from "./button";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "./card";

const meta = {
  title: "UI/Card",
  component: Card,
  parameters: {
    layout: "centered",
  },
  decorators: [
    (Story) => (
      <div className="w-[min(28rem,calc(100vw-2rem))]">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Card>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {
  render: () => (
    <Card>
      <CardHeader>
        <CardTitle>Photo details</CardTitle>
        <CardDescription>
          Metadata extracted from the selected image.
        </CardDescription>
        <CardAction>
          <Badge variant="outline">JPEG</Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-2 gap-3">
          <dt className="text-muted-foreground">Dimensions</dt>
          <dd className="text-right font-medium">4032 × 3024</dd>
          <dt className="text-muted-foreground">File size</dt>
          <dd className="text-right font-medium">4.8 MB</dd>
        </dl>
      </CardContent>
      <CardFooter className="justify-end gap-2">
        <Button variant="ghost">Dismiss</Button>
        <Button>View photo</Button>
      </CardFooter>
    </Card>
  ),
};

export const Compact: Story = {
  render: () => (
    <Card size="sm">
      <CardHeader>
        <CardTitle>Camera</CardTitle>
        <CardDescription>Apple iPhone 15 Pro</CardDescription>
      </CardHeader>
      <CardContent>24 mm · ƒ/1.78 · 1/120 s · ISO 80</CardContent>
    </Card>
  ),
};
