<script lang="ts">
  import type { MediaFile } from "$lib/types";
  import { formatSize, formatRatio } from "$lib/utils/fileSize";
  import { getFormatColor } from "$lib/utils/fileType";
  import { convertFileSrc } from "@tauri-apps/api/core";

  interface Props {
    file: MediaFile;
    onRemove: () => void;
    onWatermark?: () => void;
    onReset?: () => void;
    dimmed?: boolean;
    selected?: boolean;
    onToggleSelect?: () => void;
  }

  let { file, onRemove, onWatermark, onReset, dimmed = false, selected, onToggleSelect }: Props = $props();

  let showFullscreen = $state(false);

  let outputWidth = $state(0);
  let outputHeight = $state(0);
  let sliderValue = $state(50);

  function imgSrc(path: string): string {
    return convertFileSrc(path);
  }

  function onOutputLoad(e: Event) {
    const img = e.target as HTMLImageElement;
    outputWidth = img.naturalWidth;
    outputHeight = img.naturalHeight;
  }

  function openPreview() {
    showFullscreen = true;
    sliderValue = 50;
    document.addEventListener("keydown", onEsc);
  }

  function closePreview() {
    showFullscreen = false;
    document.removeEventListener("keydown", onEsc);
  }

  function onEsc(e: KeyboardEvent) {
    if (e.key === "Escape") closePreview();
  }
</script>

