import type { Meta, StoryObj } from "@storybook/react-vite";
import { expect } from "storybook/test";

import { ShootView } from "./ShootView";
import type { PhotographCapture, PhotographEntry, ShootDetails } from "@/types";

function placeholderUrl(width: number, height: number, color: string): string {
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}"><rect width="100%" height="100%" fill="${color}"/></svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

function photo(
  fileName: string,
  width: number,
  height: number,
  color: string,
): PhotographEntry {
  return {
    relativePath: `photographs/${fileName}`,
    fileName,
    width,
    height,
    thumbnailRelativePath: `thumbnails/${fileName}`,
    thumbnailAbsolutePath: `/tmp/${fileName}`,
    thumbnailUrl: placeholderUrl(width, height, color),
    originalAvailable: true,
    displayUrl: placeholderUrl(width, height, color),
    usesDetailCache: false,
  };
}

const sampleShoot: ShootDetails = {
  summary: {
    name: "Tel Aviv Morning",
    relativePath: "Shoots/Tel Aviv Morning",
    importedAt: "2026-08-01T10:00:00Z",
    photographCount: 3,
    skippedFileCount: 2,
    medium: null,
  },
  photographs: [
    photo("IMG_0101.jpg", 1200, 800, "#c4b5a0"),
    photo("IMG_0102.jpg", 800, 1200, "#9aab9a"),
    photo("IMG_0103.jpg", 1000, 1000, "#a8b4c4"),
  ],
};

const meta = {
  title: "Workspace/ShootView",
  component: ShootView,
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
    shoot: sampleShoot,
    skippedNoticeCount: null,
    loading: false,
    loadCapture: async (_shootName, relativePath): Promise<PhotographCapture> => {
      const photograph = sampleShoot.photographs.find((photo) => photo.relativePath === relativePath);
      if (!photograph) throw new Error("Photograph is unavailable.");
      return {
        camera: "Canon EOS R6",
        lens: "RF 50mm F1.8 STM",
        capturedAt: "2026-08-01 07:35:00",
        aperture: "f/2.8",
        shutterSpeed: "1/250 s",
        iso: photograph.width > photograph.height ? 200 : 800,
      };
    },
  },
} satisfies Meta<typeof ShootView>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const PhotographDetail: Story = {
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(await canvas.findByRole("button", { name: "IMG_0101.jpg" }));
  },
};

export const RawJpegNavigation: Story = {
  args: {
    shoot: {
      summary: { ...sampleShoot.summary, skippedFileCount: 0 },
      photographs: [
        { ...photo("IMG_0101.DNG", 1200, 800, "#ad9b86"), jpegCompanionPath: "photographs/IMG_0101.jpg" },
        sampleShoot.photographs[0],
        { ...photo("IMG_0102.NEF", 800, 1200, "#7f9180"), jpegCompanionPath: "photographs/IMG_0102.jpg" },
        sampleShoot.photographs[1],
        sampleShoot.photographs[2],
      ],
    },
    loadCapture: async () => ({ camera: null, lens: null, capturedAt: null, aperture: null, shutterSpeed: null, iso: null }),
  },
  play: async ({ canvas, userEvent }) => {
    const toggle = await canvas.findByRole("button", { name: "RAW + JPEG: IMG_0101.DNG" });
    await expect(canvas.queryByRole("button", { name: /^IMG_0101\.jpg$/ })).not.toBeInTheDocument();
    await userEvent.click(toggle);
    await userEvent.click(await canvas.findByRole("button", { name: /^IMG_0101\.jpg$/ }));
    await expect(canvas.getByRole("button", { name: /^JPEG$/ })).toHaveAttribute("aria-pressed", "true");
    await expect(canvas.getByText("1 / 3")).toBeVisible();

    await userEvent.keyboard("{ArrowRight}");
    await expect(canvas.getByRole("heading", { name: "IMG_0102.NEF" })).toBeVisible();
    await userEvent.click(canvas.getByRole("button", { name: /^JPEG$/ }));
    await expect(canvas.getByRole("heading", { name: "IMG_0102.jpg" })).toBeVisible();

    await userEvent.click(canvas.getByRole("button", { name: "Next photograph" }));
    await expect(canvas.getByRole("heading", { name: "IMG_0103.jpg" })).toBeVisible();
    await userEvent.click(canvas.getByRole("button", { name: "Previous photograph" }));
    await expect(canvas.getByRole("heading", { name: "IMG_0102.NEF" })).toBeVisible();
    await userEvent.keyboard("{ArrowLeft}");
    await expect(canvas.getByRole("heading", { name: "IMG_0101.DNG" })).toBeVisible();
  },
};

export const MissingRawPreview: Story = {
  args: {
    shoot: {
      summary: { ...sampleShoot.summary, photographCount: 1, skippedFileCount: 0 },
      photographs: [
        {
          ...photo("IMG_0101.DNG", 1200, 800, "#ad9b86"),
          jpegCompanionPath: "photographs/IMG_0101.jpg",
          originalAvailable: false,
          displayUrl: null,
        },
        sampleShoot.photographs[0],
      ],
    },
  },
  play: async ({ canvas, userEvent }) => {
    await userEvent.click(await canvas.findByRole("button", { name: /^IMG_0101\.DNG$/ }));
    await expect(canvas.getByRole("heading", { name: "IMG_0101.jpg" })).toBeVisible();
    await expect(canvas.getByText("RAW unavailable. Showing JPEG.")).toBeVisible();
  },
};

export const WithSkippedNotice: Story = {
  args: {
    skippedNoticeCount: 2,
  },
};

export const Loading: Story = {
  args: {
    loading: true,
  },
};
