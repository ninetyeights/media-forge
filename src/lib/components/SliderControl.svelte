<script lang="ts">
  interface Props {
    label: string;
    value: number;
    min: number;
    max: number;
    step?: number;
    unit?: string;
    onchange: (value: number) => void;
  }

  let { label, value, min, max, step = 1, unit = "", onchange }: Props = $props();
</script>

<div class="slider-control">
  <div class="slider-header">
    <span class="slider-label">{label}</span>
    <span class="slider-value">{value}{unit}</span>
  </div>
  <input
    type="range"
    class="slider"
    {min} {max} {step}
    {value}
    oninput={(e) => onchange(Number((e.target as HTMLInputElement).value))}
  />
</div>

<style>
  .slider-control {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .slider-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .slider-label {
    font-size: 11px;
    color: var(--color-text-secondary);
  }

  .slider-value {
    font-size: 12px;
    color: var(--color-accent);
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .slider {
    width: 100%;
    height: 4px;
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
    transition: transform 0.1s;
  }

  .slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }
</style>
