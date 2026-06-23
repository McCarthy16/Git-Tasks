/** Last path segment of a filesystem path (handles both separators). */
export function basename(path: string): string {
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts[parts.length - 1] ?? path;
}
