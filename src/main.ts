import { mount } from "svelte";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { initTraySync } from "$lib/tray";
import { loadSettings } from "$lib/stores/settings";
import App from "./App.svelte";
import "./app.css";

const app = mount(App, { target: document.getElementById("app")! });

// Module scripts are deferred — DOM is already parsed when this runs
getCurrentWindow().show().catch(console.error);
loadSettings().catch(console.error);
initTraySync();

export default app;
