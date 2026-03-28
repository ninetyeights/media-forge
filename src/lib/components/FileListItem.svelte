<script lang="ts">
  import type { MediaFile } from "$lib/types";
  import { formatSize, formatRatio } from "$lib/utils/fileSize";
  import { getFormatColor } from "$lib/utils/fileType";

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
</script>

<div class="file-item" class:processing={file.status === "processing"} class:dimmed>
  {#if onToggleSelect}
    <input type="checkbox" class="file-check" checked={selected} onchange={onToggleSelect} />
  {/if}
  <span class="format-tag" style="background-color: {getFormatColor(file.format)}">
    {file.format.toUpperCase()}
  </span>

  <span class="file-name" title={file.path}>{file.name}</span>

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
      <span class="badge done">完成</span>
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

  .file-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--color-text-primary);
  }

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
  }

  .remove-btn:hover { color: var(--color-error); background-color: color-mix(in srgb, var(--color-error) 15%, transparent); }
  .reset-btn:hover { color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 15%, transparent); }
  .watermark-btn:hover { color: var(--color-accent); background-color: color-mix(in srgb, var(--color-accent) 15%, transparent); }

  .progress-bar {
    height: 2px;
    background: var(--color-bg-hover);
  }

  .progress-fill {
    height: 100%;
    background: var(--color-accent);
    transition: width 0.15s;
  }
</style>