<div class="file-item" class:processing={file.status === "processing"} class:dimmed>
  {#if onToggleSelect}
    <input type="checkbox" class="file-check" checked={selected} onchange={onToggleSelect} />
  {/if}
  <span class="format-tag" style="background-color: {getFormatColor(file.format)}">
    {file.format.toUpperCase()}
  </span>

  <button class="file-name-btn" title={file.path} onclick={openPreview}>
    {file.name}
  </button>

  <span class="file-size">
    {formatSize(file.originalSize)}
    {#if file.status === "done" && file.compressedSize !== undefined}
      <span class="arrow">→</span>
      <span class="compressed-size">{formatSize(file.compressedSize)}</span>
      {#if file.ratio !== undefined && file.ratio > 0}
        <span class="ratio saved">-{formatRatio(file.ratio)}</span>
      {:else if file.ratio !== undefined}
        <span class="ratio grew">+{formatRatio(Math.abs(file.ratio))}</span>
      {/if}
    {/if}
  </span>

  <span class="file-status">
    {#if file.status === "pending"}
      <span class="badge pending">等待中</span>
    {:else if file.status === "processing"}
      <span class="badge processing">
        {#if file.progress !== undefined}
          {file.progress}%
        {:else}
          处理中
        {/if}
      </span>
    {:else if file.status === "done"}
      <span class="badge done" class:warn={file.error} title={file.error || ""}>{file.error ? "提示" : "完成"}</span>
    {:else if file.status === "skipped"}
      <span class="badge skipped">跳过</span>
    {:else if file.status === "error"}
      <span class="badge error" title={file.error}>失败</span>
    {/if}
  </span>

  <span class="file-actions">
    {#if (file.status === "done" || file.status === "skipped" || file.status === "error") && onReset}
      <button class="action-btn reset-btn" onclick={onReset} title="重新处理">
        <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M1 4v6h6"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
        </svg>
      </button>
    {/if}
    {#if file.status === "done" && onWatermark}
      <button class="action-btn watermark-btn" onclick={onWatermark} title="添加水印">
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12 2l3 6.3L22 9.3l-5 4.8L18.2 21 12 17.8 5.8 21 7 14.1l-5-4.8 6.9-1z"/>
        </svg>
      </button>
    {/if}
    <button class="action-btn preview-btn" class:active={showFullscreen} onclick={openPreview} title="预览">
      <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/>
      </svg>
    </button>
    <button class="action-btn remove-btn" onclick={onRemove} title="移除">
      <svg width="10" height="10" viewBox="0 0 10 10">
        <path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/>
      </svg>
    </button>
  </span>
</div>

{#if file.status === "processing" && file.progress !== undefined}
  <div class="progress-bar">
    <div class="progress-fill" style="width: {file.progress}%"></div>
  </div>
{/if}

{#if showFullscreen}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="fullscreen-overlay" onclick={closePreview}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fullscreen-content" onclick={(e) => e.stopPropagation()}>
      <button class="fullscreen-close" onclick={closePreview}>
        <svg width="16" height="16" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
      </button>

      {#if file.status === "done" && file.outputPath}
        <div class="compare-container" style="--pos: {sliderValue}%">
          <div class="compare-wrap">
            <img src={imgSrc(file.path)} alt="original" class="compare-img" />
            <img src={imgSrc(file.outputPath)} alt="processed" class="compare-img compare-top" onload={onOutputLoad} />
          </div>
          <input type="range" class="compare-range" min="0" max="100"
            bind:value={sliderValue}
          />
          <div class="compare-line"></div>
          <div class="compare-handle">
            <svg width="12" height="12" viewBox="0 0 12 12" fill="none">
              <path d="M3 6l2-3v6l-2-3zM9 6l-2-3v6l2-3z" fill="currentColor"/>
            </svg>
          </div>
          <span class="compare-label compare-label-left">原图</span>
          <span class="compare-label compare-label-right">处理后</span>
        </div>
        <div class="preview-info">
          <span class="preview-meta">
            {formatSize(file.originalSize)}
            {#if file.width && file.height} · {file.width}×{file.height}{/if}
          </span>
          <span class="preview-meta">→</span>
          <span class="preview-meta">
            {#if file.compressedSize !== undefined}{formatSize(file.compressedSize)}{/if}
            {#if outputWidth && outputHeight} · {outputWidth}×{outputHeight}{/if}
            {#if file.ratio !== undefined && file.ratio !== 0}
              <span class="preview-ratio" class:saved={file.ratio > 0} class:grew={file.ratio <= 0}>
                {file.ratio > 0 ? "-" : "+"}{formatRatio(Math.abs(file.ratio))}
              </span>
            {/if}
          </span>
        </div>
      {:else}
        <div class="single-preview">
          <img src={imgSrc(file.path)} alt={file.name} class="single-img" />
        </div>
        <div class="preview-info">
          <span class="preview-meta">
            {formatSize(file.originalSize)}
            {#if file.width && file.height} · {file.width}×{file.height}{/if}
          </span>
        </div>
      {/if}

      <div class="fullscreen-filename">{file.name}</div>
    </div>
  </div>
{/if}

<style>
  .file-item {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 10px;
    border-bottom: 0.5px solid var(--color-border);
    font-size: 12px;
    transition: background-color 0.1s;
  }

  .file-item:hover { background-color: var(--color-bg-surface); }
  .file-item:last-child { border-bottom: none; }
  .file-item.dimmed { opacity: 0.35; }

  .file-check {
    margin: 0;
    flex-shrink: 0;
    cursor: pointer;
  }

  .format-tag {
    font-size: 9px;
    font-weight: 600;
    color: white;
    padding: 1px 4px;
    border-radius: 3px;
    flex-shrink: 0;
    text-transform: uppercase;
    letter-spacing: 0.3px;
  }

  .file-name-btn {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text-primary);
    background: none;
    border: none;
    padding: 0;
    font: inherit;
    cursor: pointer;
    text-align: left;
    outline: none;
  }
  .file-name-btn:hover { color: var(--color-accent); }

  .file-size {
    color: var(--color-text-secondary);
    font-size: 11px;
    white-space: nowrap;
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .arrow { color: var(--color-text-muted); }
  .compressed-size { color: var(--color-text-primary); }
  .ratio { font-size: 10px; font-weight: 600; }
  .ratio.saved { color: var(--color-success); }
  .ratio.grew { color: var(--color-warning); }

  .file-status { flex-shrink: 0; }

  .badge {
    display: inline-block;
    padding: 1px 6px;
    border-radius: 3px;
    font-size: 10px;
    font-weight: 500;
  }

  .badge.pending { color: var(--color-text-muted); background-color: var(--color-bg-surface); }
  .badge.processing { color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 15%, transparent); }
  .badge.done { color: var(--color-success); background-color: color-mix(in srgb, var(--color-success) 15%, transparent); }
  .badge.done.warn { color: var(--color-warning); background-color: color-mix(in srgb, var(--color-warning) 15%, transparent); }
  .badge.skipped { color: var(--color-text-muted); background-color: var(--color-bg-surface); }
  .badge.error { color: var(--color-error); background-color: color-mix(in srgb, var(--color-error) 15%, transparent); }

  .file-actions {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }

  .action-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s;
    outline: none;
  }

  .remove-btn:hover { color: var(--color-error); background-color: color-mix(in srgb, var(--color-error) 15%, transparent); }
  .reset-btn:hover { color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 15%, transparent); }
  .watermark-btn:hover { color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 15%, transparent); }
  .preview-btn:hover { color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 15%, transparent); }
  .preview-btn.active { color: var(--color-accent); }

  .progress-bar {
    height: 2px;
    background: var(--color-bg-hover);
  }

  .progress-fill {
    height: 100%;
    background: var(--color-accent);
    transition: width 0.15s;
  }

  /* Fullscreen overlay */
  .fullscreen-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(0, 0, 0, 0.85);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(8px);
    -webkit-backdrop-filter: blur(8px);
  }

  .fullscreen-content {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    max-width: 90vw;
    max-height: 90vh;
  }

  .fullscreen-close {
    position: absolute;
    top: -30px;
    right: 0;
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.6);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    outline: none;
    transition: color 0.15s;
    z-index: 1;
  }
  .fullscreen-close:hover { color: white; }

  .fullscreen-filename {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.5);
  }

  /* Comparison slider */
  .compare-container {
    position: relative;
    display: inline-block;
    border-radius: 6px;
    overflow: hidden;
    background: #111;
  }

  .compare-wrap {
    position: relative;
    display: block;
    line-height: 0;
  }

  .compare-img {
    max-width: 90vw;
    max-height: 75vh;
    display: block;
  }

  .compare-top {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    object-position: right;
    clip-path: inset(0 0 0 var(--pos));
  }

  .compare-range {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    opacity: 0;
    cursor: col-resize;
    z-index: 3;
  }

  .compare-line {
    position: absolute;
    top: 0;
    bottom: 0;
    left: var(--pos);
    width: 2px;
    margin-left: -1px;
    background: white;
    box-shadow: 0 0 6px rgba(0, 0, 0, 0.6);
    pointer-events: none;
    z-index: 2;
  }

  .compare-handle {
    position: absolute;
    top: 50%;
    left: var(--pos);
    transform: translate(-50%, -50%);
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: white;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    color: #333;
    pointer-events: none;
    z-index: 2;
  }

  .compare-label {
    position: absolute;
    top: 8px;
    font-size: 11px;
    font-weight: 500;
    padding: 2px 8px;
    border-radius: 4px;
    background: rgba(0, 0, 0, 0.6);
    color: white;
    pointer-events: none;
    z-index: 1;
  }
  .compare-label-left { left: 8px; }
  .compare-label-right { right: 8px; }

  .single-preview {
    display: inline-block;
    border-radius: 6px;
    overflow: hidden;
    background: #111;
    line-height: 0;
  }
  .single-img {
    max-width: 90vw;
    max-height: 75vh;
    display: block;
  }

  .preview-info {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .preview-meta {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.6);
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .preview-ratio { font-weight: 600; }
  .preview-ratio.saved { color: var(--color-success); }
  .preview-ratio.grew { color: var(--color-warning); }
</style>
