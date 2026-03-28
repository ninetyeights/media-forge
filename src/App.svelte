<script lang="ts">
  import { isMacOS } from "$lib/platform";
  import { currentView } from "$lib/stores/navigation";
  import TitleBar from "$lib/components/TitleBar.svelte";
  import Sidebar from "$lib/components/Sidebar.svelte";
  import ImageView from "$lib/views/ImageView.svelte";
  import VideoView from "$lib/views/VideoView.svelte";
  import AudioView from "$lib/views/AudioView.svelte";
  import WatermarkView from "$lib/views/WatermarkView.svelte";
  import WatcherView from "$lib/views/WatcherView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";
</script>

<div class="app-container">
  <TitleBar />
  <div class="app-body">
    <Sidebar />
    <main class="main-content">
      {#if $currentView === "image"}
        <ImageView />
      {:else if $currentView === "video"}
        <VideoView />
      {:else if $currentView === "audio"}
        <AudioView />
      {:else if $currentView === "watermark"}
        <WatermarkView />
      {:else if $currentView === "watcher"}
        <WatcherView />
      {:else if $currentView === "settings"}
        <SettingsView />
      {/if}
    </main>
  </div>
</div>

<style>
  .app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
    background-color: var(--color-bg-primary);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
  }

  .app-body {
    display: flex;
    flex: 1;
    overflow: hidden;
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
  }
</style>
