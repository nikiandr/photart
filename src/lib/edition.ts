/** True for the Vite dev server and `tauri dev`. Release bundles leave the channel unset. */
export function isDevEdition(): boolean {
  return import.meta.env.VITE_PHOTART_CHANNEL === "dev";
}

export function editionName(dev = isDevEdition()): string {
  return dev ? "Photart Dev" : "Photart";
}
