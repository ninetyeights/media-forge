<script lang="ts">
  import { appSettings } from "$lib/stores/settings";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { open } from "@tauri-apps/plugin-dialog";
  import { readFile } from "@tauri-apps/plugin-fs";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { getVersion } from "@tauri-apps/api/app";
  import { onMount } from "svelte";
  import { hasUpdate } from "$lib/stores/navigation";
  import SegmentControl from "$lib/components/SegmentControl.svelte";

  const FEEDBACK_URL = "https://script.google.com/macros/s/AKfycbzbDbtsIFLSQqIbvxb4RN_s3rbNpKALkCxmGYPCpsoLPCFWAlw8vC5rEMBIpUmEFis_rg/exec";

  let autoStartEnabled = $state(false);
  let appVersion = $state("");

  // Feedback state
  let showFeedback = $state(false);
  let fbType = $state("bug");
  let fbDesc = $state("");
  let fbName = $state("");
  let fbContact = $state("");
  interface FbImage {
    base64: string;
    name: string;
    type: string;
  }
  let fbImages = $state<FbImage[]>([]);
  let fbSubmitting = $state(false);
  let fbResult = $state<"idle" | "success" | "error">("idle");
  let fbError = $state("");
  let updateStatus = $state<"idle" | "checking" | "available" | "downloading" | "ready" | "none" | "error">("idle");
  let updateVersion = $state("");
  let updateError = $state("");

  onMount(async () => {
    try {
      autoStartEnabled = await isEnabled();
      if (autoStartEnabled !== $appSettings.autoStart) {
        appSettings.update((s) => ({ ...s, autoStart: autoStartEnabled }));
      }
    } catch (_) {}
    try { appVersion = await getVersion(); } catch (_) {}
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

  async function checkForUpdate() {
    updateStatus = "checking";
    updateError = "";

    try {
      const update = await check();
      if (update) {
        updateVersion = update.version;
        updateStatus = "available";
      } else {
        updateStatus = "none";
      }
    } catch (e) {
      const msg = String(e);
      if (msg.includes("fallback platforms")) {
        updateError = "当前平台暂无可用更新包";
      } else if (msg.includes("fetch") || msg.includes("network") || msg.includes("reiease")) {
        updateError = "无法连接更新服务器，请检查网络";
      } else {
        updateError = msg;
      }
      updateStatus = "error";
    }
  }

  async function downloadAndInstall() {
    updateStatus = "downloading";
    hasUpdate.set(false);
    try {
      const update = await check();
      if (update) {
        await update.downloadAndInstall();
        updateStatus = "ready";
        await relaunch();
      }
    } catch (e) {
      updateError = String(e);
      updateStatus = "error";
    }
  }

  const mimeMap: Record<string, string> = { png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg", gif: "image/gif", webp: "image/webp" };

  async function pickScreenshot() {
    const paths = await open({
      multiple: true,
      filters: [{ name: "图片", extensions: ["png", "jpg", "jpeg", "gif", "webp"] }],
    });
    if (!paths) return;
    const list = Array.isArray(paths) ? paths : [paths];
    for (const p of list) {
      await addImageFromPath(p);
    }
  }

  const MAX_IMAGES = 3;

  async function addImageFromPath(path: string) {
    if (fbImages.length >= MAX_IMAGES) return;
    const data = await readFile(path);
    const base64 = arrayToBase64(data);
    const ext = path.split(".").pop()?.toLowerCase() || "png";
    fbImages = [...fbImages, {
      base64,
      name: path.split(/[/\\]/).pop() || "screenshot.png",
      type: mimeMap[ext] || "image/png",
    }];
  }

  function arrayToBase64(arr: Uint8Array): string {
    let binary = "";
    const chunk = 8192;
    for (let i = 0; i < arr.length; i += chunk) {
      binary += String.fromCharCode(...arr.subarray(i, i + chunk));
    }
    return btoa(binary);
  }

  let dropUnlisten: (() => void) | null = null;

  async function handlePaste(e: ClipboardEvent) {
    if (!showFeedback || fbType !== "bug") return;
    const items = e.clipboardData?.items;
    if (!items) return;
    for (const item of items) {
      if (item.type.startsWith("image/") && fbImages.length < MAX_IMAGES) {
        e.preventDefault();
        const blob = item.getAsFile();
        if (!blob) continue;
        const buf = await blob.arrayBuffer();
        const base64 = arrayToBase64(new Uint8Array(buf));
        const ext = blob.type.split("/")[1] || "png";
        fbImages = [...fbImages, { base64, name: `粘贴图片.${ext}`, type: blob.type }];
      }
    }
  }

  async function startDropListener() {
    const unlisten = await getCurrentWindow().onDragDropEvent((event) => {
      if (!showFeedback || fbType !== "bug") return;
      if (event.payload.type === "drop") {
        const paths = event.payload.paths.filter((p: string) => {
          const ext = p.split(".").pop()?.toLowerCase() || "";
          return ["png", "jpg", "jpeg", "gif", "webp"].includes(ext);
        });
        for (const p of paths) {
          if (fbImages.length < MAX_IMAGES) addImageFromPath(p);
        }
      }
    });
    dropUnlisten = unlisten;
  }

  function stopDropListener() {
    dropUnlisten?.();
    dropUnlisten = null;
  }

  function removeImage(index: number) {
    fbImages = fbImages.filter((_, i) => i !== index);
  }

  async function submitFeedback() {
    if (!fbDesc.trim()) return;
    fbSubmitting = true;
    fbResult = "idle";
    fbError = "";
    try {
      const body: Record<string, unknown> = {
        type: fbType === "bug" ? "Bug" : fbType === "feature" ? "功能建议" : "其他",
        description: fbDesc,
        name: fbName,
        contact: fbContact,
        images: fbImages.map((img) => ({ base64: img.base64, name: img.name, type: img.type })),
      };
      const resp = await fetch(FEEDBACK_URL, {
        method: "POST",
        body: JSON.stringify(body),
      });
      if (resp.ok) {
        fbResult = "success";
        fbDesc = "";
        fbName = "";
        fbContact = "";
        fbImages = [];
      } else {
        fbResult = "error";
        fbError = "提交失败，请稍后重试";
      }
    } catch (e) {
      fbResult = "error";
      fbError = "网络错误，请检查连接";
    }
    fbSubmitting = false;
  }

  function closeFeedback() {
    showFeedback = false;
    fbResult = "idle";
    stopDropListener();
    document.removeEventListener("paste", handlePaste);
  }

  async function pickOutputDir(key: "defaultImageOutputDir") {
    const dir = await open({ directory: true, multiple: false });
    if (dir && typeof dir === "string") {
      appSettings.update((s) => ({ ...s, [key]: dir }));
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
        <span class="setting-label">界面缩放</span>
        <span class="setting-desc">调整界面整体大小</span>
      </div>
      <SegmentControl
        options={[
          { value: "80", label: "80%" },
          { value: "90", label: "90%" },
          { value: "100", label: "100%" },
          { value: "110", label: "110%" },
          { value: "120", label: "120%" },
        ]}
        selected={String($appSettings.uiScale || 100)}
        onchange={(v) => appSettings.update((s) => ({ ...s, uiScale: Number(v) }))}
      />
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

  </div>

  <div class="settings-group">
    <span class="group-label">默认输出目录</span>

    {#each [
      { key: "defaultImageOutputDir" as const, label: "图片" },
    ] as item}
      <div class="setting-row">
        <div class="setting-info">
          <span class="setting-label">{item.label}</span>
          <span class="setting-desc">另存时未选择目录则使用此目录</span>
        </div>
        <div class="setting-control">
          <button class="path-btn" onclick={() => pickOutputDir(item.key)} title={$appSettings[item.key] || "点击选择"}>
            {$appSettings[item.key] ? $appSettings[item.key].split(/[/\\]/).pop() : "未设置"}
          </button>
          {#if $appSettings[item.key]}
            <button class="clear-btn" onclick={() => appSettings.update((s) => ({ ...s, [item.key]: "" }))} title="清除">
              <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
            </button>
          {/if}
        </div>
      </div>
    {/each}
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
        <span class="setting-desc">系统登录时自动启动 媒体工坊</span>
      </div>
      <label class="toggle">
        <input type="checkbox" checked={autoStartEnabled} onchange={toggleAutoStart} />
        <span class="toggle-track"></span>
      </label>
    </div>
  </div>

  <div class="settings-group">
    <span class="group-label">关于</span>

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">
          版本
          {#if $hasUpdate}<span class="update-dot"></span>{/if}
        </span>
        <span class="setting-desc">媒体工坊 v{appVersion}{$hasUpdate ? " · 有新版本可用" : ""}</span>
      </div>
      <div class="setting-control">
        {#if updateStatus === "idle" || updateStatus === "none" || updateStatus === "error"}
          <button class="update-btn" onclick={checkForUpdate}>
            {updateStatus === "none" ? "已是最新" : updateStatus === "error" ? "重试" : "检查更新"}
          </button>
        {:else if updateStatus === "checking"}
          <span class="update-text">检查中...</span>
        {:else if updateStatus === "available"}
          <button class="update-btn update-btn-accent" onclick={downloadAndInstall}>
            更新到 v{updateVersion}
          </button>
        {:else if updateStatus === "downloading"}
          <span class="update-text">下载安装中...</span>
        {:else if updateStatus === "ready"}
          <span class="update-text">即将重启...</span>
        {/if}
      </div>
    </div>
    {#if updateError}
      <div class="setting-row">
        <span class="update-error">{updateError}</span>
      </div>
    {/if}

    <div class="setting-row">
      <div class="setting-info">
        <span class="setting-label">反馈</span>
        <span class="setting-desc">报告问题或提出功能建议</span>
      </div>
      <button class="update-btn" onclick={() => { showFeedback = true; fbResult = "idle"; startDropListener(); document.addEventListener("paste", handlePaste); }}>
        提交反馈
      </button>
    </div>
  </div>
</div>

<!-- Feedback modal -->
{#if showFeedback}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div class="fb-overlay" onclick={closeFeedback}>
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="fb-modal" onclick={(e) => e.stopPropagation()}>
      <div class="fb-header">
        <span class="fb-title">提交反馈</span>
        <button class="fb-close" onclick={closeFeedback}>
          <svg width="14" height="14" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
        </button>
      </div>

      {#if fbResult === "success"}
        <div class="fb-success">
          <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"/><path d="M22 4L12 14.01l-3-3"/>
          </svg>
          <span>感谢你的反馈！</span>
          <button class="update-btn" onclick={closeFeedback}>关闭</button>
        </div>
      {:else}
        <div class="fb-body">
          <div class="fb-field">
            <span class="fb-label">类型</span>
            <SegmentControl
              options={[
                { value: "bug", label: "Bug" },
                { value: "feature", label: "功能建议" },
                { value: "other", label: "其他" },
              ]}
              selected={fbType}
              onchange={(v) => { fbType = v; if (v !== "bug") fbImages = []; }}
            />
          </div>

          <div class="fb-field">
            <span class="fb-label">描述</span>
            <textarea class="fb-textarea" rows="4" placeholder="请详细描述你遇到的问题或想要的功能..." bind:value={fbDesc}></textarea>
          </div>

          {#if fbType === "bug"}
          <div class="fb-field">
            <span class="fb-label">截图（可选，支持粘贴/拖拽/选择，最多 {MAX_IMAGES} 张）</span>
            <div class="fb-drop-zone">
              {#if fbImages.length > 0}
                <div class="fb-thumbs">
                  {#each fbImages as img, i}
                    <div class="fb-thumb">
                      <img src="data:{img.type};base64,{img.base64}" alt={img.name} class="fb-thumb-img" />
                      <button class="fb-thumb-remove" onclick={() => removeImage(i)}>
                        <svg width="8" height="8" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.5"/></svg>
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
              <div class="fb-drop-hint">
                <button class="update-btn" onclick={pickScreenshot}>选择图片</button>
                <span class="fb-drop-text">或拖拽 / Ctrl+V 粘贴</span>
              </div>
            </div>
          </div>
          {/if}

          <div class="fb-field">
            <span class="fb-label">联系人（可选）</span>
            <input class="fb-input" type="text" placeholder="你的名字或昵称" bind:value={fbName} />
          </div>

          <div class="fb-field">
            <span class="fb-label">联系方式（可选）</span>
            <input class="fb-input" type="text" placeholder="邮箱或其他联系方式，方便我们回复你" bind:value={fbContact} />
          </div>

          {#if fbResult === "error"}
            <span class="fb-error">{fbError}</span>
          {/if}

          <button class="fb-submit" onclick={submitFeedback} disabled={fbSubmitting || !fbDesc.trim()}>
            {fbSubmitting ? "提交中..." : "提交"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

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

  .update-btn {
    padding: 4px 12px;
    border: 1px solid var(--color-border);
    border-radius: 4px;
    background: transparent;
    color: var(--color-text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
    outline: none;
  }
  .update-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }
  .update-btn-accent {
    background: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-bg-primary);
    font-weight: 600;
  }
  .update-btn-accent:hover { background: var(--color-accent-hover); border-color: var(--color-accent-hover); color: var(--color-bg-primary); }

  .update-text {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .update-dot {
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--color-error);
    margin-left: 4px;
    vertical-align: middle;
  }

  .update-error {
    font-size: 10px;
    color: var(--color-error);
    word-break: break-all;
  }
  /* Feedback modal */
  .fb-overlay {
    position: fixed;
    top: 0; left: 0;
    width: 100vw; height: 100vh;
    background: rgba(0, 0, 0, 0.6);
    z-index: 1000;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(4px);
    -webkit-backdrop-filter: blur(4px);
  }

  .fb-modal {
    width: 420px;
    max-height: 85vh;
    background: var(--color-bg-secondary);
    border: 0.5px solid var(--color-border);
    border-radius: 10px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
  }

  .fb-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 0.5px solid var(--color-border);
  }

  .fb-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-primary);
  }

  .fb-close {
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: 4px;
    outline: none;
    display: flex;
    align-items: center;
    transition: color 0.15s;
  }
  .fb-close:hover { color: var(--color-text-primary); }

  .fb-body {
    padding: 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .fb-field {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .fb-label {
    font-size: 11px;
    color: var(--color-text-muted);
    font-weight: 500;
  }

  .fb-textarea {
    width: 100%;
    padding: 8px 10px;
    background: var(--color-bg-surface);
    border: 0.5px solid var(--color-border);
    border-radius: 5px;
    color: var(--color-text-primary);
    font-size: 12px;
    font-family: inherit;
    resize: vertical;
    outline: none;
    min-height: 80px;
  }
  .fb-textarea:focus { border-color: var(--color-accent); }
  .fb-textarea::placeholder { color: var(--color-text-muted); }

  .fb-input {
    width: 100%;
    padding: 6px 10px;
    background: var(--color-bg-surface);
    border: 0.5px solid var(--color-border);
    border-radius: 5px;
    color: var(--color-text-primary);
    font-size: 12px;
    outline: none;
  }
  .fb-input:focus { border-color: var(--color-accent); }
  .fb-input::placeholder { color: var(--color-text-muted); }

  .fb-drop-zone {
    border: 1.5px dashed var(--color-border);
    border-radius: 6px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: border-color 0.15s;
    outline: none;
  }
  .fb-drop-zone:focus, .fb-drop-zone:hover { border-color: var(--color-accent); }

  .fb-drop-hint {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .fb-drop-text {
    font-size: 10px;
    color: var(--color-text-muted);
  }

  .fb-thumbs {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }

  .fb-thumb {
    position: relative;
    width: 60px;
    height: 60px;
    border-radius: 4px;
    overflow: hidden;
    border: 0.5px solid var(--color-border);
  }

  .fb-thumb-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .fb-thumb-remove {
    position: absolute;
    top: 2px;
    right: 2px;
    width: 16px;
    height: 16px;
    border: none;
    border-radius: 50%;
    background: rgba(0, 0, 0, 0.6);
    color: white;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    outline: none;
    transition: background 0.15s;
  }
  .fb-thumb-remove:hover { background: var(--color-error); }

  .fb-submit {
    padding: 7px 0;
    background: var(--color-accent);
    color: var(--color-bg-primary);
    border: none;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
    outline: none;
  }
  .fb-submit:hover:not(:disabled) { background: var(--color-accent-hover); }
  .fb-submit:disabled { opacity: 0.5; cursor: not-allowed; }

  .fb-error {
    font-size: 11px;
    color: var(--color-error);
  }

  .fb-success {
    padding: 32px 16px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    color: var(--color-success);
    font-size: 14px;
    font-weight: 500;
  }
</style>
