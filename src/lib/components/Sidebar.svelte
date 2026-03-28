<script lang="ts">
  import { currentView, hasUpdate, type ViewName } from "$lib/stores/navigation";
  import SidebarItem from "./SidebarItem.svelte";

  const mainItems: { id: ViewName; label: string; icon: string }[] = [
    { id: "image", label: "图片", icon: "image" },
    { id: "video", label: "视频", icon: "video" },
    { id: "audio", label: "音频", icon: "audio" },
    { id: "watermark", label: "水印", icon: "watermark" },
  ];

  const bottomItems: { id: ViewName; label: string; icon: string }[] = [
    { id: "watcher", label: "监听", icon: "watcher" },
    { id: "history", label: "历史", icon: "history" },
    { id: "settings", label: "设置", icon: "settings" },
  ];
</script>

<nav class="sidebar">
  <div class="sidebar-main">
    {#each mainItems as item}
      <SidebarItem
        icon={item.icon}
        label={item.label}
        active={$currentView === item.id}
        onClick={() => currentView.set(item.id)}
      />
    {/each}
  </div>

  <div class="sidebar-divider"></div>

  <div class="sidebar-bottom">
    {#each bottomItems as item}
      <SidebarItem
        icon={item.icon}
        label={item.label}
        active={$currentView === item.id}
        badge={item.id === "settings" && $hasUpdate}
        onClick={() => currentView.set(item.id)}
      />
    {/each}
  </div>
</nav>

<style>
  .sidebar {
    width: 64px;
    background-color: var(--color-bg-secondary);
    border-right: 0.5px solid var(--color-border);
    display: flex;
    flex-direction: column;
    padding: 8px 0;
    flex-shrink: 0;
  }

  .sidebar-main {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .sidebar-divider {
    height: 0.5px;
    background-color: var(--color-border);
    margin: 8px 10px;
  }

  .sidebar-bottom {
    display: flex;
    flex-direction: column;
    gap: 2px;
    margin-top: auto;
  }
</style>
