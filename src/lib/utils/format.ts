export const formatModelSize = (sizeMb: number | null | undefined): string => {
  if (!sizeMb || !Number.isFinite(sizeMb) || sizeMb <= 0) {
    return "Unknown size";
  }

  if (sizeMb >= 1024) {
    const sizeGb = sizeMb / 1024;
    const formatter = new Intl.NumberFormat(undefined, {
      minimumFractionDigits: sizeGb >= 10 ? 0 : 1,
      maximumFractionDigits: sizeGb >= 10 ? 0 : 1,
    });
    return `${formatter.format(sizeGb)} GB`;
  }

  const formatter = new Intl.NumberFormat(undefined, {
    minimumFractionDigits: sizeMb >= 100 ? 0 : 1,
    maximumFractionDigits: sizeMb >= 100 ? 0 : 1,
  });

  return `${formatter.format(sizeMb)} MB`;
};

/**
 * Format a byte count as a short, locale-aware size (e.g. "4.2 MB").
 */
export const formatFileSize = (bytes: number | null | undefined): string => {
  if (!bytes || !Number.isFinite(bytes) || bytes <= 0) {
    return "—";
  }

  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }

  const formatter = new Intl.NumberFormat(undefined, {
    minimumFractionDigits: unitIndex === 0 || value >= 100 ? 0 : 1,
    maximumFractionDigits: unitIndex === 0 || value >= 100 ? 0 : 1,
  });

  return `${formatter.format(value)} ${units[unitIndex]}`;
};

/**
 * Format a millisecond duration as `h:mm:ss` or `m:ss`.
 * Digit-only, so it reads the same in every locale.
 */
export const formatDuration = (
  durationMs: number | null | undefined,
): string | null => {
  if (
    durationMs === null ||
    durationMs === undefined ||
    !Number.isFinite(durationMs) ||
    durationMs < 0
  ) {
    return null;
  }

  const totalSeconds = Math.round(durationMs / 1000);
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pad = (n: number) => String(n).padStart(2, "0");

  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(seconds)}`
    : `${minutes}:${pad(seconds)}`;
};
