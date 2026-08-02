import { cn } from "@/lib/utils";

export type PhotartMarkProps = {
  className?: string;
  title?: string;
};

/**
 * Released Frame mark: a warm-paper P in front of two receding photographic
 * frames on a cool graphite field. Colors are identity constants from the
 * app icon source (`src-tauri/icons/icon-source.svg`).
 */
export function PhotartMark({
  className,
  title = "Photart",
}: PhotartMarkProps) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 650 650"
      role="img"
      aria-label={title}
      className={cn("size-8 shrink-0", className)}
    >
      <title>{title}</title>
      <rect width="650" height="650" rx="30" fill="#202329" />
      <g fill="#5e6268">
        <rect x="161" y="230" width="34" height="252" />
        <rect x="161" y="230" width="194" height="34" />
        <rect x="321" y="230" width="34" height="252" />
        <rect x="161" y="448" width="194" height="34" />
      </g>
      <rect x="220" y="170" width="225" height="351" fill="#202329" />
      <g fill="#8b8d8d">
        <rect x="220" y="170" width="38" height="351" />
        <rect x="220" y="170" width="225" height="38" />
        <rect x="407" y="170" width="38" height="351" />
        <rect x="220" y="483" width="225" height="38" />
      </g>
      <rect x="289" y="99" width="242" height="258" fill="#202329" />
      <g fill="#f4f1ec">
        <rect x="289" y="99" width="54" height="446" />
        <rect x="289" y="99" width="242" height="54" />
        <rect x="477" y="99" width="54" height="258" />
        <rect x="289" y="303" width="242" height="54" />
      </g>
    </svg>
  );
}
