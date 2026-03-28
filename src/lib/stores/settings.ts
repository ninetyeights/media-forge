import { writable } from "svelte/store";
import { load } from "@tauri-apps/plugin-store";

export interface ImageSettings {
  preset: string;
  compressMode: "off" | "quality" | "target";
  quality: number;
  targetSize: number;
  outputFormat: string;
  resizeWidth: number | null;
  resizeHeight: number | null;
  keepAspectRatio: boolean;
  minSize: number | null;
  maxSize: number | null;
  hideFiltered: boolean;
  outputDir: string;
  outputMode: "overwrite" | "saveto";
  preserveStructure: boolean;
  fixExtension: boolean;
  jpegExtension: "jpg" | "jpeg";
  stripExif: boolean;
}

export interface ImagePreset {
  id: string;
  label: string;
  quality: number;
  resizeWidth: number | null;
  targetSize: number | null;
}

export const imagePresets: ImagePreset[] = [
  { id: "lossless",   label: "无损",     quality: 100, resizeWidth: null, targetSize: null },
  { id: "light",      label: "轻压缩",   quality: 85,  resizeWidth: null, targetSize: null },
  { id: "standard",   label: "标准压缩",  quality: 70,  resizeWidth: null, targetSize: null },
  { id: "max",        label: "极限压缩",  quality: 45,  resizeWidth: null, targetSize: null },
  { id: "under1m",    label: "≤1MB",     quality: 80,  resizeWidth: null, targetSize: 1 },
  { id: "under500k",  label: "≤500KB",   quality: 70,  resizeWidth: null, targetSize: 0.5 },
  { id: "web",        label: "Web优化",   quality: 80,  resizeWidth: 1920, targetSize: null },
];

export interface VideoSettings {
  compressEnabled: boolean;
  crf: number;
  codec: string;
  convertEnabled: boolean;
  outputFormat: string;
  resizeEnabled: boolean;
  resolution: string;
  customWidth: number | null;
  customHeight: number | null;
  outputDir: string;
}

export interface AudioSettings {
  compressEnabled: boolean;
  bitrate: string;
  convertEnabled: boolean;
  outputFormat: string;
  sampleRateEnabled: boolean;
  sampleRate: string;
  outputDir: string;
}

const IMAGE_DEFAULTS: ImageSettings = {
  preset: "lossless",
  compressMode: "off",
  quality: 80,
  targetSize: 1,
  outputFormat: "original",
  resizeWidth: null,
  resizeHeight: null,
  keepAspectRatio: true,
  minSize: null,
  maxSize: null,
  hideFiltered: false,
  outputDir: "",
  outputMode: "overwrite",
  preserveStructure: true,
  fixExtension: true,
  jpegExtension: "jpg",
  stripExif: false,
};

const VIDEO_DEFAULTS: VideoSettings = {
  compressEnabled: true,
  crf: 23,
  codec: "h264",
  convertEnabled: false,
  outputFormat: "mp4",
  resizeEnabled: false,
  resolution: "original",
  customWidth: null,
  customHeight: null,
  outputDir: "",
};

const AUDIO_DEFAULTS: AudioSettings = {
  compressEnabled: true,
  bitrate: "192",
  convertEnabled: false,
  outputFormat: "mp3",
  sampleRateEnabled: false,
  sampleRate: "44100",
  outputDir: "",
};

export interface AppSettings {
  sizeBase: 1024 | 1000;
  concurrency: number;
  notificationsEnabled: boolean;
  autoStart: boolean;
  defaultOutputDir: string;
}

const APP_DEFAULTS: AppSettings = {
  sizeBase: 1024,
  concurrency: 4,
  notificationsEnabled: true,
  autoStart: false,
  defaultOutputDir: "",
};

export const appSettings = writable<AppSettings>(APP_DEFAULTS);
export const imageSettings = writable<ImageSettings>(IMAGE_DEFAULTS);
export const videoSettings = writable<VideoSettings>(VIDEO_DEFAULTS);
export const audioSettings = writable<AudioSettings>(AUDIO_DEFAULTS);

// --- Persistence ---

const STORE_FILE = "settings.json";
let storeReady = false;

export async function loadSettings(): Promise<void> {
  try {
    const store = await load(STORE_FILE);
    const app = await store.get<AppSettings>("app");
    if (app) appSettings.set({ ...APP_DEFAULTS, ...app });
    const img = await store.get<ImageSettings>("image");
    if (img) imageSettings.set({ ...IMAGE_DEFAULTS, ...img });
    const vid = await store.get<VideoSettings>("video");
    if (vid) videoSettings.set({ ...VIDEO_DEFAULTS, ...vid });
    const aud = await store.get<AudioSettings>("audio");
    if (aud) audioSettings.set({ ...AUDIO_DEFAULTS, ...aud });
  } catch (_) {}

  storeReady = true;

  appSettings.subscribe((v) => { if (storeReady) saveKey("app", v); });
  imageSettings.subscribe((v) => { if (storeReady) saveKey("image", v); });
  videoSettings.subscribe((v) => { if (storeReady) saveKey("video", v); });
  audioSettings.subscribe((v) => { if (storeReady) saveKey("audio", v); });
}

let saveTimer: ReturnType<typeof setTimeout> | null = null;
let pendingSaves: Record<string, unknown> = {};

function saveKey(key: string, value: unknown) {
  pendingSaves[key] = value;
  if (saveTimer) clearTimeout(saveTimer);
  saveTimer = setTimeout(flushSaves, 300);
}

async function flushSaves() {
  const batch = pendingSaves;
  pendingSaves = {};
  try {
    const store = await load(STORE_FILE);
    for (const [key, value] of Object.entries(batch)) {
      await store.set(key, value);
    }
    await store.save();
  } catch (_) {}
}
