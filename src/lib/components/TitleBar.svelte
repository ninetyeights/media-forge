<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { isMacOS } from "$lib/platform";

  const appWindow = getCurrentWindow();

  let isMaximized = $state(false);

  async function toggleMaximize() {
    await appWindow.toggleMaximize();
    isMaximized = await appWindow.isMaximized();
  }

  appWindow.onResized(async () => {
    isMaximized = await appWindow.isMaximized();
  });
</script>

<!-- macOS uses native Overlay titlebar, no custom bar needed -->
{#if !isMacOS}
  <div class="titlebar" data-tauri-drag-region>
    <span class="titlebar-title" data-tauri-drag-region>Media Forge</span>
    <div class="win-buttons">
      <button class="win-btn" onclick={() => appWindow.minimize()} aria-label="最小化">
        <svg width="10" height="1" viewBox="0 0 10 1"><rect width="10" height="1" fill="currentColor"/></svg>
      </button>
      <button class="win-btn" onclick={toggleMaximize} aria-label="最大化">
        {#if isMaximized}
          <svg width="10" height="10" viewBox="0 0 10 10"><path d="M2 0h6v2h2v6H8v2H0V4h2V0zm1 1v2h5v5h1V2H3zm-2 3v5h6V4H1z" fill="currentColor"/></svg>
        {:else}
          <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0.5" y="0.5" width="9" height="9" stroke="currentColor" fill="none" stroke-width="1"/></svg>
        {/if}
      </button>
      <button class="win-btn win-close" onclick={() => appWindow.close()} aria-label="关闭">
        <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1l-8 8" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
    </div>
  </div>
{/if}

<style>
  .titlebar {
    display: flex;
    align-items: center;
    height: 36px;
    padding: 0 12px;
    flex-shrink: 0;
    gap: 8px;
    background-color: var(--color-bg-secondary);
    border-bottom: 0.5px solid var(--color-border);
    -webkit-app-region: drag;
  }

  .titlebar-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--color-text-secondary);
    pointer-events: none;
  }

  .win-buttons {
    display: flex;
    height: 100%;
    margin-left: auto;
    -webkit-app-region: no-drag;
  }

  .win-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--color-text-secondary);
    cursor: pointer;
    transition: background-color 0.15s;
  }

  .win-btn:hover { background-color: var(--color-bg-hover); }
  .win-close:hover { background-color: var(--color-error); color: white; }
</style>
