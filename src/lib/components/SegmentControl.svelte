<script lang="ts">
  interface Props {
    options: { value: string; label: string }[];
    selected: string;
    onchange: (value: string) => void;
  }

  let { options, selected, onchange }: Props = $props();
</script>

<div class="segment">
  {#each options as opt}
    <button
      class="segment-btn"
      class:active={selected === opt.value}
      onclick={() => onchange(opt.value)}
    >
      {opt.label}
    </button>
  {/each}
</div>

<style>
  .segment {
    display: flex;
    background-color: var(--color-bg-surface);
    border-radius: 5px;
    padding: 2px;
    gap: 1px;
  }

  .segment-btn {
    padding: 3px 10px;
    border: none;
    background: transparent;
    color: var(--color-text-muted);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    border-radius: 4px;
    transition: all 0.15s;
    white-space: nowrap;
  }

  .segment-btn:hover:not(.active) {
    color: var(--color-text-secondary);
  }

  .segment-btn.active {
    background-color: var(--color-accent);
    color: var(--color-bg-primary);
  }
</style>
