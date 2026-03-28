import { writable } from "svelte/store";

export type ViewName =
  | "image"
  | "video"
  | "audio"
  | "watermark"
  | "watcher"
  | "history"
  | "settings";

export const currentView = writable<ViewName>("image");
export const hasUpdate = writable(false);
