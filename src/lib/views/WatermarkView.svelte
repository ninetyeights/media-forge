<script lang="ts">
  import { watermarkFiles } from "$lib/stores/fileQueue";
  import SegmentControl from "$lib/components/SegmentControl.svelte";
  import SliderControl from "$lib/components/SliderControl.svelte";
  import FileDropZone from "$lib/components/FileDropZone.svelte";
  import FileListItem from "$lib/components/FileListItem.svelte";
  import { removeFile } from "$lib/stores/fileQueue";

  const allExts = ["jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff", "mp4", "webm", "mov"];
  const typeOptions = [
    { value: "text", label: "文字水印" },
    { value: "image", label: "图片水印" },
  ];
  const positionOptions = [
    { value: "top-left", label: "左上" },
    { value: "top-center", label: "中上" },
    { value: "top-right", label: "右上" },
    { value: "center-left", label: "左中" },
    { value: "center", label: "居中" },
    { value: "center-right", label: "右中" },
    { value: "bottom-left", label: "左下" },
    { value: "bottom-center", label: "中下" },
    { value: "bottom-right", label: "右下" },
  ];

  let watermarkType = $state("text");
  let watermarkText = $state("MediaForge");
  let opacity = $state(50);
  let position = $state("bottom-right");
  let fontSize = $state(24);
  let tiled = $state(false);

  function handleFilesSelected(paths: string[]) {
    // TODO: add files to watermark queue
  }
</script>

<div class="view-container">
  <div class="file-area">
    <div class="file-area-header">
      <h2 class="view-title">水印工具</h2>
    </div>
    <FileDropZone
      onFilesSelected={handleFilesSelected}
      accept={allExts}
      hint="支持图片和视频文件"
    />
    {#if $watermarkFiles.length > 0}
      <div class="file-list">
        {#each $watermarkFiles as file, i (file.id)}
          <FileListItem
            {file}
            onRemove={() => removeFile(watermarkFiles, i)}
          />
        {/each}
      </div>
    {/if}
    <div class="placeholder">水印功能后续版本实现</div>
  </div>

  <div class="settings-panel">
    <div class="settings-section">
      <span class="section-title">水印类型</span>
      <SegmentControl
        options={typeOptions}
        selected={watermarkType}
        onchange={(v) => watermarkType = v}
      />
    </div>

    <div class="settings-divider"></div>

    {#if watermarkType === "text"}
      <div class="settings-section">
        <span class="section-title">水印文字</span>
        <input
          type="text"
          class="text-input"
          bind:value={watermarkText}
          placeholder="输入水印文字"
        />
        <SliderControl label="字号" value={fontSize} min={8} max={72} unit="px" onchange={(v) => fontSize = v} />
      </div>
    {:else}
      <div class="settings-section">
        <span class="section-title">水印图片</span>
        <button class="btn-secondary">选择图片</button>
      </div>
    {/if}

    <div class="settings-divider"></div>

    <div class="settings-section">
      <SliderControl label="透明度" value={opacity} min={0} max={100} unit="%" onchange={(v) => opacity = v} />
    </div>

    <div class="settings-divider"></div>

    <div class="settings-section">
      <span class="section-title">位置</span>
      <div class="position-grid">
        {#each positionOptions as pos}
          <button
            class="pos-btn"
            class:active={position === pos.value}
            onclick={() => position = pos.value}
          >{pos.label}</button>
        {/each}
      </div>
      <label class="checkbox-label">
        <input type="checkbox" bind:checked={tiled} />
        <span>平铺</span>
      </label>
    </div>
  </div>
</div>

<style>
  .view-container { display: flex; flex: 1; overflow: hidden; }
  .file-area { flex: 1; display: flex; flex-direction: column; gap: 10px; padding: 14px; overflow-y: auto; }
  .file-area-header { display: flex; align-items: center; justify-content: space-between; }
  .view-title { margin: 0; font-size: 14px; font-weight: 600; color: var(--color-text-primary); }
  .file-list { border: 0.5px solid var(--color-border); border-radius: 6px; overflow: hidden; }
  .settings-panel { width: 200px; border-left: 0.5px solid var(--color-border); padding: 14px; overflow-y: auto; flex-shrink: 0; }
  .settings-section { display: flex; flex-direction: column; gap: 8px; }
  .section-title { font-size: 12px; font-weight: 500; color: var(--color-text-primary); }
  .settings-divider { height: 0.5px; background-color: var(--color-border); margin: 10px 0; }
  .placeholder { font-size: 12px; color: var(--color-text-muted); text-align: center; padding: 20px; }

  .text-input {
    width: 100%;
    padding: 4px 6px;
    background: var(--color-bg-surface);
    border: 0.5px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-primary);
    font-size: 12px;
    outline: none;
  }
  .text-input:focus { border-color: var(--color-accent); }

  .btn-secondary {
    padding: 4px 8px;
    background: var(--color-bg-surface);
    border: 0.5px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-secondary);
    font-size: 11px;
    cursor: pointer;
  }
  .btn-secondary:hover { border-color: var(--color-accent); color: var(--color-accent); }

  .position-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 3px;
  }

  .pos-btn {
    padding: 3px 2px;
    border: 0.5px solid var(--color-border);
    background: var(--color-bg-surface);
    color: var(--color-text-muted);
    font-size: 10px;
    border-radius: 3px;
    cursor: pointer;
    transition: all 0.15s;
  }
  .pos-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
  .pos-btn.active { background-color: var(--color-accent); border-color: var(--color-accent); color: var(--color-bg-primary); }

  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--color-text-secondary);
    cursor: pointer;
  }
  .checkbox-label input[type="checkbox"] { width: 13px; height: 13px; accent-color: var(--color-accent); }
</style>
