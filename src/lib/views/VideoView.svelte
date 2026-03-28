<script lang="ts">
  import { videoSettings } from "$lib/stores/settings";
  import SegmentControl from "$lib/components/SegmentControl.svelte";
  import FileDropZone from "$lib/components/FileDropZone.svelte";

  const videoExts = ["mp4", "webm", "mov", "avi", "mkv", "flv", "wmv"];
  const codecOptions = [
    { value: "h264", label: "H.264" },
    { value: "h265", label: "H.265" },
    { value: "vp9", label: "VP9" },
    { value: "av1", label: "AV1" },
  ];
  const formatOptions = [
    { value: "mp4", label: "MP4" },
    { value: "webm", label: "WEBM" },
    { value: "mov", label: "MOV" },
    { value: "avi", label: "AVI" },
    { value: "mkv", label: "MKV" },
  ];
  const resolutionOptions = [
    { value: "original", label: "原始" },
    { value: "2160", label: "4K" },
    { value: "1080", label: "1080p" },
    { value: "720", label: "720p" },
    { value: "480", label: "480p" },
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
      accept={videoExts}
      hint="支持 MP4, WEBM, MOV, AVI, MKV, FLV, WMV"
    />
  {:else}
    <div class="toolbar">
      <FileDropZone onFilesSelected={handleFilesSelected} accept={videoExts} hint="" compact />
      <div class="toolbar-sep"></div>
      <SegmentControl options={formatOptions} selected={$videoSettings.outputFormat}
        onchange={(v) => videoSettings.update((s) => ({ ...s, outputFormat: v }))} />
      <div class="toolbar-sep"></div>
      <SegmentControl options={codecOptions} selected={$videoSettings.codec}
        onchange={(v) => videoSettings.update((s) => ({ ...s, codec: v }))} />
    </div>
    <div class="toolbar-secondary">
      <span class="field-label">CRF</span>
      <input type="range" class="slider" min="0" max="51" value={$videoSettings.crf}
        oninput={(e) => videoSettings.update((s) => ({ ...s, crf: Number((e.target as HTMLInputElement).value) }))} />
      <span class="field-value">{$videoSettings.crf}</span>
      <div class="toolbar-sep"></div>
      <span class="field-label">分辨率</span>
      <SegmentControl options={resolutionOptions} selected={$videoSettings.resolution}
        onchange={(v) => videoSettings.update((s) => ({ ...s, resolution: v }))} />
    </div>
    <div class="empty-list">
      <span>需要安装 FFmpeg 支持，后续版本实现</span>
    </div>
  {/if}
</div>

<style>
  .view { flex: 1; display: flex; flex-direction: column; padding: 12px; overflow: hidden; }

  .toolbar { display: flex; align-items: center; gap: 8px; padding: 6px 0; flex-shrink: 0; }
  .toolbar-secondary { display: flex; align-items: center; gap: 8px; padding: 6px 0 8px; border-bottom: 0.5px solid var(--color-border); flex-shrink: 0; }
  .toolbar-sep { width: 1px; height: 18px; background-color: var(--color-border); flex-shrink: 0; }
  .field-label { font-size: 11px; color: var(--color-text-muted); flex-shrink: 0; }

  .slider { width: 100px; height: 3px; appearance: none; background: var(--color-bg-hover); border-radius: 2px; outline: none; cursor: pointer; }
  .slider::-webkit-slider-thumb { appearance: none; width: 12px; height: 12px; border-radius: 50%; background: var(--color-accent); cursor: pointer; }
  .field-value { font-size: 11px; color: var(--color-accent); font-weight: 600; min-width: 20px; text-align: right; font-variant-numeric: tabular-nums; }

  .empty-list { flex: 1; display: flex; align-items: center; justify-content: center; color: var(--color-text-muted); font-size: 12px; }
</style>
