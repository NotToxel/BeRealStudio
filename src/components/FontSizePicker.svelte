<script lang="ts">
  import Minus from 'lucide-svelte/icons/minus';
  import Plus from 'lucide-svelte/icons/plus';

  export let value: number = 100;
  export let label: string = 'Font Size';

  const PRESETS = [
    { label: 'S', size: 60 },
    { label: 'M', size: 80 },
    { label: 'L', size: 100 },
    { label: 'XL', size: 120 },
    { label: '2XL', size: 140 },
  ];

  function increment() {
    value = Math.min(value + 5, 200);
  }

  function decrement() {
    value = Math.max(value - 5, 30);
  }
</script>

<div class="font-size-picker">
  <label class="label" for="font-size-input">{label}</label>

  <div class="controls-row">
    <!-- Preset Pills -->
    <div class="presets-group">
      {#each PRESETS as preset}
        <button
          type="button"
          class="preset-pill"
          class:active={value === preset.size}
          on:click={() => (value = preset.size)}
        >
          {preset.label}
        </button>
      {/each}
    </div>

    <!-- Stepper & Direct Input -->
    <div class="stepper-wrap">
      <button type="button" class="btn-step" on:click={decrement} title="Decrease size">
        <Minus size={13} />
      </button>

      <div class="input-unit-wrap">
        <input
          id="font-size-input"
          type="number"
          class="num-input font-mono"
          bind:value
          min="30"
          max="200"
          step="5"
        />
        <span class="unit-text">px</span>
      </div>

      <button type="button" class="btn-step" on:click={increment} title="Increase size">
        <Plus size={13} />
      </button>
    </div>
  </div>
</div>

<style>
  .font-size-picker {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }

  .label {
    font-size: 13px;
    font-weight: 500;
    color: var(--text-main);
  }

  .controls-row {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
  }

  .presets-group {
    display: flex;
    background: #0f0f13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 3px;
    gap: 2px;
  }

  .preset-pill {
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 600;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .preset-pill:hover {
    color: var(--text-main);
    background: #181820;
  }

  .preset-pill.active {
    background: rgba(139, 92, 246, 0.25);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.4);
    box-shadow: 0 0 8px rgba(139, 92, 246, 0.3);
  }

  .stepper-wrap {
    display: flex;
    align-items: center;
    background: #0f0f13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 2px;
  }

  .btn-step {
    width: 28px;
    height: 28px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .btn-step:hover {
    background: #1c1c24;
    color: var(--text-main);
  }

  .input-unit-wrap {
    display: flex;
    align-items: center;
    padding: 0 4px;
  }

  .num-input {
    width: 44px;
    background: transparent;
    border: none;
    color: var(--text-main);
    text-align: right;
    font-size: 13px;
    font-weight: 600;
    outline: none;
    appearance: textfield;
    -moz-appearance: textfield;
  }

  .num-input::-webkit-inner-spin-button,
  .num-input::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .unit-text {
    font-size: 11px;
    color: var(--text-muted);
    margin-left: 2px;
  }
</style>
