<script lang="ts">
  import { appSettings } from "$lib/stores/settings";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount } from "svelte";
  import SegmentControl from "$lib/components/SegmentControl.svelte";

  let autoStartEnabled = $state(false);

  onMount(async () => {
    try {
      autoStartEnabled = await isEnabled();
      // Sync store with actual system state
      if (autoStartEnabled !== $appSettings.autoStart) {
        appSettings.update((s) => ({ ...s, autoStart: autoStartEnabled }));
      }
    } catch (_) {}
  });

  async function toggleAutoStart() {
    try {
      if (autoStartEnabled) {
        await disable();
        autoStartEnabled = false;
      } else {
        await enable();
        autoStartEnabled = true;
      }
      appSettings.update((s) => ({ ...s, autoStart: autoStartEnabled }));
    } catch (_) {}
  }

  async function pickDefaultOutputDir() {
    const dir = await open({ directory: true, multiple: false });
    if (dir && typeof dir === "string") {
      appSettings.update((s) => ({ ...s, defaultOutputDir: dir }));
    }
  }
</script>

<div class="view">
  <h2 class="view-title">设置</h2>

  <div class="settings-group">
    <span class="group-label">通用</span>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">并发处理数</span>
        <span class="setting-desc">同时处理的文件数量，越大速度越快但占用更多资源</span>
      </div>
      <div class="setting-control">
        <input type="range" class="slider" min="1" max="16"
          value={$appSettings.concurrency}
          oninput={(e) => appSettings.update((s) => ({ ...s, concurrency: Number((e.target as HTMLInputElement).value) }))}
        />
        <span class="slider-value">{$appSettings.concurrency}</span>
      </div>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">文件大小单位</span>
        <span class="setting-desc">1024 与 Windows 一致，1000 与 macOS 一致</span>
      </div>
      <SegmentControl
        options={[
          { value: "1024", label: "1024" },
          { value: "1000", label: "1000" },
        ]}
        selected={String($appSettings.sizeBase)}
        onchange={(v) => appSettings.update((s) => ({ ...s, sizeBase: Number(v) as 1024 | 1000 }))}
      />
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">默认输出目录</span>
        <span class="setting-desc">新建任务时的默认另存到目录</span>
      </div>
      <div class="setting-control">
        <button class="path-btn" onclick={pickDefaultOutputDir} title={$appSettings.defaultOutputDir || "点击选择"}>
          {$appSettings.defaultOutputDir ? $appSettings.defaultOutputDir.split("/").pop() : "未设置"}
        </button>
        {#if $appSettings.defaultOutputDir}
          <button class="clear-btn" onclick={() => appSettings.update((s) => ({ ...s, defaultOutputDir: "" }))} title="清除">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
          </button>
        {/if}
      </div>
    </div>
  </div>

  <div class="settings-group">
    <span class="group-label">系统</span>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">系统通知</span>
        <span class="setting-desc">后台处理完成时发送系统通知</span>
      </div>
      <label class="toggle">
        <input type="checkbox" checked={$appSettings.notificationsEnabled}
          onchange={(e) => appSettings.update((s) => ({ ...s, notificationsEnabled: (e.target as HTMLInputElement).checked }))} />
        <span class="toggle-track"></span>
      </label>
    </div>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">开机自启</span>
        <span class="setting-desc">系统登录时自动启动 Media Forge</span>
      </div>
      <label class="toggle">
        <input type="checkbox" checked={autoStartEnabled} onchange={toggleAutoStart} />
        <span class="toggle-track"></span>
      </label>
    </div>
  </div>
</div>

<style>
  .view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 14px;
    gap: 20px;
    overflow-y: auto;
  }

  .view-title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .settings-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .group-label {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    padding: 0 12px 6px;
  }

  .setting-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    padding: 10px 12px;
    border-radius: 6px;
    background: var(--color-bg-surface);
  }

  .setting-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .setting-label {
    font-size: 12px;
    color: var(--color-text-primary);
  }

  .setting-desc {
    font-size: 10px;
    color: var(--color-text-muted);
  }

  .setting-control {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .slider {
    width: 100px;
    height: 3px;
    appearance: none;
    background: var(--color-bg-hover);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .slider::-webkit-slider-thumb {
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
  }

  .slider-value {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-accent);
    min-width: 20px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }

  .path-btn {
    padding: 3px 10px;
    background: var(--color-bg-hover);
    border: 0.5px solid var(--color-border);
    border-radius: 4px;
    color: var(--color-text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
    max-width: 160px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    outline: none;
  }
  .path-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }

  .clear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 3px;
    transition: all 0.15s;
    outline: none;
  }
  .clear-btn:hover { background: var(--color-bg-hover); color: var(--color-error); }

  /* Toggle switch */
  .toggle {
    position: relative;
    display: inline-flex;
    cursor: pointer;
    flex-shrink: 0;
  }
  .toggle input { display: none; }
  .toggle-track {
    width: 34px;
    height: 20px;
    border-radius: 10px;
    background: var(--color-bg-hover);
    border: 1px solid var(--color-border);
    position: relative;
    transition: all 0.2s;
  }
  .toggle-track::after {
    content: "";
    position: absolute;
    left: 2px;
    top: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-text-muted);
    transition: all 0.2s;
  }
  .toggle input:checked + .toggle-track {
    background: color-mix(in srgb, var(--color-accent) 30%, transparent);
    border-color: var(--color-accent);
  }
  .toggle input:checked + .toggle-track::after {
    left: 16px;
    background: var(--color-accent);
  }
</style>
