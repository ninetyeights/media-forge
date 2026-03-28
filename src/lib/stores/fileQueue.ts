import { writable, derived } from "svelte/store";
import type { MediaFile } from "$lib/types";

export const imageFiles = writable<MediaFile[]>([]);
export const videoFiles = writable<MediaFile[]>([]);
export const audioFiles = writable<MediaFile[]>([]);
export const watermarkFiles = writable<MediaFile[]>([]);

export const isProcessing = writable(false);

export function sendToWatermark(file: MediaFile) {
  watermarkFiles.update((q) => [...q, { ...file, status: "pending" }]);
}

export function removeFile(
  store: typeof imageFiles,
  index: number,
) {
  store.update((files) => files.filter((_, i) => i !== index));
}

export function clearFiles(store: typeof imageFiles) {
  store.set([]);
}
