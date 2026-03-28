import { writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";

export interface ProgressPayload {
  fileId: string;
  progress: number;
  status: string;
  outputSize?: number;
}

export const progressMap = writable<Map<string, ProgressPayload>>(new Map());

let initialized = false;

export async function initProgressListener() {
  if (initialized) return;
  initialized = true;

  await listen<ProgressPayload>("processing-progress", (event) => {
    progressMap.update((map) => {
      map.set(event.payload.fileId, event.payload);
      return new Map(map);
    });
  });
}
