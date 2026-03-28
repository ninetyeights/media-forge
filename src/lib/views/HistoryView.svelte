<script lang="ts">
</script>

<div class="view-container">
  <div class="content">
    <h2 class="view-title">历史记录</h2>
    <div class="placeholder">
      <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2" class="placeholder-icon">
        <circle cx="12" cy="12" r="10"/>
        <path d="M12 6v6l4 2"/>
      </svg>
      <p>处理历史记录将在此显示</p>
      <p class="sub">后续版本实现</p>
    </div>
  </div>
</div>

<style>
  .view-container { display: flex; flex: 1; overflow: hidden; }
  .content { flex: 1; display: flex; flex-direction: column; gap: 14px; padding: 14px; }
  .view-title { margin: 0; font-size: 14px; font-weight: 600; color: var(--color-text-primary); }
  .placeholder { display: flex; flex-direction: column; align-items: center; gap: 8px; padding: 40px; color: var(--color-text-muted); }
  .placeholder-icon { opacity: 0.5; }
  .placeholder p { margin: 0; font-size: 12px; }
  .placeholder .sub { font-size: 11px; opacity: 0.6; }
</style>
