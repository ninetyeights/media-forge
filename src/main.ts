import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { check } from "@tauri-apps/plugin-updater";
import { initTraySync } from "$lib/tray";
import { loadSettings, appSettings } from "$lib/stores/settings";
import { hasUpdate } from "$lib/stores/navigation";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, { target: document.getElementById("app")! });

getCurrentWindow().show().catch(console.error);
loadSettings().catch(console.error);
initTraySync();

// Auto check for updates silently
check().then((update) => {
  if (update) hasUpdate.set(true);
}).catch(() => {});

// Apply UI scale
appSettings.subscribe((s) => {
  document.documentElement.style.zoom = `${s.uiScale || 100}%`;
});

export default app;
