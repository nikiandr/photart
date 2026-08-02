/** Frontend mirror of backend single-path-component name rules. */
export function validateName(name: string): string | null {
  const trimmed = name.trim();
  if (trimmed.length === 0) {
    return "Name cannot be empty.";
  }
  if (trimmed === "." || trimmed === "..") {
    return 'Name cannot be "." or "..".';
  }
  if (trimmed.includes("/") || trimmed.includes("\\")) {
    return "Name cannot contain path separators.";
  }
  if (trimmed.includes("\0")) {
    return "Name cannot contain null characters.";
  }
  return null;
}

export function pathBasename(path: string): string {
  const cleaned = path.replace(/[/\\]+$/, "");
  const parts = cleaned.split(/[/\\]/);
  return parts[parts.length - 1] || "Shoot";
}
