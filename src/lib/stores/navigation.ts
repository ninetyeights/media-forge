import { writable } from "svelte/store";

export type ViewName =
  | "image"
  | "watcher"
  | "settings";

export const currentView = writable<ViewName>("image");
export const hasUpdate = writable(false);
