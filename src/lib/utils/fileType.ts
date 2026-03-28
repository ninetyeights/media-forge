const IMAGE_EXTS = new Set(["jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff", "tif"]);
const VIDEO_EXTS = new Set(["mp4", "webm", "mov", "avi", "mkv", "flv", "wmv"]);
const AUDIO_EXTS = new Set(["mp3", "aac", "ogg", "wav", "flac", "m4a", "wma"]);

export function getExtension(path: string): string {
  return path.split(".").pop()?.toLowerCase() || "";
}

export function isImageFile(path: string): boolean {
  return IMAGE_EXTS.has(getExtension(path));
}

export function isVideoFile(path: string): boolean {
  return VIDEO_EXTS.has(getExtension(path));
}

export function isAudioFile(path: string): boolean {
  return AUDIO_EXTS.has(getExtension(path));
}

export function getMediaType(path: string): "image" | "video" | "audio" | null {
  if (isImageFile(path)) return "image";
  if (isVideoFile(path)) return "video";
  if (isAudioFile(path)) return "audio";
  return null;
}

export function getFormatColor(format: string): string {
  const f = format.toLowerCase();
  if (["jpg", "jpeg"].includes(f)) return "#3b82f6";
  if (f === "png") return "#eab308";
  if (f === "webp") return "#22c55e";
  if (f === "avif") return "#a855f7";
  if (f === "gif") return "#f97316";
  if (["bmp", "tiff"].includes(f)) return "#6b7280";
  if (["mp4", "mov", "mkv"].includes(f)) return "#ef4444";
  if (["webm", "avi"].includes(f)) return "#f43f5e";
  if (["mp3", "aac", "m4a"].includes(f)) return "#06b6d4";
  if (["wav", "flac"].includes(f)) return "#8b5cf6";
  if (f === "ogg") return "#14b8a6";
  return "#6b7280";
}
