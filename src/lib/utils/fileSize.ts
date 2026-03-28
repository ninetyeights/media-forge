import { get } from "svelte/store";
import { appSettings } from "$lib/stores/settings";

export function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const base = get(appSettings).sizeBase;
  const sizes = base === 1024 ? ["B", "KB", "MB", "GB"] : ["B", "kB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(base));
  return parseFloat((bytes / Math.pow(base, i)).toFixed(1)) + " " + sizes[i];
}

export function formatRatio(ratio: number): string {
  return (ratio * 100).toFixed(1) + "%";
}
