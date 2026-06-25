/** Format a millisecond timestamp as a human-readable relative time string. */
export function formatTime(millis: number): string {
  const diff = Date.now() - millis;
  const abs = Math.abs(diff);

  if (abs < 60_000) return "just now";
  if (abs < 3_600_000) return `${Math.floor(abs / 60_000)}m ago`;
  if (abs < 86_400_000) return `${Math.floor(abs / 3_600_000)}h ago`;
  if (abs < 7 * 86_400_000) return `${Math.floor(abs / 86_400_000)}d ago`;

  return new Date(millis).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: new Date(millis).getFullYear() !== new Date().getFullYear() ? "numeric" : undefined,
  });
}

/** Format a millisecond timestamp as an absolute date + time string. */
export function formatAbsoluteTime(millis: number): string {
  return new Date(millis).toLocaleString(undefined, {
    month: "short",
    day: "numeric",
    year: new Date(millis).getFullYear() !== new Date().getFullYear() ? "numeric" : undefined,
    hour: "numeric",
    minute: "2-digit",
  });
}
