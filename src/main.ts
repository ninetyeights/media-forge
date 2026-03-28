import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { initTraySync } from "$lib/tray";
import { loadSettings, appSettings } from "$lib/stores/settings";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, { target: document.getElementById("app")! });

getCurrentWindow().show().catch(console.error);
loadSettings().catch(console.error);
initTraySync();

// Apply UI scale
appSettings.subscribe((s) => {
  document.documentElement.style.zoom = `${s.uiScale || 100}%`;
});

export default app;
