<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";
  import { load as loadStore } from "@tauri-apps/plugin-store";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onDestroy, onMount } from "svelte";
  import { watchFolders, createWatchFolder, extsForType, loadWatchFolders, type WatchFolder } from "$lib/stores/watcher";
  import { imagePresets, appSettings, type ImagePreset } from "$lib/stores/settings";
  import { formatSize, formatRatio } from "$lib/utils/fileSize";
  import { splitPath, joinPath, fileName as getFileName } from "$lib/utils/path";
  import SegmentControl from "$lib/components/SegmentControl.svelte";
  import type { ImageInfo, ProcessResult, MediaFile } from "$lib/types";

  async function notifyIfBackground(title: string, body: string) {
    if (!$appSettings.notificationsEnabled) return;
    try {
      const focused = await getCurrentWindow().isFocused();
      if (focused) return;
      let granted = await isPermissionGranted();
      if (!granted) {
        const perm = await requestPermission();
        granted = perm === "granted";
      }
      if (granted) sendNotification({ title, body });
    } catch (_) {}
  }

  function formatTime(ts: number): string {
    const d = new Date(ts);
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}`;
  }

  function defaultDirForType(mt: string): string {
    if (mt === "image") return $appSettings.defaultImageOutputDir;
    if (mt === "video") return $appSettings.defaultVideoOutputDir;
    return $appSettings.defaultAudioOutputDir;
  }

  function defaultDirLabel(mt: string): string {
    const dir = defaultDirForType(mt);
    return dir ? `默认: ${getFileName(dir)}` : "选择目录...";
  }

  function getPreset(presetId: string): ImagePreset {
    return imagePresets.find((p) => p.id === presetId) || imagePresets[0];
  }

  // Per-folder file maps: folderId -> MediaFile[]
  let folderFiles: Record<string, MediaFile[]> = $state({});
  let processing = $state(false);
  let unlisten: (() => void) | null = null;
  let selectedFolderId: string | null = $state(null);
  let logsReady = false;

  // --- Log persistence ---
  const LOG_STORE = "watcher-logs.json";
  const LOG_KEY = "logs";
  let logSaveTimer: ReturnType<typeof setTimeout> | null = null;

  async function loadLogs() {
    try {
      const store = await loadStore(LOG_STORE);
      const saved = await store.get<Record<string, MediaFile[]>>(LOG_KEY);
      if (saved && typeof saved === "object") {
        // Only restore completed/error/skipped entries (not pending/processing)
        const cleaned: Record<string, MediaFile[]> = {};
        for (const [fid, files] of Object.entries(saved)) {
          cleaned[fid] = (files as MediaFile[]).filter((f) =>
            f.status === "done" || f.status === "error" || f.status === "skipped",
          );
        }
        folderFiles = cleaned;
      }
    } catch (_) {}
    logsReady = true;
  }

  function scheduleSaveLogs() {
    if (!logsReady) return;
    if (logSaveTimer) clearTimeout(logSaveTimer);
    logSaveTimer = setTimeout(saveLogs, 500);
  }

  async function saveLogs() {
    try {
      const store = await loadStore(LOG_STORE);
      await store.set(LOG_KEY, folderFiles);
      await store.save();
    } catch (_) {}
  }

  // Counter-based ignore: each write adds 1, each watcher event consumes 1.
  // 10s expiry as safety net in case a watcher event never arrives.
  const ignorePaths = new Map<string, { count: number; expiry: number }>();

  function addIgnorePath(path: string) {
    const e = ignorePaths.get(path);
    if (e) { e.count++; e.expiry = Date.now() + 10000; }
    else ignorePaths.set(path, { count: 1, expiry: Date.now() + 10000 });
  }

  function isIgnored(path: string): boolean {
    const e = ignorePaths.get(path);
    if (!e) return false;
    if (Date.now() > e.expiry) { ignorePaths.delete(path); return false; }
    e.count--;
    if (e.count <= 0) ignorePaths.delete(path);
    return true;
  }

  setInterval(() => {
    const now = Date.now();
    for (const [p, e] of ignorePaths) { if (now > e.expiry) ignorePaths.delete(p); }
  }, 10000);

  function getFiles(folderId: string | null): MediaFile[] {
    if (!folderId) return [];
    return folderFiles[folderId] ?? [];
  }

  function setFiles(folderId: string, files: MediaFile[]) {
    folderFiles = { ...folderFiles, [folderId]: files };
    scheduleSaveLogs();
  }

  // Popover state
  let popoverId: string | null = $state(null);
  let popoverPos = $state({ top: 0, right: 0 });
  let popoverEl: HTMLDivElement | null = null;

  const mediaTypeOptions = [
    { value: "image", label: "图片" },
    { value: "video", label: "视频" },
    { value: "audio", label: "音频" },
  ];

  const formatOptions = [
    { value: "original", label: "原格式" },
    { value: "webp", label: "WEBP" },
    { value: "avif", label: "AVIF" },
    { value: "png", label: "PNG" },
    { value: "jpeg", label: "JPG" },
  ];

  const mediaLabels: Record<string, string> = { image: "图片", video: "视频", audio: "音频" };
  const formatLabels: Record<string, string> = { original: "原格式", webp: "WEBP", avif: "AVIF", png: "PNG", jpeg: "JPG" };

  function folderSummary(f: WatchFolder): string {
    const allExts = extsForType(f.mediaType);
    const parts: string[] = [];
    parts.push(f.active ? "监听中" : "已停止");
    parts.push(mediaLabels[f.mediaType] || f.mediaType);
    if (f.fileExts.length < allExts.length) {
      parts.push(f.fileExts.map((e) => `.${e}`).join(" "));
    }
    if (f.autoProcess && f.mediaType === "image") {
      if (f.outputFormat === "png") {
        const q = f.pngQuality ?? 100;
        parts.push(q === 100 ? "PNG 无损" : `PNG ${q}%`);
      } else {
        const p = getPreset(f.preset);
        parts.push(p.label);
        if (f.outputFormat !== "original") parts.push(formatLabels[f.outputFormat] || f.outputFormat);
      }
    }
    parts.push(f.outputMode === "saveto" ? "另存" : "覆盖");
    if (!f.recursive) parts.push("仅当前层");
    return parts.join(" · ");
  }

  function folderForFile(baseDir: string): WatchFolder | undefined {
    return $watchFolders.find((f) => baseDir.startsWith(f.path));
  }

  function openPopover(id: string, event: MouseEvent) {
    const btn = event.currentTarget as HTMLElement;
    const rect = btn.getBoundingClientRect();
    const viewEl = btn.closest(".view") as HTMLElement;
    const viewRect = viewEl.getBoundingClientRect();
    popoverPos = {
      top: rect.bottom - viewRect.top + 4,
      right: viewRect.right - rect.right,
    };
    popoverId = popoverId === id ? null : id;
  }

  function closePopover() { popoverId = null; }

  function handleOutsideClick(e: MouseEvent) {
    if (!popoverId) return;
    const target = e.target as HTMLElement;
    if (popoverEl && !popoverEl.contains(target) && !target.closest(".gear-btn")) {
      closePopover();
    }
  }

  onMount(async () => {
    document.addEventListener("mousedown", handleOutsideClick);

    await loadWatchFolders();
    await loadLogs();

    // Restore active states from backend
    try {
      const activePaths: string[] = await invoke("get_active_watchers");
      if (activePaths.length > 0) {
        watchFolders.update((list) =>
          list.map((f) => activePaths.includes(f.path) ? { ...f, active: true } : f),
        );
      }
    } catch (_) {}

    unlisten = await listen<{ path: string; base_dir: string }>("watcher:new-file", (event) => {
      const { path, base_dir } = event.payload;
      const folder = folderForFile(base_dir);
      if (!folder) return;
      const fid = folder.id;

      // Only skip if the same path is currently pending or processing
      if (getFiles(fid).some((f) => f.path === path && (f.status === "pending" || f.status === "processing"))) return;
      if (isIgnored(path)) return;

      const name = getFileName(path);
      const ext = name.split(".").pop()?.toLowerCase() || "unknown";
      const fileId = crypto.randomUUID();

      // Add to list immediately (synchronous) to prevent concurrent overwrites
      const placeholder: MediaFile = {
        id: fileId, path, name, baseDir: base_dir,
        originalSize: 0, format: ext,
        status: "pending", mediaType: folder.mediaType,
      };
      setFiles(fid, [...getFiles(fid), placeholder]);

      // Fetch details and auto-process asynchronously
      if (folder.mediaType === "image") {
        invoke<ImageInfo>("get_image_info", { path }).then((info) => {
          const files = getFiles(fid);
          setFiles(fid, files.map((f) =>
            f.id === fileId
              ? { ...f, originalSize: info.file_size, format: info.format, width: info.width, height: info.height }
              : f,
          ));
          if (folder.autoProcess) {
            const idx = getFiles(fid).findIndex((f) => f.id === fileId);
            if (idx >= 0) autoProcessFile(fid, idx);
          }
        }).catch((e) => {
          const files = getFiles(fid);
          setFiles(fid, files.map((f) =>
            f.id === fileId ? { ...f, status: "error", error: String(e), processedAt: Date.now() } : f,
          ));
        });
      }
    });
  });

  onDestroy(() => {
    unlisten?.();
    document.removeEventListener("mousedown", handleOutsideClick);
  });

  async function addWatchDir() {
    const dir = await open({ directory: true, multiple: false });
    if (dir && typeof dir === "string") {
      watchFolders.update((list) => {
        if (list.some((f) => f.path === dir)) return list;
        return [...list, createWatchFolder(dir)];
      });
    }
  }

  function removeWatchDir(id: string) {
    if (popoverId === id) closePopover();
    const folder = $watchFolders.find((f) => f.id === id);
    if (folder?.active) {
      invoke("stop_folder_watcher", { path: folder.path }).catch(() => {});
    }
    watchFolders.update((list) => list.filter((f) => f.id !== id));
  }

  function updateFolder(id: string, patch: Partial<WatchFolder>) {
    const folder = $watchFolders.find((f) => f.id === id);
    watchFolders.update((list) => list.map((f) => (f.id === id ? { ...f, ...patch } : f)));

    // Auto-restart watcher if backend-relevant settings changed while active
    if (folder?.active && ("fileExts" in patch || "recursive" in patch)) {
      const updated = { ...folder, ...patch };
      invoke("stop_folder_watcher", { path: updated.path }).then(() =>
        invoke("start_folder_watcher", {
          folder: { path: updated.path, exts: updated.fileExts, recursive: updated.recursive },
        }),
      ).catch((e) => console.error("Failed to restart watcher:", e));
    }
  }

  async function pickFolderOutputDir(id: string) {
    const dir = await open({ directory: true, multiple: false });
    if (dir && typeof dir === "string") updateFolder(id, { outputDir: dir });
  }

  async function toggleFolder(id: string) {
    const folder = $watchFolders.find((f) => f.id === id);
    if (!folder) return;

    if (folder.active) {
      await invoke("stop_folder_watcher", { path: folder.path });
      updateFolder(id, { active: false });
    } else {
      try {
        await invoke("start_folder_watcher", {
          folder: { path: folder.path, exts: folder.fileExts, recursive: folder.recursive },
        });
        updateFolder(id, { active: true });
      } catch (e) {
        console.error("Failed to start watcher for", folder.path, e);
      }
    }
  }

  function getOutputPath(file: MediaFile, folder: WatchFolder | undefined): string {
    const { dir: originalDir, name: fileName, sep } = splitPath(file.path);
    const dotIdx = fileName.lastIndexOf(".");
    const baseName = dotIdx > 0 ? fileName.substring(0, dotIdx) : fileName;
    const originalExt = dotIdx > 0 ? fileName.substring(dotIdx + 1) : "";

    const outputFormat = folder?.outputFormat ?? "original";
    let ext = originalExt;
    if (outputFormat !== "original") {
      ext = outputFormat === "jpeg" ? "jpg" : outputFormat;
    }

    const outputMode = folder?.outputMode ?? "overwrite";
    const mediaType = folder?.mediaType ?? "image";
    const defaultDir = mediaType === "image" ? $appSettings.defaultImageOutputDir
      : mediaType === "video" ? $appSettings.defaultVideoOutputDir
      : $appSettings.defaultAudioOutputDir;
    const outputDir = folder?.outputDir || defaultDir;
    const preserveStructure = folder?.preserveStructure ?? true;

    if (outputMode === "saveto" && outputDir) {
      if (preserveStructure && file.baseDir) {
        const relative = file.path.startsWith(file.baseDir)
          ? file.path.slice(file.baseDir.length).replace(/^[/\\]/, "")
          : fileName;
        const { dir: relDir, name: relName } = splitPath(relative);
        const relDotIdx = relName.lastIndexOf(".");
        const relBase = relDotIdx > 0 ? relName.substring(0, relDotIdx) : relName;
        const outDir = relDir ? joinPath([outputDir, relDir], sep) : outputDir;
        return joinPath([outDir, `${relBase}.${ext}`], sep);
      }
      return joinPath([outputDir, `${baseName}.${ext}`], sep);
    }

    return joinPath([originalDir, `${baseName}.${ext}`], sep);
  }

  async function autoProcessFile(folderId: string, index: number) {
    const files = getFiles(folderId);
    const file = files[index];
    if (!file || file.status !== "pending" || file.mediaType !== "image") return;

    const folder = $watchFolders.find((f) => f.id === folderId);
    const outputFormat = folder?.outputFormat ?? "original";
    const needsConvert = outputFormat !== "original";
    const isPngOutput = outputFormat === "png";

    // PNG output: lossless only, ignore preset
    const preset = isPngOutput ? null : getPreset(folder?.preset ?? "lossless");

    const isTarget = preset?.targetSize !== null && preset?.targetSize !== undefined;

    // Skip entirely if file already meets target size AND no format conversion needed
    if (isTarget && !needsConvert && file.originalSize > 0) {
      const targetBytes = Math.round(preset!.targetSize! * 1024 * 1024);
      if (file.originalSize <= targetBytes) {
        setFiles(folderId, files.map((f, i) =>
          i === index ? { ...f, status: "skipped", processInfo: `已满足 ≤${preset!.targetSize}MB`, processedAt: Date.now() } : f,
        ));
        return;
      }
    }

    setFiles(folderId, files.map((f, i) => (i === index ? { ...f, status: "processing" } : f)));

    try {
      let compressEnabled: boolean;
      let quality: number;
      let targetBytes: number | null = null;
      let resizeEnabled = false;
      let resizeWidth: number | null = null;

      if (isPngOutput) {
        const q = folder?.pngQuality ?? 100;
        compressEnabled = true;
        quality = q;
      } else if (preset) {
        const isQuality = !isTarget && preset.quality < 100;
        targetBytes = isTarget ? Math.round(preset.targetSize! * 1024 * 1024) : null;
        const alreadyFits = isTarget && file.originalSize > 0 && targetBytes !== null && file.originalSize <= targetBytes;
        compressEnabled = alreadyFits ? false : (isQuality || isTarget);
        quality = compressEnabled ? preset.quality : 100;
        if (alreadyFits) targetBytes = null;
        resizeEnabled = preset.resizeWidth !== null;
        resizeWidth = preset.resizeWidth;
      } else {
        compressEnabled = false;
        quality = 100;
      }

      const outputPath = getOutputPath(file, folder);
      addIgnorePath(outputPath);

      const result: ProcessResult = await invoke("process_image", {
        fileId: file.id,
        filePath: file.path, outputPath,
        settings: {
          compress_enabled: compressEnabled,
          quality,
          convert_enabled: needsConvert,
          output_format: outputFormat,
          resize_enabled: resizeEnabled,
          resize_width: resizeWidth,
          resize_height: null,
          keep_aspect_ratio: true,
          target_size: targetBytes,
        },
      });

      addIgnorePath(result.output_path);

      if (folder?.outputMode === "overwrite" && result.output_path !== file.path) {
        await invoke("delete_file", { path: file.path }).catch(() => {});
      }

      // Build process info description
      const infoParts: string[] = [];
      if (isPngOutput) {
        const q = folder?.pngQuality ?? 100;
        infoParts.push(q === 100 ? "PNG 无损" : `PNG ${q}%`);
      } else if (preset) {
        const isQuality = !isTarget && preset.quality < 100;
        if (isQuality) infoParts.push(`质量 ${preset.quality}%`);
        else if (isTarget) infoParts.push(`目标 ≤${preset.targetSize}MB`);
        else infoParts.push("无损");
        if (preset.resizeWidth) infoParts.push(`缩放 ${preset.resizeWidth}px`);
      }
      if (outputFormat !== "original") infoParts.push(`→ ${outputFormat.toUpperCase()}`);
      if (folder?.outputMode === "overwrite") infoParts.push("覆盖");
      else if (folder?.outputDir) infoParts.push(`→ ${getFileName(folder.outputDir)}`);
      const processInfo = infoParts.join(" · ");

      const updated = getFiles(folderId);
      setFiles(folderId, updated.map((f, i) =>
        i === index
          ? { ...f, status: "done", compressedSize: result.compressed_size, ratio: result.ratio, outputPath: result.output_path, processInfo, processedAt: Date.now() }
          : f,
      ));

      const saved = result.original_size - result.compressed_size;
      notifyIfBackground("处理完成", `${file.name} · ${processInfo}${saved > 0 ? ` · 节省 ${formatSize(saved)}` : ""}`);
    } catch (e) {
      const updated = getFiles(folderId);
      setFiles(folderId, updated.map((f, i) =>
        i === index ? { ...f, status: "error", error: String(e), processedAt: Date.now() } : f,
      ));
      notifyIfBackground("处理失败", `${file.name} · ${String(e)}`);
    }
  }

  async function processAllPending() {
    if (!selectedFolderId) return;
    processing = true;
    const files = getFiles(selectedFolderId);
    const pending = files
      .map((f, i) => ({ f, i }))
      .filter(({ f }) => f.status === "pending" && f.mediaType === "image");
    for (const { i } of pending) {
      await autoProcessFile(selectedFolderId!, i);
    }
    processing = false;
  }

  function retryFile(index: number) {
    if (!selectedFolderId) return;
    const files = getFiles(selectedFolderId);
    setFiles(selectedFolderId, files.map((f, i) => i === index ? { ...f, status: "pending", error: undefined } : f));
    autoProcessFile(selectedFolderId, index);
  }

  function removeFile(index: number) {
    if (!selectedFolderId) return;
    setFiles(selectedFolderId, getFiles(selectedFolderId).filter((_, i) => i !== index));
  }

  function clearFiles() {
    if (!selectedFolderId) return;
    setFiles(selectedFolderId, []);
  }

  function selectFolder(id: string) {
    selectedFolderId = selectedFolderId === id ? null : id;
  }

  let currentFiles = $derived(getFiles(selectedFolderId));
  let doneCount = $derived(currentFiles.filter((f) => f.status === "done").length);
  let errorCount = $derived(currentFiles.filter((f) => f.status === "error").length);
  let pendingCount = $derived(currentFiles.filter((f) => f.status === "pending").length);
  let totalSaved = $derived(
    currentFiles.reduce((acc, f) => {
      if (f.status === "done" && f.compressedSize !== undefined) return acc + (f.originalSize - f.compressedSize);
      return acc;
    }, 0),
  );

  function folderFileCount(folderId: string): number {
    return (folderFiles[folderId] ?? []).length;
  }
</script>

<div class="view">
  <div class="config-section">
    <div class="config-header">
      <h2 class="view-title">文件夹监听</h2>
      <div class="header-actions">
        <button class="add-dir-btn" onclick={addWatchDir}>
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M5 1v8M1 5h8" stroke="currentColor" stroke-width="1.4"/></svg>
          添加文件夹
        </button>
      </div>
    </div>

    {#if $watchFolders.length > 0}
      <div class="folder-list">
        {#each $watchFolders as folder (folder.id)}
          <div class="folder-row" class:active={popoverId === folder.id} class:selected={selectedFolderId === folder.id}>
            <button class="play-btn" class:watching={folder.active} onclick={() => toggleFolder(folder.id)} title={folder.active ? "停止" : "开始"}>
              <svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                {#if folder.active}
                  <rect x="6" y="4" width="4" height="16"/><rect x="14" y="4" width="4" height="16"/>
                {:else}
                  <polygon points="5,3 19,12 5,21"/>
                {/if}
              </svg>
            </button>
            <!-- svelte-ignore a11y_click_events_have_key_events -->
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div class="folder-info" onclick={() => selectFolder(folder.id)}>
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="dir-icon">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
              </svg>
              <span class="dir-name" title={folder.path}>{getFileName(folder.path) || folder.path}</span>
              {#if folderFileCount(folder.id) > 0}
                <span class="file-count">{folderFileCount(folder.id)}</span>
              {/if}
              <span class="folder-summary">{folderSummary(folder)}</span>
            </div>
            <button class="gear-btn" class:open={popoverId === folder.id} onclick={(e) => openPopover(folder.id, e)} title="设置">
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
                <circle cx="12" cy="12" r="3"/>
                <path d="M19.4 15a1.65 1.65 0 00.33 1.82l.06.06a2 2 0 01-2.83 2.83l-.06-.06a1.65 1.65 0 00-1.82-.33 1.65 1.65 0 00-1 1.51V21a2 2 0 01-4 0v-.09A1.65 1.65 0 009 19.4a1.65 1.65 0 00-1.82.33l-.06.06a2 2 0 01-2.83-2.83l.06-.06A1.65 1.65 0 004.68 15a1.65 1.65 0 00-1.51-1H3a2 2 0 010-4h.09A1.65 1.65 0 004.6 9a1.65 1.65 0 00-.33-1.82l-.06-.06a2 2 0 012.83-2.83l.06.06A1.65 1.65 0 009 4.68a1.65 1.65 0 001-1.51V3a2 2 0 014 0v.09a1.65 1.65 0 001 1.51 1.65 1.65 0 001.82-.33l.06-.06a2 2 0 012.83 2.83l-.06.06A1.65 1.65 0 0019.4 9a1.65 1.65 0 001.51 1H21a2 2 0 010 4h-.09a1.65 1.65 0 00-1.51 1z"/>
              </svg>
            </button>
            <button class="icon-btn" title="移除" onclick={() => removeWatchDir(folder.id)}>
              <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  <!-- Popover -->
  {#if popoverId}
    {@const folder = $watchFolders.find((f) => f.id === popoverId)}
    {#if folder}
      <div class="popover" style="top:{popoverPos.top}px;right:{popoverPos.right}px" bind:this={popoverEl}>
        <div class="pop-path" title={folder.path}>{folder.path}</div>

        <div class="pop-row">
          <span class="pop-label">类型</span>
          <SegmentControl
            options={mediaTypeOptions}
            selected={folder.mediaType}
            onchange={(v) => {
              const mt = v as "image" | "video" | "audio";
              updateFolder(folder.id, { mediaType: mt, fileExts: extsForType(mt) });
            }}
          />
        </div>

        <div class="pop-row">
          <span class="pop-label">格式</span>
          <div class="ext-group">
            {#each extsForType(folder.mediaType) as ext}
              <label class="ext-chip" class:active={folder.fileExts.includes(ext)}>
                <input type="checkbox"
                  checked={folder.fileExts.includes(ext)}
                  onchange={(e) => {
                    const checked = (e.target as HTMLInputElement).checked;
                    const newExts = checked
                      ? [...folder.fileExts, ext]
                      : folder.fileExts.filter((x) => x !== ext);
                    updateFolder(folder.id, { fileExts: newExts });
                  }}
                />
                .{ext}
              </label>
            {/each}
          </div>
        </div>

        <div class="pop-row">
          <span class="pop-label">输出</span>
          <SegmentControl
            options={[
              { value: "overwrite", label: "覆盖" },
              { value: "saveto", label: "另存到" },
            ]}
            selected={folder.outputMode}
            onchange={(v) => updateFolder(folder.id, { outputMode: v as "overwrite" | "saveto" })}
          />
        </div>
        {#if folder.outputMode === "saveto"}
          <div class="pop-row">
            <span class="pop-label">目录</span>
            <button class="path-btn" onclick={() => pickFolderOutputDir(folder.id)} title={folder.outputDir || "选择目录"}>
              {folder.outputDir ? getFileName(folder.outputDir) : defaultDirLabel(folder.mediaType)}
            </button>
            {#if folder.outputDir}
              <button class="icon-btn" title="清除" onclick={() => updateFolder(folder.id, { outputDir: "" })}>
                <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
              </button>
            {/if}
            <label class="opt">
              <input type="checkbox" checked={folder.preserveStructure}
                onchange={(e) => updateFolder(folder.id, { preserveStructure: (e.target as HTMLInputElement).checked })} />
              保留层级
            </label>
          </div>
        {/if}

        <div class="pop-row">
          <label class="opt">
            <input type="checkbox" checked={folder.recursive}
              onchange={(e) => updateFolder(folder.id, { recursive: (e.target as HTMLInputElement).checked })} />
            包含子文件夹
          </label>
        </div>

        <div class="pop-row">
          <label class="opt">
            <input type="checkbox" checked={folder.autoProcess}
              onchange={(e) => updateFolder(folder.id, { autoProcess: (e.target as HTMLInputElement).checked })} />
            自动处理
          </label>
        </div>

        {#if folder.autoProcess && folder.mediaType === "image"}
          <div class="pop-row">
            <span class="pop-label">格式</span>
            <SegmentControl
              options={formatOptions}
              selected={folder.outputFormat}
              onchange={(v) => updateFolder(folder.id, { outputFormat: v })}
            />
          </div>

          {#if folder.outputFormat === "png"}
            <div class="pop-row">
              <span class="pop-label">压缩</span>
              <input type="range" class="pop-slider" min="1" max="100"
                value={folder.pngQuality ?? 100}
                oninput={(e) => {
                  const q = Number((e.target as HTMLInputElement).value);
                  updateFolder(folder.id, {
                    pngQuality: q,
                    pngMode: q === 100 ? "lossless" : "lossy",
                  });
                }}
              />
              <span class="pop-value">{(folder.pngQuality ?? 100) === 100 ? "无损" : `${folder.pngQuality}%`}</span>
            </div>
          {:else}
            <div class="pop-row">
              <span class="pop-label">预设</span>
              <div class="preset-group">
                {#each imagePresets as p}
                  <button
                    class="preset-chip"
                    class:active={folder.preset === p.id}
                    onclick={() => updateFolder(folder.id, { preset: p.id })}
                  >{p.label}</button>
                {/each}
              </div>
            </div>
          {/if}
        {/if}
      </div>
    {/if}
  {/if}

  {#if selectedFolderId && currentFiles.length > 0}
    <div class="file-list">
      {#each currentFiles as file, i (file.id)}
        <div class="file-item">
          <div class="file-info">
            <span class="file-name" title={file.path}>{file.name}</span>
            <span class="file-meta">
              {#if file.originalSize > 0}{formatSize(file.originalSize)}{/if}
              {#if file.width && file.height} · {file.width}×{file.height}{/if}
              {#if file.format} · {file.format.toUpperCase()}{/if}
            </span>
            {#if file.processedAt}
              <span class="file-process-info">
                <span class="file-time">{formatTime(file.processedAt)}</span>
                {#if file.processInfo}{file.processInfo}{/if}
                {#if file.status === "error" && file.error}
                  <span class="file-error-text">{file.error}</span>
                {/if}
              </span>
            {/if}
          </div>
          <div class="file-status">
            {#if file.status === "pending"}
              <span class="badge badge-pending">等待</span>
            {:else if file.status === "processing"}
              <span class="badge badge-processing">处理中</span>
            {:else if file.status === "done"}
              <span class="badge badge-done">完成</span>
              {#if file.compressedSize !== undefined}
                <span class="file-result">
                  {formatSize(file.originalSize)} → {formatSize(file.compressedSize)}
                  {#if file.ratio !== undefined && file.ratio !== 0}
                    <span class="ratio" class:positive={file.ratio > 0}>
                      {file.ratio > 0 ? "-" : "+"}{formatRatio(Math.abs(file.ratio))}
                    </span>
                  {/if}
                </span>
              {/if}
            {:else if file.status === "error"}
              <span class="badge badge-error">错误</span>
              <button class="retry-btn" title="重试" onclick={() => retryFile(i)}>
                <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <path d="M1 4v6h6"/><path d="M3.51 15a9 9 0 1 0 2.13-9.36L1 10"/>
                </svg>
              </button>
            {:else if file.status === "skipped"}
              <span class="badge badge-skipped">跳过</span>
            {/if}
          </div>
          <button class="icon-btn" onclick={() => removeFile(i)} title="移除">
            <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
          </button>
        </div>
      {/each}
    </div>
  {:else}
    <div class="empty-state">
      <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" class="empty-icon">
        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/>
        <circle cx="12" cy="12" r="3"/>
      </svg>
      {#if !selectedFolderId}
        <p>点击文件夹查看处理记录</p>
      {:else if $watchFolders.find((f) => f.id === selectedFolderId)?.active}
        <p>正在监听，等待新文件...</p>
        <div class="pulse-dot"></div>
      {:else}
        <p>该文件夹暂无处理记录</p>
      {/if}
    </div>
  {/if}

  {#if selectedFolderId && currentFiles.length > 0}
    <div class="status-bar">
      <span class="status-text">
        {currentFiles.length} 个文件
        {#if doneCount > 0} · {doneCount} 已完成{/if}
        {#if errorCount > 0} · {errorCount} 错误{/if}
        {#if pendingCount > 0} · {pendingCount} 等待中{/if}
        {#if totalSaved > 0} · 节省 {formatSize(totalSaved)}{/if}
      </span>
      <div class="status-actions">
        <button class="btn-text-danger" onclick={clearFiles}>清空</button>
        {#if pendingCount > 0}
          <button class="btn-primary" onclick={processAllPending} disabled={processing}>
            {processing ? "处理中..." : `处理 ${pendingCount} 个文件`}
          </button>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .view {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 12px;
    overflow: hidden;
    position: relative;
  }

  .view-title { margin: 0; font-size: 14px; font-weight: 600; color: var(--color-text-primary); }

  .config-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-bottom: 12px;
    border-bottom: 0.5px solid var(--color-border);
    flex-shrink: 0;
  }

  .config-header { display: flex; align-items: center; justify-content: space-between; }
  .header-actions { display: flex; align-items: center; gap: 8px; }

  .add-dir-btn {
    display: flex; align-items: center; gap: 4px;
    padding: 3px 10px;
    background: var(--color-bg-surface);
    border: 0.5px dashed var(--color-border);
    border-radius: 4px;
    color: var(--color-text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
    outline: none;
  }
  .add-dir-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }

  /* --- Folder list --- */
  .folder-list {
    display: flex;
    flex-direction: column;
    gap: 2px;
    max-height: 140px;
    overflow-y: auto;
  }

  .folder-row {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 8px;
    border-radius: 4px;
    transition: background 0.1s;
  }
  .folder-row:hover { background: var(--color-bg-hover); }
  .folder-row.active { background: var(--color-bg-surface); }
  .folder-row.selected { background: var(--color-bg-surface); border-left: 2px solid var(--color-accent); padding-left: 6px; }

  .play-btn {
    display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px;
    border: none; background: transparent;
    color: var(--color-text-muted);
    cursor: pointer; border-radius: 3px; padding: 0;
    transition: all 0.15s; outline: none; flex-shrink: 0;
  }
  .play-btn:hover { color: var(--color-accent); background: var(--color-bg-surface); }
  .play-btn.watching { color: var(--color-success); }
  .play-btn.watching:hover { color: var(--color-error); }

  .folder-info {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .file-count {
    font-size: 9px;
    min-width: 16px;
    height: 16px;
    line-height: 16px;
    text-align: center;
    border-radius: 8px;
    background: var(--color-accent);
    color: var(--color-bg-primary);
    font-weight: 600;
    flex-shrink: 0;
  }

  .dir-icon { flex-shrink: 0; color: var(--color-text-muted); }

  .dir-name {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    flex-shrink: 0;
    max-width: 140px;
  }

  .folder-summary {
    flex: 1;
    min-width: 0;
    font-size: 10px;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .gear-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px; height: 22px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 4px;
    padding: 0;
    transition: all 0.15s;
    outline: none;
    flex-shrink: 0;
  }
  .gear-btn:hover { background: var(--color-bg-surface); color: var(--color-text-secondary); }
  .gear-btn.open { color: var(--color-accent); background: var(--color-bg-surface); }

  .icon-btn {
    display: flex; align-items: center; justify-content: center;
    width: 20px; height: 20px;
    border: none; background: transparent;
    color: var(--color-text-muted);
    cursor: pointer; border-radius: 3px; padding: 0;
    transition: all 0.15s; outline: none; flex-shrink: 0;
  }
  .icon-btn:hover { background-color: var(--color-bg-hover); }

  /* --- Popover --- */
  .popover {
    position: absolute;
    z-index: 100;
    min-width: 280px;
    max-width: 360px;
    padding: 10px 12px;
    background: var(--color-bg-secondary);
    border: 0.5px solid var(--color-border);
    border-radius: 8px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.35);
    display: flex;
    flex-direction: column;
    gap: 8px;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .pop-path {
    font-size: 10px;
    color: var(--color-text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    padding-bottom: 6px;
    border-bottom: 0.5px solid var(--color-border);
  }

  .pop-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .pop-label {
    font-size: 11px;
    color: var(--color-text-muted);
    flex-shrink: 0;
    min-width: 28px;
  }

  .pop-slider {
    width: 70px;
    height: 3px;
    appearance: none;
    background: var(--color-bg-hover);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .pop-slider::-webkit-slider-thumb {
    appearance: none;
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
  }

  .pop-value {
    font-size: 11px;
    color: var(--color-accent);
    font-weight: 600;
    min-width: 28px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .ext-group {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-wrap: wrap;
  }

  .ext-chip {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 1px 7px;
    border: 1px solid var(--color-border);
    border-radius: 3px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 10px;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
    user-select: none;
  }
  .ext-chip input { display: none; }
  .ext-chip:hover { border-color: var(--color-accent); color: var(--color-text-secondary); }
  .ext-chip.active {
    background-color: color-mix(in srgb, var(--color-accent) 15%, transparent);
    border-color: var(--color-accent);
    color: var(--color-accent);
  }

  .preset-group {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-wrap: wrap;
  }

  .preset-chip {
    padding: 1px 8px;
    border: 1px solid var(--color-border);
    border-radius: 10px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 10px;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
    outline: none;
  }
  .preset-chip:hover { border-color: var(--color-accent); color: var(--color-text-secondary); }
  .preset-chip.active {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-bg-primary);
  }

  .path-btn {
    padding: 2px 8px;
    background: var(--color-bg-surface);
    border: 0.5px solid var(--color-border);
    border-radius: 3px;
    color: var(--color-text-secondary);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
    max-width: 120px;
    overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
    outline: none;
  }
  .path-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }

  .opt {
    display: flex; align-items: center; gap: 4px;
    font-size: 11px;
    color: var(--color-text-secondary);
    cursor: pointer; white-space: nowrap; user-select: none;
  }

  /* --- File list --- */
  .file-list {
    flex: 1; min-height: 0; overflow-y: auto;
    border: 0.5px solid var(--color-border);
    border-radius: 5px; margin-top: 10px;
  }

  .file-item {
    display: flex; align-items: center; gap: 8px;
    padding: 6px 10px;
    border-bottom: 0.5px solid var(--color-border);
    transition: background 0.1s;
  }
  .file-item:last-child { border-bottom: none; }
  .file-item:hover { background-color: var(--color-bg-hover); }

  .file-info { flex: 1; min-width: 0; display: flex; flex-direction: column; gap: 1px; }
  .file-name { font-size: 11px; color: var(--color-text-primary); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .retry-btn {
    display: flex; align-items: center; justify-content: center;
    width: 18px; height: 18px;
    border: none; background: transparent;
    color: var(--color-accent); cursor: pointer;
    border-radius: 3px; padding: 0;
    transition: all 0.15s; outline: none; flex-shrink: 0;
  }
  .retry-btn:hover { background: var(--color-bg-surface); }

  .file-meta { font-size: 10px; color: var(--color-text-muted); }
  .file-process-info {
    font-size: 10px;
    color: var(--color-accent);
    opacity: 0.8;
    display: flex;
    align-items: center;
    gap: 4px;
  }
  .file-time {
    color: var(--color-text-muted);
    font-variant-numeric: tabular-nums;
    flex-shrink: 0;
  }
  .file-error-text {
    color: var(--color-error);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .file-status { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }
  .file-result { font-size: 10px; color: var(--color-text-muted); font-variant-numeric: tabular-nums; }
  .ratio { font-weight: 600; }
  .ratio.positive { color: var(--color-success); }

  .badge { font-size: 10px; padding: 1px 6px; border-radius: 8px; font-weight: 500; white-space: nowrap; }
  .badge-pending { background-color: var(--color-bg-surface); color: var(--color-text-muted); }
  .badge-processing { background-color: color-mix(in srgb, var(--color-accent) 20%, transparent); color: var(--color-accent); }
  .badge-done { background-color: color-mix(in srgb, var(--color-success) 20%, transparent); color: var(--color-success); }
  .badge-error { background-color: color-mix(in srgb, var(--color-error) 20%, transparent); color: var(--color-error); }
  .badge-skipped { background-color: var(--color-bg-surface); color: var(--color-text-muted); }

  /* --- Empty state --- */
  .empty-state {
    flex: 1; display: flex; flex-direction: column;
    align-items: center; justify-content: center;
    gap: 8px; color: var(--color-text-muted);
  }
  .empty-icon { opacity: 0.4; }
  .empty-state p { margin: 0; font-size: 12px; }

  .pulse-dot {
    width: 8px; height: 8px; border-radius: 50%;
    background-color: var(--color-success);
    animation: pulse 1.5s ease-in-out infinite;
  }
  @keyframes pulse {
    0%, 100% { opacity: 0.4; transform: scale(0.8); }
    50% { opacity: 1; transform: scale(1.2); }
  }

  /* --- Status bar --- */
  .status-bar {
    display: flex; align-items: center; justify-content: space-between;
    padding-top: 8px; flex-shrink: 0;
  }
  .status-text { font-size: 11px; color: var(--color-text-muted); }
  .status-actions { display: flex; align-items: center; gap: 8px; }

  .btn-text-danger {
    background: none; border: none;
    color: var(--color-text-muted); cursor: pointer;
    font-size: 11px; padding: 3px 8px; border-radius: 4px;
    transition: all 0.15s; outline: none;
  }
  .btn-text-danger:hover {
    color: var(--color-error);
    background-color: color-mix(in srgb, var(--color-error) 10%, transparent);
  }

  .btn-primary {
    padding: 5px 14px;
    background-color: var(--color-accent);
    color: var(--color-bg-primary);
    border: none; border-radius: 5px;
    font-size: 12px; font-weight: 600;
    cursor: pointer; transition: all 0.15s; outline: none;
  }
  .btn-primary:hover:not(:disabled) { background-color: var(--color-accent-hover); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
