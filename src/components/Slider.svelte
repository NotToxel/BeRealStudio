<script lang="ts">
  export let label: string;
  export let value: number;
  export let min: number = 0;
  export let max: number = 100;
  export let step: number = 1;
  export let unit: string = '';
  export let disabled: boolean = false;
  export let accentColor: 'yellow' | 'violet' | 'emerald' | 'cyan' = 'yellow';

  $: percent = Math.min(Math.max(((value - min) / (max - min)) * 100, 0), 100);
</script>

<div class="slider-group" class:disabled>
  <div class="header">
    <label class="label" for="slider-input">{label}</label>
    <span class="value-badge badge-{accentColor}">{value}{unit}</span>
  </div>

  <div class="slider-track-container">
    <div
      class="slider-fill fill-{accentColor}"
      style="width: {percent}%;"
    ></div>
    <input
      id="slider-input"
      type="range"
      class="range-input input-{accentColor}"
      bind:value
      {min}
      {max}
      {step}
      {disabled}
    />
  </div>
</div>

<style>
  .slider-group {
    display: flex;
    flex-direction: column;
    gap: 8px;
    width: 100%;
  }

  .slider-group.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-main);
  }

  .value-badge {
    font-size: 11px;
    font-family: var(--font-mono);
    font-weight: 600;
    padding: 2px 8px;
    border-radius: var(--radius-full);
  }

  .badge-yellow {
    background: rgba(255, 230, 0, 0.15);
    color: #ffe600;
    border: 1px solid rgba(255, 230, 0, 0.3);
  }
  .badge-violet {
    background: rgba(139, 92, 246, 0.15);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.3);
  }
  .badge-emerald {
    background: rgba(52, 211, 153, 0.15);
    color: #34d399;
    border: 1px solid rgba(52, 211, 153, 0.3);
  }
  .badge-cyan {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    border: 1px solid rgba(56, 189, 248, 0.3);
  }

  .slider-track-container {
    position: relative;
    height: 8px;
    background: #1c1c24;
    border-radius: var(--radius-full);
    display: flex;
    align-items: center;
  }

  .slider-fill {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    border-radius: var(--radius-full);
    pointer-events: none;
  }

  .fill-yellow {
    background: linear-gradient(90deg, #f59e0b 0%, #ffe600 100%);
    box-shadow: 0 0 8px rgba(255, 230, 0, 0.4);
  }
  .fill-violet {
    background: linear-gradient(90deg, #6366f1 0%, #a78bfa 100%);
    box-shadow: 0 0 8px rgba(139, 92, 246, 0.4);
  }
  .fill-emerald {
    background: linear-gradient(90deg, #059669 0%, #34d399 100%);
    box-shadow: 0 0 8px rgba(52, 211, 153, 0.4);
  }
  .fill-cyan {
    background: linear-gradient(90deg, #0284c7 0%, #38bdf8 100%);
    box-shadow: 0 0 8px rgba(56, 189, 248, 0.4);
  }

  .range-input {
    position: absolute;
    width: 100%;
    height: 100%;
    opacity: 0;
    cursor: pointer;
    margin: 0;
    z-index: 2;
  }
</style>
