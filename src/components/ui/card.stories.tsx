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
  name: "Discrete object",
  render: () => (
    <Card>
      <CardHeader className="border-b">
        <CardTitle>Export package</CardTitle>
        <CardDescription>
          A discrete handoff object — not a photograph stage.
        </CardDescription>
        <CardAction>
          <Badge variant="outline">Ready</Badge>
        </CardAction>
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-[1fr_auto] gap-x-4 gap-y-2 text-sm">
          <dt className="text-muted-foreground">Photographs</dt>
          <dd className="font-mono text-[0.8125rem]">48</dd>
          <dt className="text-muted-foreground">Destination</dt>
          <dd className="font-mono text-[0.8125rem]">~/Pictures/Selects</dd>
        </dl>
      </CardContent>
      <CardFooter className="justify-end gap-2">
        <Button variant="ghost">Dismiss</Button>
        <Button>Open folder</Button>
      </CardFooter>
    </Card>
  ),
};

export const Compact: Story = {
  render: () => (
    <Card size="sm">
      <CardHeader>
        <CardTitle>Nearby alternatives</CardTitle>
        <CardDescription>3 frames within ±0.4 s</CardDescription>
      </CardHeader>
      <CardContent className="font-mono text-[0.8125rem] text-muted-foreground">
        −0.2 s · +0.1 s · +0.4 s
      </CardContent>
    </Card>
  ),
};
