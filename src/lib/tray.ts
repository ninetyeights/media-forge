import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { watchFolders, type WatchFolder } from "$lib/stores/watcher";
import { get } from "svelte/store";

let syncTimer: ReturnType<typeof setTimeout> | null = null;

export function initTraySync(): void {
  watchFolders.subscribe(() => scheduleSyncTray());

  // Listen for per-folder toggle from tray menu
  listen<string>("tray:toggle-folder", async (event) => {
    const path = event.payload;
    const folders = get(watchFolders);
    const folder = folders.find((f) => f.path === path);
    if (!folder) return;

    if (folder.active) {
      await invoke("stop_folder_watcher", { path: folder.path });
      watchFolders.update((list) => list.map((f) => f.path === path ? { ...f, active: false } : f));
    } else {
      try {
        await invoke("start_folder_watcher", {
          folder: { path: folder.path, exts: folder.fileExts, recursive: folder.recursive },
        });
        watchFolders.update((list) => list.map((f) => f.path === path ? { ...f, active: true } : f));
      } catch (e) {
        console.error("Tray: failed to toggle watcher for", path, e);
      }
    }
  });
}

function scheduleSyncTray(): void {
  if (syncTimer) clearTimeout(syncTimer);
  syncTimer = setTimeout(syncTray, 50);
}

function syncTray(): void {
  const folders = get(watchFolders);

  const trayFolders = folders.map((f) => ({
    name: f.path.split("/").pop() || f.path,
    path: f.path,
    media_type: f.mediaType,
    auto_process: f.autoProcess,
    active: f.active,
  }));

  invoke("update_tray", { folders: trayFolders }).catch(() => {});
}
