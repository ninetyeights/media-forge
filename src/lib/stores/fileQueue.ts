import { writable } from "svelte/store";
import type { MediaFile } from "$lib/types";

export const imageFiles = writable<MediaFile[]>([]);

export const isProcessing = writable(false);
export const imageSelectedIds = writable<Set<string>>(new Set());
export const imageBatchElapsed = writable(0);

export function removeFile(
  store: typeof imageFiles,
  index: number,
) {
  store.update((files) => files.filter((_, i) => i !== index));
}

export function clearFiles(store: typeof imageFiles) {
  store.set([]);
}
