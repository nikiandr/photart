import type { Meta, StoryObj } from "@storybook/react-vite";

const meta = {
  title: "Foundations/Typography",
  parameters: {
    layout: "padded",
  },
} satisfies Meta;

export default meta;
type Story = StoryObj;

export const Roles: Story = {
  name: "Type roles",
  render: () => (
    <div className="mx-auto flex max-w-2xl flex-col gap-10 bg-background p-6 text-foreground">
      <section className="space-y-3">
        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
          IBM Plex Sans · interface
        </p>
        <h1 className="font-sans text-2xl font-semibold tracking-tight">
          Review this Shoot
        </h1>
        <h2 className="font-sans text-lg font-medium">Technical findings</h2>
        <p className="font-sans text-sm/relaxed text-foreground">
          Photart presents observations calmly. Controls, navigation, findings,
          and explanatory copy use Sans at a 14px baseline.
        </p>
        <p className="font-sans text-sm/relaxed text-muted-foreground">
          Focus may fall behind the subject. Continue when you&apos;re ready.
        </p>
      </section>

      <section className="space-y-3">
        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
          Newsreader · editorial
        </p>
        <h2 className="font-editorial text-3xl font-medium tracking-tight">
          Golden Gate evening
        </h2>
        <p className="font-editorial text-base/relaxed text-muted-foreground">
          Reserved for Shoot names, Gallery headings, and sparse reflective
          moments — never dense chrome.
        </p>
      </section>

      <section className="space-y-3">
        <p className="text-xs font-medium tracking-wide text-muted-foreground uppercase">
          IBM Plex Mono · data
        </p>
        <dl className="grid grid-cols-[8rem_1fr] gap-x-4 gap-y-2 text-sm">
          <dt className="font-sans text-muted-foreground">Filename</dt>
          <dd className="font-mono text-[0.8125rem]">IMG_2048.JPG</dd>
          <dt className="font-sans text-muted-foreground">Exposure</dt>
          <dd className="font-mono text-[0.8125rem]">1/120 s · ƒ/1.78 · ISO 80</dd>
          <dt className="font-sans text-muted-foreground">Dimensions</dt>
          <dd className="font-mono text-[0.8125rem]">4032 × 3024</dd>
          <dt className="font-sans text-muted-foreground">Coordinates</dt>
          <dd className="font-mono text-[0.8125rem]">37.819900 · −122.478300</dd>
          <dt className="font-sans text-muted-foreground">Count</dt>
          <dd className="font-mono text-[0.8125rem]">12 selected</dd>
          <dt className="font-sans text-muted-foreground">Shortcut</dt>
          <dd className="font-mono text-[0.8125rem]">⌘↩ Select</dd>
        </dl>
      </section>
    </div>
  ),
};
