import { cn } from "@/lib/utils";

export type PhotartMarkProps = {
  className?: string;
  title?: string;
  appearance?: "icon" | "sidebar";
};

// The icon palette matches src-tauri/icons/icon-source.svg.
const palettes = {
  icon: {
    field: "#202329",
    farFrame: "#5e6268",
    middleFrame: "#8b8d8d",
    foreground: "#f4f1ec",
  },
  sidebar: {
    field: "var(--sidebar-accent)",
    farFrame: "color-mix(in oklab, var(--sidebar-foreground) 52%, var(--sidebar-accent))",
    middleFrame: "var(--muted-foreground)",
    foreground: "var(--sidebar-foreground)",
  },
};

export function PhotartMark({
  className,
  title = "Photart",
  appearance = "icon",
}: PhotartMarkProps) {
  const { field, farFrame, middleFrame, foreground } = palettes[appearance];

  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 650 650"
      role="img"
      aria-label={title}
      className={cn("size-8 shrink-0", className)}
    >
      <title>{title}</title>
      <rect width="650" height="650" rx="30" fill={field} />
      <g fill={farFrame}>
        <rect x="161" y="230" width="34" height="252" />
        <rect x="161" y="230" width="194" height="34" />
        <rect x="321" y="230" width="34" height="252" />
        <rect x="161" y="448" width="194" height="34" />
      </g>
      <rect x="220" y="170" width="225" height="351" fill={field} />
      <g fill={middleFrame}>
        <rect x="220" y="170" width="38" height="351" />
        <rect x="220" y="170" width="225" height="38" />
        <rect x="407" y="170" width="38" height="351" />
        <rect x="220" y="483" width="225" height="38" />
      </g>
      <rect x="289" y="99" width="242" height="258" fill={field} />
      <g fill={foreground}>
        <rect x="289" y="99" width="54" height="446" />
        <rect x="289" y="99" width="242" height="54" />
        <rect x="477" y="99" width="54" height="258" />
        <rect x="289" y="303" width="242" height="54" />
      </g>
    </svg>
  );
}
