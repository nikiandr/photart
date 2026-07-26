import { WarningCircle } from "@phosphor-icons/react/dist/csr/WarningCircle";
import type { Meta, StoryObj } from "@storybook/react-vite";

import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Separator } from "@/components/ui/separator";

function AtmosphereSample({ label }: { label: string }) {
  return (
    <div className="flex min-h-[28rem] flex-col gap-5 bg-background p-6 text-foreground">
      <div className="space-y-1">
        <p className="font-sans text-xs font-medium tracking-wide text-muted-foreground uppercase">
          {label}
        </p>
        <h2 className="font-sans text-lg font-medium">Quiet primitives</h2>
        <p className="max-w-md text-sm text-muted-foreground">
          Same components on each atmosphere. Photographs stay the strongest
          color source.
        </p>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Button>Choose photo</Button>
        <Button variant="outline">Analyze another</Button>
        <Button variant="secondary">Compare</Button>
        <Button variant="ghost">Dismiss</Button>
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Badge>Suggested frame</Badge>
        <Badge variant="outline">All good</Badge>
        <Badge variant="secondary" className="font-mono text-[0.6875rem]">
          ISO 80
        </Badge>
      </div>

      <Alert>
        <AlertTitle>Shoot ready</AlertTitle>
        <AlertDescription>
          Technical Check finished. Review when you&apos;re ready.
        </AlertDescription>
      </Alert>

      <Alert variant="destructive">
        <WarningCircle weight="fill" aria-hidden="true" />
        <AlertTitle>Couldn&apos;t analyze that photo</AlertTitle>
        <AlertDescription>
          Select a JPEG, PNG, or WebP image and try again.
        </AlertDescription>
      </Alert>

      <Card className="max-w-sm">
        <CardHeader className="border-b">
          <CardTitle>Export package</CardTitle>
          <CardDescription>A discrete object, not a photo stage.</CardDescription>
        </CardHeader>
        <CardContent className="font-mono text-[0.8125rem]">
          48 photographs · ~/Pictures/Selects
        </CardContent>
      </Card>

      <div className="space-y-3">
        <p className="font-sans text-sm font-medium">Stage surface</p>
        <div className="flex h-28 items-center justify-center bg-stage text-stage-foreground">
          <span className="font-mono text-[0.8125rem] text-muted-foreground">
            stage / stage-foreground
          </span>
        </div>
        <Separator />
      </div>
    </div>
  );
}

const meta = {
  title: "Foundations/Atmospheres",
  parameters: {
    layout: "fullscreen",
  },
} satisfies Meta;

export default meta;
type Story = StoryObj;

export const Gallery: Story = {
  parameters: {
    themes: {
      themeOverride: "Gallery",
    },
  },
  render: () => <AtmosphereSample label="Gallery atmosphere" />,
};

export const Review: Story = {
  parameters: {
    themes: {
      themeOverride: "Review",
    },
  },
  render: () => <AtmosphereSample label="Review atmosphere" />,
};

export const SideBySide: Story = {
  name: "Gallery and Review",
  parameters: {
    themes: {
      disable: true,
    },
  },
  render: () => (
    <div className="grid min-h-svh lg:grid-cols-2">
      <div data-atmosphere="gallery">
        <AtmosphereSample label="Gallery" />
      </div>
      <div data-atmosphere="review">
        <AtmosphereSample label="Review" />
      </div>
    </div>
  ),
};
