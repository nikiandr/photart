import type { Meta, StoryObj } from "@storybook/react-vite";

import { ThumbnailGrid } from "./ThumbnailGrid";
import type { PhotographEntry } from "@/types";

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
  };
}

const mixedAspectPhotographs: PhotographEntry[] = [
  photo("landscape-01.jpg", 1600, 1000, "#c4b5a0"),
  photo("portrait-01.jpg", 900, 1400, "#9aab9a"),
  photo("square-01.jpg", 1100, 1100, "#a8b4c4"),
  photo("landscape-02.jpg", 1800, 900, "#b5a89a"),
  photo("portrait-02.jpg", 800, 1200, "#8fa0b0"),
  photo("square-02.jpg", 1000, 1000, "#b0a898"),
];

const meta = {
  title: "Workspace/ThumbnailGrid",
  component: ThumbnailGrid,
  parameters: {
    layout: "padded",
  },
  decorators: [
    (Story) => (
      <div className="mx-auto max-w-4xl bg-background p-4">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ThumbnailGrid>;

export default meta;
type Story = StoryObj<typeof meta>;

export const MixedAspectRatios: Story = {
  args: {
    photographs: mixedAspectPhotographs,
  },
};

export const Empty: Story = {
  args: {
    photographs: [],
  },
};

export const PortraitLandscapeSquare: Story = {
  name: "Portrait, landscape, and square",
  args: {
    photographs: [
      photo("portrait.jpg", 800, 1200, "#9aab9a"),
      photo("landscape.jpg", 1600, 1000, "#c4b5a0"),
      photo("square.jpg", 1000, 1000, "#a8b4c4"),
    ],
  },
};
