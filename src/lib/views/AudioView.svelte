<script lang="ts">
  import { audioSettings } from "$lib/stores/settings";
  import SegmentControl from "$lib/components/SegmentControl.svelte";
  import FileDropZone from "$lib/components/FileDropZone.svelte";

  const audioExts = ["mp3", "aac", "ogg", "wav", "flac", "m4a", "wma"];
  const bitrateOptions = [
    { value: "320", label: "320k" },
    { value: "256", label: "256k" },
    { value: "192", label: "192k" },
    { value: "128", label: "128k" },
    { value: "64", label: "64k" },
  ];
  const formatOptions = [
    { value: "mp3", label: "MP3" },
    { value: "aac", label: "AAC" },
    { value: "ogg", label: "OGG" },
    { value: "wav", label: "WAV" },
    { value: "flac", label: "FLAC" },
    { value: "m4a", label: "M4A" },
  ];
  const sampleRateOptions = [
    { value: "original", label: "原始" },
    { value: "48000", label: "48kHz" },
    { value: "44100", label: "44.1kHz" },
    { value: "22050", label: "22kHz" },
  ];

  let hasFiles = false;

  function handleFilesSelected(_paths: string[]) {
    hasFiles = true;
  }
</script>

<div class="view">
  {#if !hasFiles}
    <FileDropZone
      onFilesSelected={handleFilesSelected}
      accept={audioExts}
      hint="支持 MP3, AAC, OGG, WAV, FLAC, M4A, WMA"
    />
  {:else}
    <div class="toolbar">
      <FileDropZone onFilesSelected={handleFilesSelected} accept={audioExts} hint="" compact />
      <div class="toolbar-sep"></div>
      <SegmentControl options={formatOptions} selected={$audioSettings.outputFormat}
        onchange={(v) => audioSettings.update((s) => ({ ...s, outputFormat: v }))} />
      <div class="toolbar-sep"></div>
      <span class="field-label">码率</span>
      <SegmentControl options={bitrateOptions} selected={$audioSettings.bitrate}
        onchange={(v) => audioSettings.update((s) => ({ ...s, bitrate: v }))} />
    </div>
    <div class="toolbar-secondary">
      <span class="field-label">采样率</span>
      <SegmentControl options={sampleRateOptions} selected={$audioSettings.sampleRate}
        onchange={(v) => audioSettings.update((s) => ({ ...s, sampleRate: v }))} />
    </div>
    <div class="empty-list">
      <span>需要安装 FFmpeg 支持，后续版本实现</span>
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: 12px; overflow: hidden; }

  .toolbar { display: flex; align-items: center; gap: 8px; padding: 6px 0; flex-shrink: 0; flex-wrap: wrap; }
  .toolbar-secondary { display: flex; align-items: center; gap: 8px; padding: 6px 0 8px; border-bottom: 0.5px solid var(--color-border); flex-shrink: 0; }
  .toolbar-sep { width: 1px; height: 18px; background-color: var(--color-border); flex-shrink: 0; }
  .field-label { font-size: 11px; color: var(--color-text-muted); flex-shrink: 0; }

  .empty-list { flex: 1; display: flex; align-items: center; justify-content: center; color: var(--color-text-muted); font-size: 12px; }
</style>
