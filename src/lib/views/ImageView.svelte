<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { open } from "@tauri-apps/plugin-dialog";
  import { onMount, onDestroy } from "svelte";
  import { imageFiles, isProcessing, removeFile, clearFiles, imageSelectedIds, imageBatchElapsed } from "$lib/stores/fileQueue";
  import { imageSettings, imagePresets, appSettings, type ImageSettings, type AppSettings } from "$lib/stores/settings";
  import { currentView } from "$lib/stores/navigation";
  import type { MediaFile, ImageInfo, ProcessResult, ScannedFile } from "$lib/types";
  import FileDropZone from "$lib/components/FileDropZone.svelte";
  import FileListItem from "$lib/components/FileListItem.svelte";
  import SegmentControl from "$lib/components/SegmentControl.svelte";
  import { formatSize } from "$lib/utils/fileSize";
  import { splitPath, joinPath, fileName as getFileName, parentDir, splitSegments, pathSep, stripLeadingSep } from "$lib/utils/path";

  let progressUnlisten: (() => void) | null = null;

  function handleGlobalClick(e: MouseEvent) {
    if (showTip && !(e.target as HTMLElement).closest(".tip-btn")) {
      showTip = null;
    }
  }

  onMount(async () => {
    document.addEventListener("click", handleGlobalClick);
    progressUnlisten = await listen<{ fileId: string; progress: number; status: string }>(
      "processing-progress",
      (event) => {
        const { fileId, progress } = event.payload;
        imageFiles.update((files) =>
          files.map((f) => f.id === fileId ? { ...f, progress } : f),
        );
      },
    );

  });

  onDestroy(() => {
    progressUnlisten?.();
    document.removeEventListener("click", handleGlobalClick);
  });

  async function pickOutputDir() {
    const dir = await open({ directory: true, multiple: false });
    if (dir) {
      imageSettings.update((s) => ({ ...s, outputDir: dir as string }));
    }
  }

  const imageExts = ["jpg", "jpeg", "png", "webp", "avif", "gif", "bmp", "tiff"];
  const formatOptions = [
    { value: "original", label: "原格式" },
    { value: "webp", label: "WEBP" },
    { value: "avif", label: "AVIF" },
    { value: "png", label: "PNG" },
    { value: "jpeg", label: "JPG" },
  ];

  let processing = $state(false);
  let showTip = $state<string | null>(null);

  function applyPreset(presetId: string) {
    const p = imagePresets.find((x) => x.id === presetId);
    if (!p) return;
    imageSettings.update((s) => ({
      ...s,
      preset: presetId,
      quality: p.quality,
      resizeWidth: p.resizeWidth,
      resizeHeight: p.resizeWidth ? null : s.resizeHeight,
      targetSize: p.targetSize ?? s.targetSize,
    }));
  }

  // When user manually changes a setting, mark preset as custom
  function manualUpdate(fn: (s: ImageSettings) => ImageSettings) {
    imageSettings.update((s) => ({ ...fn(s), preset: "custom" }));
  }

  async function handleFilesSelected(rawPaths: string[]) {
    const scanned: ScannedFile[] = await invoke("scan_paths", { paths: rawPaths, mediaType: "image" });

    for (const { path, base_dir } of scanned) {
      let exists = false;
      let effectiveBaseDir = base_dir;
      imageFiles.update((files) => {
        const idx = files.findIndex((f) => f.path === path);
        if (idx >= 0) {
          exists = true;
          // Update baseDir if new one is a higher-level ancestor
          const old = files[idx].baseDir;
          if (base_dir.length < old.length && old.startsWith(base_dir)) {
            const updated = [...files];
            updated[idx] = { ...updated[idx], baseDir: base_dir };
            return updated;
          }
          return files;
        }
        // New file: check if an existing baseDir is an ancestor of this file's path
        // If so, use that baseDir so the file joins the existing tree
        for (const f of files) {
          if (f.baseDir && path.startsWith(f.baseDir + "/") && f.baseDir.length < effectiveBaseDir.length) {
            effectiveBaseDir = f.baseDir;
          }
        }
        return files;
      });
      if (exists) continue;

      const name = getFileName(path);
      try {
        const info: ImageInfo = await invoke("get_image_info", { path });
        const file: MediaFile = {
          id: crypto.randomUUID(),
          path,
          name,
          baseDir: effectiveBaseDir,
          originalSize: info.file_size,
          format: info.format,
          width: info.width,
          height: info.height,
          status: "pending",
          mediaType: "image",
        };
        imageFiles.update((files) => [...files, file]);
        imageSelectedIds.update((s) => new Set([...s, file.id]));
        expandFileParents(path);
      } catch (e) {
        const ext = name.split(".").pop()?.toLowerCase() || "unknown";
        const file: MediaFile = {
          id: crypto.randomUUID(),
          path,
          name,
          baseDir: effectiveBaseDir,
          originalSize: 0,
          format: ext,
          status: "error",
          error: String(e),
          mediaType: "image",
        };
        imageFiles.update((files) => [...files, file]);
      }
    }
  }

  function getOutputPath(file: MediaFile, settings: ImageSettings): string {
    const { dir: originalDir, name: fileName, sep } = splitPath(file.path);
    const dotIdx = fileName.lastIndexOf(".");
    const baseName = dotIdx > 0 ? fileName.substring(0, dotIdx) : fileName;
    const originalExt = dotIdx > 0 ? fileName.substring(dotIdx + 1) : "";

    const converting = settings.outputFormat !== "original";
    let ext = originalExt;

    if (converting) {
      ext = settings.outputFormat;
      const lower = ext.toLowerCase();
      if (lower === "jpg" || lower === "jpeg") ext = settings.jpegExtension;
    } else if (settings.fixExtension && file.format) {
      const validJpegExts = new Set(["jpg", "jpeg"]);
      const isOriginalValidJpeg = validJpegExts.has(originalExt.toLowerCase()) && file.format === "jpg";
      if (!isOriginalValidJpeg) {
        ext = file.format;
      }
    }

    if (ext.toLowerCase() === "tif") ext = "tiff";

    const outputDir = settings.outputDir || $appSettings.defaultImageOutputDir;
    if (settings.outputMode === "saveto" && outputDir) {
      if (settings.preserveStructure && file.baseDir) {
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

  function shouldSkip(file: MediaFile, settings: ImageSettings): boolean {
    const sizeMB = file.originalSize / (1024 * 1024);
    if (settings.minSize !== null && sizeMB < settings.minSize) return true;
    if (settings.maxSize !== null && sizeMB > settings.maxSize) return true;
    return false;
  }

  let CONCURRENCY = $derived($appSettings.concurrency);

  async function processOne(i: number, settings: ImageSettings) {
    const files = $imageFiles;
    const file = files[i];

    if (shouldSkip(file, settings)) {
      imageFiles.update((f) => {
        f[i] = { ...f[i], status: "skipped", error: "不符合大小过滤" };
        return [...f];
      });
      return;
    }

    imageFiles.update((f) => {
      f[i] = { ...f[i], status: "processing" };
      return [...f];
    });

    try {
      const outputPath = getOutputPath(file, settings);
      const isQuality = settings.compressMode === "quality";
      const isTarget = settings.compressMode === "target";
      const targetBytes = isTarget ? Math.round(settings.targetSize * 1024 * 1024) : null;

      // Skip unsupported formats (GIF/BMP/TIFF) for lossy/target modes when keeping original format
      const fileFormat = file.format?.toLowerCase() || "";
      const unsupportedLossy = new Set(["gif", "bmp", "tiff"]);
      if (settings.outputFormat === "original" && unsupportedLossy.has(fileFormat)) {
        if (isTarget || (isQuality && settings.quality < 100)) {
          imageFiles.update((f) => {
            f[i] = { ...f[i], status: "skipped", progress: undefined, error: `${fileFormat.toUpperCase()} 不支持${isTarget ? "按大小" : "有损"}压缩，建议转为 JPG/WebP` };
            return [...f];
          });
          return;
        }
      }

      // Compute resize settings once (used in both skip and normal paths)
      const resizeSettings = (() => {
        const mode = settings.resizeMode || "off";
        if (mode === "max") {
          const w = settings.resizeWidth;
          const h = settings.resizeHeight;
          return { resize_enabled: w !== null || h !== null, resize_width: w, resize_height: h };
        }
        if (mode === "percent") {
          const pct = settings.resizePercent ?? 100;
          if (pct < 100 && file.width && file.height) {
            return {
              resize_enabled: true,
              resize_width: Math.round(file.width * pct / 100),
              resize_height: Math.round(file.height * pct / 100),
            };
          }
        }
        return { resize_enabled: false, resize_width: null, resize_height: null };
      })();

      // Skip compression if target size mode and file already fits
      // But still handle extension rename and resize
      if (isTarget && file.originalSize <= targetBytes!) {
        const needsWork = outputPath !== file.path || resizeSettings.resize_enabled;
        if (needsWork) {
          await invoke("process_image", {
            fileId: file.id,
            filePath: file.path,
            outputPath,
            settings: {
              compress_enabled: false,
              quality: 100,
              convert_enabled: settings.outputFormat !== "original",
              output_format: settings.outputFormat,
              ...resizeSettings,
              keep_aspect_ratio: settings.keepAspectRatio,
              target_size: null,
            },
          });
          if (settings.outputMode === "overwrite" && outputPath !== file.path) {
            await invoke("delete_file", { path: file.path }).catch(() => {});
          }
        }
        if (!needsWork) {
          imageFiles.update((f) => {
            f[i] = { ...f[i], status: "skipped", compressedSize: file.originalSize, ratio: 0, outputPath };
            return [...f];
          });
          return;
        }
        // If resize was done, get the actual output size
        const outSize = await invoke<{ file_size: number }>("get_image_info", { path: outputPath });
        imageFiles.update((f) => {
          f[i] = { ...f[i], status: "done", progress: undefined, compressedSize: (outSize as any).file_size, ratio: 1 - (outSize as any).file_size / file.originalSize, outputPath };
          return [...f];
        });
        return;
      }

      const invokeSettings = {
          compress_enabled: isQuality || isTarget,
          quality: isQuality ? settings.quality : 80,
          convert_enabled: settings.outputFormat !== "original",
          output_format: settings.outputFormat,
          ...resizeSettings,
          keep_aspect_ratio: settings.keepAspectRatio,
          target_size: targetBytes,
      };

      const result: ProcessResult = await invoke("process_image", {
        fileId: file.id,
        filePath: file.path,
        outputPath,
        settings: invokeSettings,
      });

      // Overwrite mode: delete original if output path differs
      if (settings.outputMode === "overwrite" && result.output_path !== file.path) {
        await invoke("delete_file", { path: file.path }).catch(() => {});
      }

      const noChange = result.compressed_size === result.original_size && result.ratio === 0 && result.output_path === file.path;
      const missedTarget = isTarget && targetBytes !== null && result.compressed_size > targetBytes;
      imageFiles.update((f) => {
        f[i] = {
          ...f[i],
          status: noChange ? "skipped" : missedTarget ? "done" : "done",
          progress: undefined,
          compressedSize: result.compressed_size,
          ratio: result.ratio,
          outputPath: result.output_path,
          elapsedMs: result.elapsed_ms,
          error: missedTarget ? `PNG 无损压缩无法达到 ≤${settings.targetSize}MB` : undefined,
        };
        return [...f];
      });
    } catch (e) {
      imageFiles.update((f) => {
        f[i] = { ...f[i], status: "error", error: String(e) };
        return [...f];
      });
    }
  }

  async function startProcess() {
    const settings = structuredClone($imageSettings);
    processing = true;
    isProcessing.set(true);
    const startTime = Date.now();

    const files = $imageFiles;
    const pending: number[] = [];
    for (let i = 0; i < files.length; i++) {
      if (files[i].status !== "pending" && files[i].status !== "error") continue;
      if (shouldSkip(files[i], settings)) continue;
      if (!selectedIds.has(files[i].id)) continue;
      pending.push(i);
    }

    // Process with concurrency limit
    let cursor = 0;
    async function next(): Promise<void> {
      while (cursor < pending.length) {
        const idx = pending[cursor++];
        await processOne(idx, settings);
      }
    }

    const workers = Array.from({ length: Math.min(CONCURRENCY, pending.length) }, () => next());
    await Promise.all(workers);

    imageBatchElapsed.set(Date.now() - startTime);
    processing = false;
    isProcessing.set(false);
  }

  function resetFile(index: number) {
    imageFiles.update((files) => {
      const f = files[index];
      if (!f) return files;
      const updated = [...files];
      updated[index] = { ...f, status: "pending", compressedSize: undefined, ratio: undefined, outputPath: undefined, error: undefined, progress: undefined };
      return updated;
    });
  }


  // Selection state (persisted in store across view switches)
  let selectedIds = $derived($imageSelectedIds);

  function toggleFileSelect(id: string) {
    imageSelectedIds.update((s) => {
      const next = new Set(s);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }

  function toggleFolderSelect(node: TreeNode) {
    const ids = collectNodeFileIds(node);
    imageSelectedIds.update((s) => {
      const next = new Set(s);
      const allSelected = ids.every((id) => next.has(id));
      if (allSelected) ids.forEach((id) => next.delete(id));
      else ids.forEach((id) => next.add(id));
      return next;
    });
  }

  function collectNodeFileIds(node: TreeNode): string[] {
    const ids = node.files.map((e) => e.file.id);
    for (const child of node.children) ids.push(...collectNodeFileIds(child));
    return ids;
  }

  function folderCheckState(node: TreeNode): "all" | "some" | "none" {
    const ids = collectNodeFileIds(node);
    if (ids.length === 0) return "none";
    const count = ids.filter((id) => selectedIds.has(id)).length;
    if (count === ids.length) return "all";
    if (count > 0) return "some";
    return "none";
  }

  function selectAll() {
    const all = $imageFiles.map((f) => f.id);
    if (all.every((id) => selectedIds.has(id))) imageSelectedIds.set(new Set());
    else imageSelectedIds.set(new Set(all));
  }

  // Expanded folder paths
  let expandedDirs = $state(new Set<string>());
  function toggleDir(dir: string) {
    if (expandedDirs.has(dir)) expandedDirs.delete(dir);
    else expandedDirs.add(dir);
    expandedDirs = new Set(expandedDirs);
  }

  // Collect dirs that need expanding; apply after all files added
  const pendingExpands = new Set<string>();
  let expandTimer: ReturnType<typeof setTimeout> | null = null;

  function expandFileParents(filePath: string) {
    const fileDir = parentDir(filePath);
    pendingExpands.add(fileDir);
    if (expandTimer) clearTimeout(expandTimer);
    expandTimer = setTimeout(flushExpands, 100);
  }

  function flushExpands() {
    const tree = treeData.tree;
    let changed = false;
    function expandAncestors(nodes: TreeNode[]) {
      for (const node of nodes) {
        for (const dir of pendingExpands) {
          if (dir.startsWith(node.fullPath) || dir === node.fullPath) {
            if (!expandedDirs.has(node.fullPath)) {
              expandedDirs.add(node.fullPath);
              changed = true;
            }
            expandAncestors(node.children);
            break;
          }
        }
      }
    }
    expandAncestors(tree);
    pendingExpands.clear();
    if (changed) expandedDirs = new Set(expandedDirs);
  }

  interface TreeNode {
    name: string;
    fullPath: string;
    depth: number;
    files: { file: MediaFile; index: number }[];
    children: TreeNode[];
    totalFiles: number;
  }

  interface TreeData {
    tree: TreeNode[];
    rootFiles: { file: MediaFile; index: number }[];
    hasTree: boolean;
  }

  function buildTree(files: MediaFile[]): TreeData {
    const rootFiles: { file: MediaFile; index: number }[] = [];
    const folderFileEntries: { file: MediaFile; index: number }[] = [];

    // Separate: individual files (baseDir === file parent) vs folder files
    for (let i = 0; i < files.length; i++) {
      const f = files[i];
      const fileParent = parentDir(f.path);
      if (!f.baseDir || f.baseDir === fileParent) {
        rootFiles.push({ file: f, index: i });
      } else {
        folderFileEntries.push({ file: f, index: i });
      }
    }

    // Merge rootFiles into tree if:
    // 1. rootFiles themselves come from multiple directories, OR
    // 2. rootFiles' directory differs from folderFiles' directories (different tree)
    if (rootFiles.length > 0) {
      const rootDirs = new Set(rootFiles.map((e) => parentDir(e.file.path)));
      let shouldMerge = rootDirs.size > 1;

      if (!shouldMerge && folderFileEntries.length > 0) {
        // Check if rootFiles share a common ancestor with folderFiles
        const rootDir = [...rootDirs][0];
        const folderDirs = folderFileEntries.map((e) => parentDir(e.file.path));
        const anyShared = folderDirs.some((fd) => fd.startsWith(rootDir + "/") || rootDir.startsWith(fd + "/") || fd === rootDir);
        if (!anyShared) shouldMerge = true;
      }

      if (shouldMerge) {
        folderFileEntries.push(...rootFiles);
        rootFiles.length = 0;
      }
    }

    if (folderFileEntries.length === 0) {
      return { tree: [], rootFiles, hasTree: false };
    }

    // Find common root: longest common directory prefix among folder files
    const allDirs = folderFileEntries.map((e) => parentDir(e.file.path));
    const sep = pathSep(allDirs[0] || "/");
    let commonRoot = allDirs[0] || "";
    for (let i = 1; i < allDirs.length; i++) {
      while (commonRoot && !allDirs[i].startsWith(commonRoot + sep) && allDirs[i] !== commonRoot) {
        const idx = Math.max(commonRoot.lastIndexOf("/"), commonRoot.lastIndexOf("\\"));
        if (idx <= 0) { commonRoot = ""; break; }
        commonRoot = commonRoot.substring(0, idx);
      }
    }
    // Count distinct first-level children under commonRoot
    const childNames = new Set<string>();
    for (const d of allDirs) {
      const rel = d === commonRoot ? "" : stripLeadingSep(d.slice(commonRoot.length));
      const first = splitSegments(rel)[0];
      if (first) childNames.add(first);
    }
    // Only go up one level if there's a single child folder
    if (childNames.size <= 1) {
      const idx = Math.max(commonRoot.lastIndexOf("/"), commonRoot.lastIndexOf("\\"));
      if (idx > 0) commonRoot = commonRoot.substring(0, idx);
    }

    // Group folder files by directory
    const dirMap = new Map<string, { file: MediaFile; index: number }[]>();
    for (const entry of folderFileEntries) {
      const fileDir = parentDir(entry.file.path);
      if (!dirMap.has(fileDir)) dirMap.set(fileDir, []);
      dirMap.get(fileDir)!.push(entry);
    }

    // Build nested tree
    const tree: TreeNode[] = [];
    const nodeMap = new Map<string, TreeNode>();

    for (const fullDir of [...dirMap.keys()].sort()) {
      const rel = fullDir.startsWith(commonRoot)
        ? stripLeadingSep(fullDir.slice(commonRoot.length))
        : fullDir;
      const segments = rel ? splitSegments(rel) : [];
      if (segments.length === 0) continue;

      let currentPath = commonRoot;
      let parentChildren = tree;

      for (let d = 0; d < segments.length; d++) {
        currentPath = currentPath ? `${currentPath}${sep}${segments[d]}` : segments[d];
        let node = nodeMap.get(currentPath);
        if (!node) {
          node = { name: segments[d], fullPath: currentPath, depth: d, files: [], children: [], totalFiles: 0 };
          nodeMap.set(currentPath, node);
          parentChildren.push(node);
        }
        parentChildren = node.children;
      }

      const node = nodeMap.get(currentPath);
      if (node) node.files = dirMap.get(fullDir) || [];
    }

    function countFiles(node: TreeNode): number {
      let total = node.files.length;
      for (const child of node.children) total += countFiles(child);
      node.totalFiles = total;
      return total;
    }
    for (const n of tree) countFiles(n);

    return { tree, rootFiles, hasTree: true };
  }

  let files = $derived($imageFiles);
  let hasFiles = $derived(files.length > 0);
  let treeData = $derived(buildTree(files));
  let filteredCount = $derived(files.filter((f) => shouldSkip(f, $imageSettings)).length);
  let pendingCount = $derived(files.filter((f) => f.status === "pending" && !shouldSkip(f, $imageSettings)).length);
  // Files that will actually be processed when clicking the button
  let actionableCount = $derived(files.filter((f) => {
    if (!selectedIds.has(f.id)) return false;
    if (f.status !== "pending" && f.status !== "error") return false;
    if (shouldSkip(f, $imageSettings)) return false;
    return true;
  }).length);
  let doneCount = $derived(files.filter((f) => f.status === "done").length);
  let skippedCount = $derived(files.filter((f) => f.status === "skipped").length);
  let totalSaved = $derived(files.reduce((acc, f) => {
    if (f.status === "done" && f.compressedSize !== undefined) {
      return acc + (f.originalSize - f.compressedSize);
    }
    return acc;
  }, 0));
  let batchElapsed = $derived($imageBatchElapsed);

  function formatElapsed(ms: number): string {
    if (ms < 1000) return `${ms}ms`;
    return `${(ms / 1000).toFixed(1)}s`;
  }
</script>

{#snippet renderTree(nodes: TreeNode[])}
  {#each nodes as node (node.fullPath)}
    {@const checkState = folderCheckState(node)}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div class="dir-row" style="padding-left:{12 + node.depth * 16}px" onclick={() => toggleDir(node.fullPath)}>
      <span class="dir-check" onclick={(e) => { e.stopPropagation(); toggleFolderSelect(node); }}>
        <input type="checkbox" checked={checkState === "all"} indeterminate={checkState === "some"} />
      </span>
      <span class="dir-toggle">
        <svg width="10" height="10" viewBox="0 0 10 10" class="dir-chevron" class:open={expandedDirs.has(node.fullPath)}>
          <path d="M3.5 2.5l3 2.5-3 2.5" fill="none" stroke="currentColor" stroke-width="1.3"/>
        </svg>
        <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="dir-folder-icon">
          <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>
        </svg>
        <span class="dir-name">{node.name}</span>
        <span class="dir-count">{node.totalFiles}</span>
      </span>
    </div>
    {#if expandedDirs.has(node.fullPath)}
      {#if node.children.length > 0}
        {@render renderTree(node.children)}
      {/if}
      {#each node.files as { file, index } (file.id)}
        {@const skipped = shouldSkip(file, $imageSettings)}
        {#if !skipped || !$imageSettings.hideFiltered}
          <div style="padding-left:{(node.depth + 1) * 16}px">
            <FileListItem
              {file}
              dimmed={skipped}
              selected={selectedIds.has(file.id)}

              onToggleSelect={() => toggleFileSelect(file.id)}
              onRemove={() => removeFile(imageFiles, index)}
              onReset={() => resetFile(index)}
            />
          </div>
        {/if}
      {/each}
    {/if}
  {/each}
{/snippet}

<div class="view">
  {#if !hasFiles}
    <FileDropZone
      onFilesSelected={handleFilesSelected}
      accept={imageExts}
      hint="支持 JPG, PNG, WebP, AVIF, GIF, BMP, TIFF"
    />
  {:else}
    <!-- Row 1: add + format + compress + clear -->
    <div class="toolbar">
      <div class="toolbar-left">
        <span class="toolbar-group">
          <FileDropZone onFilesSelected={handleFilesSelected} accept={imageExts} hint="" compact />
        </span>
        <div class="toolbar-sep"></div>
        <span class="toolbar-group">
          <SegmentControl
            options={formatOptions}
            selected={$imageSettings.outputFormat}
            onchange={(v) => imageSettings.update((s) => ({
              ...s,
              outputFormat: v,
              compressMode: v === "png" && s.compressMode === "target" ? "off" : s.compressMode,
              quality: v === "png" && s.outputFormat !== "png" ? 100 : (v !== "png" && s.outputFormat === "png" && s.quality === 100 ? 80 : s.quality),
            }))}
          />
        </span>
        <div class="toolbar-sep"></div>
        <span class="toolbar-group">
          <SegmentControl
            options={$imageSettings.outputFormat === "png"
              ? [
                  { value: "off", label: "原图" },
                  { value: "quality", label: "压缩" },
                ]
              : [
                  { value: "off", label: "原图" },
                  { value: "quality", label: "压缩" },
                  { value: "target", label: "按大小" },
                ]
            }
            selected={($imageSettings.outputFormat === "png" && $imageSettings.compressMode === "target") ? "off" : ($imageSettings.compressMode === "lossless" ? "quality" : $imageSettings.compressMode)}
            onchange={(v) => imageSettings.update((s) => ({ ...s, compressMode: v as "off" | "quality" | "target" }))}
          />
          {#if $imageSettings.compressMode === "quality" || $imageSettings.compressMode === "lossless"}
            <input type="range" class="slider" min="1" max="100"
              value={$imageSettings.quality}
              oninput={(e) => manualUpdate((s) => ({ ...s, quality: Number((e.target as HTMLInputElement).value) }))}
            />
            <span class="field-value">{$imageSettings.quality === 100 ? "无损" : `${$imageSettings.quality}%`}</span>
            <button class="tip-btn" onclick={() => showTip = showTip === "quality" ? null : "quality"}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/>
              </svg>
            </button>
            {#if showTip === "quality"}
              <div class="tip-popover">
                {#if $imageSettings.quality === 100}
                  只减小文件体积，画质不变
                {:else}
                  数值越低文件越小，画质越低。GIF/BMP/TIFF 格式不支持有损压缩，将跳过
                {/if}
              </div>
            {/if}
          {:else if $imageSettings.compressMode === "target"}
            <div class="target-size-group">
              <span class="field-value">≤</span>
              <input type="text" inputmode="decimal" class="num-input num-input-target" placeholder="1"
                value={$imageSettings.targetSize}
                oninput={(e) => {
                  const raw = (e.target as HTMLInputElement).value;
                  const v = raw.replace(/[^\d.]/g, "");
                  if (v !== raw) (e.target as HTMLInputElement).value = v;
                  if (v === "" || v === ".") return;
                  if (v.endsWith(".")) return;
                  const n = parseFloat(v);
                  if (!isNaN(n) && n > 0) imageSettings.update((s) => ({ ...s, targetSize: n }));
                }}
              />
              <span class="field-label">MB</span>
            </div>
            <button class="tip-btn" onclick={() => showTip = showTip === "target" ? null : "target"}>
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/>
              </svg>
            </button>
            {#if showTip === "target"}
              <div class="tip-popover">
                自动搜索最优质量压缩到目标大小以内。PNG 可能无法达到目标大小。GIF/BMP/TIFF 格式不支持，将跳过
              </div>
            {/if}
          {/if}
        </span>
      </div>
      <button class="btn-text-danger" onclick={() => { clearFiles(imageFiles); imageSelectedIds.set(new Set()); imageBatchElapsed.set(0); }}>清空</button>
    </div>

    <!-- Row 2: size + filter + options -->
    <div class="toolbar-secondary">
      <div class="inline-group">
        <span class="field-label">过滤</span>
        <input type="text" inputmode="decimal" class="num-input num-input-sm" placeholder="最小MB"
          value={$imageSettings.minSize ?? ""}
          oninput={(e) => {
            const raw = (e.target as HTMLInputElement).value;
            const v = raw.replace(/[^\d.]/g, "");
            if (v !== raw) (e.target as HTMLInputElement).value = v;
            if (v === "" || v === ".") { imageSettings.update((s) => ({ ...s, minSize: null })); return; }
            if (v.endsWith(".")) return;
            const n = parseFloat(v);
            if (!isNaN(n)) imageSettings.update((s) => ({ ...s, minSize: n }));
          }}
        />
        <span class="x-sep">~</span>
        <input type="text" inputmode="decimal" class="num-input num-input-sm" placeholder="最大MB"
          value={$imageSettings.maxSize ?? ""}
          oninput={(e) => {
            const raw = (e.target as HTMLInputElement).value;
            const v = raw.replace(/[^\d.]/g, "");
            if (v !== raw) (e.target as HTMLInputElement).value = v;
            if (v === "" || v === ".") { imageSettings.update((s) => ({ ...s, maxSize: null })); return; }
            if (v.endsWith(".")) return;
            const n = parseFloat(v);
            if (!isNaN(n)) imageSettings.update((s) => ({ ...s, maxSize: n }));
          }}
        />
        <label class="opt">
          <input type="checkbox" checked={$imageSettings.hideFiltered}
            onchange={(e) => imageSettings.update((s) => ({ ...s, hideFiltered: (e.target as HTMLInputElement).checked }))} />
          隐藏
        </label>
      </div>

      <div class="sep"></div>

      <div class="inline-group">
        <span class="field-label">缩放</span>
        <SegmentControl
          options={[
            { value: "off", label: "关" },
            { value: "max", label: "最大尺寸" },
            { value: "percent", label: "百分比" },
          ]}
          selected={$imageSettings.resizeMode}
          onchange={(v) => imageSettings.update((s) => ({ ...s, resizeMode: v as "off" | "max" | "percent" }))}
        />
        {#if $imageSettings.resizeMode === "max"}
          <input type="text" inputmode="decimal" class="num-input num-input-sm" placeholder="宽"
            value={$imageSettings.resizeWidth ?? ""}
            oninput={(e) => {
              const raw = (e.target as HTMLInputElement).value;
              const v = raw.replace(/[^\d]/g, "");
              if (v !== raw) (e.target as HTMLInputElement).value = v;
              if (v === "") { imageSettings.update((s) => ({ ...s, resizeWidth: null })); return; }
              const n = parseInt(v);
              if (!isNaN(n) && n > 0) imageSettings.update((s) => ({ ...s, resizeWidth: n }));
            }}
          />
          <span class="x-sep">×</span>
          <input type="text" inputmode="decimal" class="num-input num-input-sm" placeholder="高"
            value={$imageSettings.resizeHeight ?? ""}
            oninput={(e) => {
              const raw = (e.target as HTMLInputElement).value;
              const v = raw.replace(/[^\d]/g, "");
              if (v !== raw) (e.target as HTMLInputElement).value = v;
              if (v === "") { imageSettings.update((s) => ({ ...s, resizeHeight: null })); return; }
              const n = parseInt(v);
              if (!isNaN(n) && n > 0) imageSettings.update((s) => ({ ...s, resizeHeight: n }));
            }}
          />
          <span class="field-hint">px</span>
        {:else if $imageSettings.resizeMode === "percent"}
          <input type="range" class="slider" min="10" max="100" step="5"
            value={$imageSettings.resizePercent}
            oninput={(e) => imageSettings.update((s) => ({ ...s, resizePercent: Number((e.target as HTMLInputElement).value) }))}
          />
          <span class="field-value">{$imageSettings.resizePercent}%</span>
        {/if}
      </div>

      <div class="sep"></div>

      <div class="inline-group">
        <span class="field-label">输出</span>
        <SegmentControl
          options={[
            { value: "overwrite", label: "覆盖" },
            { value: "saveto", label: "另存到" },
          ]}
          selected={$imageSettings.outputMode}
          onchange={(v) => imageSettings.update((s) => ({ ...s, outputMode: v as "overwrite" | "saveto" }))}
        />
        {#if $imageSettings.outputMode === "saveto"}
          <button class="path-btn" onclick={pickOutputDir} title={$imageSettings.outputDir || "点击选择目录"}>
            {$imageSettings.outputDir ? getFileName($imageSettings.outputDir) : $appSettings.defaultImageOutputDir ? `默认: ${getFileName($appSettings.defaultImageOutputDir)}` : "选择目录..."}
          </button>
          {#if $imageSettings.outputDir}
            <button class="icon-btn" title="清除"
              onclick={() => imageSettings.update((s) => ({ ...s, outputDir: "" }))}>
              <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
            </button>
          {/if}
          <label class="opt">
            <input type="checkbox" checked={$imageSettings.preserveStructure}
              onchange={(e) => imageSettings.update((s) => ({ ...s, preserveStructure: (e.target as HTMLInputElement).checked }))} />
            保留层级
          </label>
        {/if}
      </div>

      <div class="sep"></div>

      <label class="opt">
        <input type="checkbox" checked={$imageSettings.fixExtension}
          onchange={(e) => imageSettings.update((s) => ({ ...s, fixExtension: (e.target as HTMLInputElement).checked }))} />
        纠正错误扩展名
      </label>
      <SegmentControl
        options={[
          { value: "jpg", label: ".jpg" },
          { value: "jpeg", label: ".jpeg" },
        ]}
        selected={$imageSettings.jpegExtension}
        onchange={(v) => imageSettings.update((s) => ({ ...s, jpegExtension: v as "jpg" | "jpeg" }))}
      />
    </div>

    <!-- File list -->
    <div class="file-list">
      <!-- Folder tree -->
      {#if treeData.hasTree}
        {@render renderTree(treeData.tree)}
      {/if}
      <!-- Root-level files (individually added) -->
      {#each treeData.rootFiles as { file, index } (file.id)}
        {@const skipped = shouldSkip(file, $imageSettings)}
        {#if !skipped || !$imageSettings.hideFiltered}
          <FileListItem
            {file}
            dimmed={skipped}
            selected={selectedIds.has(file.id)}
            onToggleSelect={() => toggleFileSelect(file.id)}
            onRemove={() => removeFile(imageFiles, index)}
            onReset={() => resetFile(index)}
          />
        {/if}
      {/each}
    </div>

    <!-- Status bar -->
    <div class="status-bar">
      <span class="status-text">
        {files.length} 个文件
        {#if selectedIds.size > 0} · 已选 {selectedIds.size}{/if}
        {#if filteredCount > 0} · {filteredCount} 已过滤{/if}
        {#if doneCount > 0} · {doneCount} 已完成{/if}
        {#if skippedCount > 0} · {skippedCount} 已跳过{/if}
        {#if totalSaved > 0} · 节省 {formatSize(totalSaved)}{/if}
        {#if batchElapsed > 0} · 耗时 {formatElapsed(batchElapsed)}{/if}
      </span>
      <button class="btn-primary" onclick={startProcess}
        disabled={processing || actionableCount === 0}>
        {#if processing}
          处理中...
        {:else}
          开始处理 ({actionableCount})
        {/if}
      </button>
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
  }

  /* ---- Preset bar ---- */
  .preset-bar {
    display: flex;
    align-items: center;
    gap: 4px;
    padding-bottom: 8px;
    flex-shrink: 0;
  }

  .preset-bar .field-label {
    margin-right: 4px;
  }

  .preset-btn {
    padding: 3px 10px;
    border: 1px solid var(--color-border);
    border-radius: 12px;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: all 0.15s;
    white-space: nowrap;
  }
  .preset-btn:hover {
    border-color: var(--color-accent);
    color: var(--color-text-secondary);
  }
  .preset-btn.active {
    background-color: var(--color-accent);
    border-color: var(--color-accent);
    color: var(--color-bg-primary);
  }

  .preset-custom {
    font-size: 10px;
    color: var(--color-text-muted);
    font-style: italic;
    margin-left: 4px;
  }

  /* ---- Toolbar row 1 ---- */
  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 0;
    flex-shrink: 0;
    flex-wrap: wrap;
    gap: 4px;
  }

  .toolbar-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 1;
    min-width: 0;
    position: relative;
    flex-wrap: wrap;
  }

  .toolbar-group {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .toolbar-sep {
    width: 1px;
    height: 18px;
    background-color: var(--color-border);
    flex-shrink: 0;
  }

  /* ---- Toolbar row 2 ---- */
  .toolbar-secondary {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 0 8px;
    border-bottom: 0.5px solid var(--color-border);
    flex-shrink: 0;
    flex-wrap: wrap;
  }

  /* ---- Shared inline ---- */
  .inline-group {
    display: flex;
    align-items: center;
    gap: 5px;
    flex-shrink: 0;
  }

  .field-label {
    font-size: 11px;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .field-value {
    font-size: 11px;
    color: var(--color-accent);
    font-weight: 600;
    min-width: 32px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .sep {
    width: 1px;
    height: 14px;
    background-color: var(--color-border);
    flex-shrink: 0;
  }

  .x-sep {
    color: var(--color-text-muted);
    font-size: 10px;
  }

  /* Slider */
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
    width: 12px;
    height: 12px;
    border-radius: 50%;
    background: var(--color-accent);
    cursor: pointer;
    transition: transform 0.1s;
  }
  .slider::-webkit-slider-thumb:hover { transform: scale(1.15); }

  /* Number inputs */
  .num-input {
    width: 56px;
    padding: 2px 4px;
    background: var(--color-bg-surface);
    border: 0.5px solid var(--color-border);
    border-radius: 3px;
    color: var(--color-text-primary);
    font-size: 11px;
    outline: none;
    text-align: center;
  }
  .num-input:focus { border-color: var(--color-accent); }
  .num-input::placeholder { color: var(--color-text-muted); }
  .num-input-sm { width: 62px; }
  .num-input-target { width: 42px; }

  .field-hint {
    font-size: 10px;
    color: var(--color-text-muted);
    opacity: 0.6;
    flex-shrink: 0;
  }

  .tip-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    cursor: pointer;
    border-radius: 3px;
    padding: 0;
    flex-shrink: 0;
    outline: none;
    transition: all 0.15s;
    position: relative;
  }
  .tip-btn:hover { color: var(--color-text-secondary); }

  .tip-popover {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 4px;
    padding: 6px 10px;
    background: var(--color-bg-secondary);
    border: 0.5px solid var(--color-border);
    border-radius: 6px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
    font-size: 11px;
    color: var(--color-text-secondary);
    white-space: normal;
    width: 220px;
    line-height: 1.5;
    z-index: 50;
    backdrop-filter: blur(12px);
    -webkit-backdrop-filter: blur(12px);
  }

  .target-size-group {
    display: flex;
    align-items: center;
    gap: 3px;
    flex-shrink: 0;
  }

  .icon-btn {
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
    padding: 0;
    transition: all 0.15s;
  }
  .icon-btn:hover { background-color: var(--color-bg-hover); }
  .icon-btn.active { color: var(--color-accent); }

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
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .path-btn:hover { border-color: var(--color-accent); color: var(--color-accent); }

  .opt {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--color-text-secondary);
    cursor: pointer;
    white-space: nowrap;
    user-select: none;
  }

  .btn-text-danger {
    background: none;
    border: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font-size: 11px;
    padding: 3px 8px;
    border-radius: 4px;
    transition: all 0.15s;
    flex-shrink: 0;
  }
  .btn-text-danger:hover {
    color: var(--color-error);
    background-color: color-mix(in srgb, var(--color-error) 10%, transparent);
  }

  /* ---- File list ---- */
  .file-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    border: 0.5px solid var(--color-border);
    border-radius: 5px;
    margin-top: 8px;
  }

  .dir-row {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 5px 10px;
    border-bottom: 0.5px solid var(--color-border);
    background: var(--color-bg-surface);
    color: var(--color-text-primary);
    font-size: 11px;
    font-weight: 500;
    transition: background 0.1s;
  }
  .dir-row:hover { background: var(--color-bg-hover); }
  .dir-row { cursor: pointer; }

  .dir-check {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    cursor: pointer;
  }
  .dir-check input {
    margin: 0;
    cursor: pointer;
  }

  .dir-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 0;
    cursor: pointer;
  }

  .dir-chevron {
    flex-shrink: 0;
    color: var(--color-text-muted);
    transition: transform 0.15s;
  }
  .dir-chevron.open { transform: rotate(90deg); }

  .dir-folder-icon { flex-shrink: 0; color: var(--color-text-muted); }

  .dir-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .dir-count {
    font-size: 9px;
    min-width: 16px;
    height: 16px;
    line-height: 16px;
    text-align: center;
    border-radius: 8px;
    background: var(--color-bg-hover);
    color: var(--color-text-muted);
    font-weight: 600;
    flex-shrink: 0;
  }

  /* ---- Status bar ---- */
  .status-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding-top: 8px;
    flex-shrink: 0;
  }

  .status-text {
    font-size: 11px;
    color: var(--color-text-muted);
  }

  .btn-primary {
    padding: 5px 14px;
    background-color: var(--color-accent);
    color: var(--color-bg-primary);
    border: none;
    border-radius: 5px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s;
  }
  .btn-primary:hover:not(:disabled) { background-color: var(--color-accent-hover); }
  .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
</style>
