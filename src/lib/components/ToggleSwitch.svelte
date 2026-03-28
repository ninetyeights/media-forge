<script lang="ts">
  interface Props {
    label: string;
    checked: boolean;
    onchange: (value: boolean) => void;
  }

  let { label, checked, onchange }: Props = $props();
</script>

<label class="toggle">
  <span class="toggle-label">{label}</span>
  <button
    class="toggle-track"
    class:on={checked}
    onclick={() => onchange(!checked)}
    role="switch"
    aria-checked={checked}
  >
    <span class="toggle-thumb"></span>
  </button>
</label>

<style>
  .toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    cursor: pointer;
  }

  .toggle-label {
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .toggle-track {
    position: relative;
    width: 32px;
    height: 18px;
    border-radius: 9px;
    background-color: var(--color-bg-hover);
    border: none;
    cursor: pointer;
    transition: background-color 0.2s;
    padding: 0;
    flex-shrink: 0;
  }

  .toggle-track.on {
    background-color: var(--color-accent);
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: white;
    transition: transform 0.2s;
  }

  .toggle-track.on .toggle-thumb {
    transform: translateX(14px);
  }
</style>
