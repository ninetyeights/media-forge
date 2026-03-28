import { writable } from "svelte/store";
import { load } from "@tauri-apps/plugin-store";

export const ALL_IMAGE_EXTS = ["jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff"];
export const ALL_VIDEO_EXTS = ["mp4", "webm", "mov", "avi", "mkv", "flv", "wmv"];
export const ALL_AUDIO_EXTS = ["mp3", "aac", "ogg", "wav", "flac", "m4a", "wma"];

export function extsForType(mediaType: "image" | "video" | "audio"): string[] {
  if (mediaType === "image") return [...ALL_IMAGE_EXTS];
  if (mediaType === "video") return [...ALL_VIDEO_EXTS];
  return [...ALL_AUDIO_EXTS];
}

export interface WatchFolder {
  id: string;
  path: string;
  active: boolean;
  mediaType: "image" | "video" | "audio";
  fileExts: string[];
  autoProcess: boolean;
  preset: string;
  outputFormat: string;
  outputDir: string;
  outputMode: "overwrite" | "saveto";
  preserveStructure: boolean;
  recursive: boolean;
}

export function createWatchFolder(path: string): WatchFolder {
  return {
    id: crypto.randomUUID(),
    path,
    active: false,
    mediaType: "image",
    fileExts: [...ALL_IMAGE_EXTS],
    autoProcess: true,
    preset: "lossless",
    outputFormat: "original",
    outputDir: "",
    outputMode: "saveto",
    preserveStructure: true,
    recursive: true,
  };
}

export const watchFolders = writable<WatchFolder[]>([]);

// --- Persistence ---

const STORE_FILE = "watcher.json";
const STORE_KEY = "folders";

let storeReady = false;

export async function loadWatchFolders(): Promise<void> {
  try {
    const store = await load(STORE_FILE);
    const saved = await store.get<WatchFolder[]>(STORE_KEY);
    if (saved && Array.isArray(saved) && saved.length > 0) {
      // Reset active state on load (watchers aren't running yet)
      watchFolders.set(saved.map((f) => ({ ...f, active: false })));
    }
    storeReady = true;

    watchFolders.subscribe((folders) => {
      if (storeReady) saveWatchFolders(folders);
    });
  } catch (_) {
    storeReady = true;
  }
}

async function saveWatchFolders(folders: WatchFolder[]): Promise<void> {
  try {
    const store = await load(STORE_FILE);
    await store.set(STORE_KEY, folders);
    await store.save();
  } catch (_) {}
}
