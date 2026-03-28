<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";

  interface Props {
    onFilesSelected: (paths: string[]) => void;
    accept: string[];
    hint: string;
    compact?: boolean;
  }

  let { onFilesSelected, accept, hint, compact = false }: Props = $props();
  let isDragging = $state(false);

  onMount(() => {
    const unlisten = getCurrentWindow().onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        isDragging = true;
      } else if (event.payload.type === "leave") {
        isDragging = false;
      } else if (event.payload.type === "drop") {
        isDragging = false;
        const paths = event.payload.paths;
        if (paths.length > 0) onFilesSelected(paths);
      }
    });
    return () => { unlisten.then((fn) => fn()); };
  });

  async function handleClick() {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [{ name: "Media", extensions: accept }],
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      onFilesSelected(paths);
    }
  }

  async function handleClickFolder() {
    const selected = await open({
      multiple: true,
      directory: true,
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      onFilesSelected(paths);
    }
  }
</script>

{#if compact}
  <div class="compact-group" class:dragging={isDragging}>
    <button class="dropzone-compact" onclick={handleClick} type="button">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M12 5v14M5 12h14" />
      </svg>
      <span>添加文件</span>
    </button>
    <button class="dropzone-compact" onclick={handleClickFolder} type="button" title="选择文件夹">
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M22 19a2 2 0 01-2 2H4a2 2 0 01-2-2V5a2 2 0 012-2h5l2 3h9a2 2 0 012 2z"/>
      </svg>
    </button>
  </div>
{:else}
  <button class="dropzone" class:dragging={isDragging} onclick={handleClick} type="button">
    <div class="dropzone-content">
      <svg width="28" height="28" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" class="dropzone-icon">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4M17 8l-5-5-5 5M12 3v12" />
      </svg>
      <p class="dropzone-text">拖拽文件或文件夹到此处</p>
      <p class="dropzone-actions">
        <span class="action-link" role="button" tabindex="0"
          onclick={(e: MouseEvent) => { e.stopPropagation(); handleClick(); }}>选择文件</span>
        <span class="action-sep">|</span>
        <span class="action-link" role="button" tabindex="0"
          onclick={(e: MouseEvent) => { e.stopPropagation(); handleClickFolder(); }}>选择文件夹</span>
      </p>
      <p class="dropzone-hint">{hint}</p>
    </div>
  </button>
{/if}

<style>
  /* Full drop zone (no files) */
  .dropzone {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 100%;
    flex: 1;
    min-height: 120px;
    border: 1.5px dashed var(--color-border);
    border-radius: 6px;
    background: transparent;
    cursor: pointer;
    transition: all 0.2s;
    padding: 20px;
  }

  .dropzone:hover {
    border-color: var(--color-accent);
    background-color: color-mix(in srgb, var(--color-accent) 5%, transparent);
  }

  .dropzone.dragging {
    border-color: var(--color-accent);
    background-color: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }

  .dropzone-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
  }

  .dropzone-icon { color: var(--color-text-muted); }
  .dropzone-text { margin: 0; font-size: 12px; color: var(--color-text-secondary); }
  .dropzone-actions { margin: 0; font-size: 11px; display: flex; gap: 6px; align-items: center; }
  .action-link { color: var(--color-accent); cursor: pointer; }
  .action-link:hover { text-decoration: underline; }
  .action-sep { color: var(--color-text-muted); font-size: 10px; }
  .dropzone-hint { margin: 0; font-size: 10px; color: var(--color-text-muted); }

  /* Compact drop zone (has files) */
  .dropzone-compact {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 5px 10px;
    border: 1px dashed var(--color-border);
    border-radius: 5px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
    flex-shrink: 0;
  }

  .dropzone-compact:hover {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .dropzone-compact.dragging {
    border-color: var(--color-accent);
    background-color: color-mix(in srgb, var(--color-accent) 8%, transparent);
    color: var(--color-accent);
  }

  .compact-hint {
    color: var(--color-text-muted);
    opacity: 0.6;
    margin-left: 2px;
  }

  .compact-group {
    display: flex;
    gap: 4px;
    flex-shrink: 0;
  }
  .compact-group.dragging .dropzone-compact {
    border-color: var(--color-accent);
    color: var(--color-accent);
  }
</style>
