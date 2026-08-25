<script lang="ts">
  import Minus from 'lucide-svelte/icons/minus';
  import Plus from 'lucide-svelte/icons/plus';

  export let value: number = 100;
  export let label: string = 'Font Size';

  let isFocused = false;

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
    <div class="stepper-wrap" class:focused={isFocused}>
      <button
        type="button"
        class="btn-step"
        on:click={decrement}
        title="Decrease size"
        disabled={value <= 30}
      >
        <Minus size={11} />
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
          on:focus={() => (isFocused = true)}
          on:blur={() => (isFocused = false)}
        />
        <span class="unit-badge font-mono">px</span>
      </div>

      <button
        type="button"
        class="btn-step"
        on:click={increment}
        title="Increase size"
        disabled={value >= 200}
      >
        <Plus size={11} />
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
    min-width: 0;
    box-sizing: border-box;
  }

  .label {
    font-size: 12.5px;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .controls-row {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: nowrap;
    height: 32px;
    width: 100%;
    box-sizing: border-box;
  }

  .presets-group {
    display: flex;
    background: linear-gradient(180deg, #101015 0%, #0a0a0e 100%);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 2px;
    gap: 1px;
    height: 32px;
    align-items: center;
    flex: 1;
    min-width: 0;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.4);
    box-sizing: border-box;
  }

  .preset-pill {
    flex: 1;
    min-width: 0;
    padding: 3px 2px;
    font-size: 11px;
    font-weight: 600;
    color: var(--text-muted);
    background: transparent;
    border: 1px solid transparent;
    border-radius: calc(var(--radius-sm) - 1px);
    cursor: pointer;
    transition: color 120ms ease, background 120ms ease, border-color 120ms ease;
    white-space: nowrap;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    box-sizing: border-box;
  }

  .preset-pill:hover {
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.05);
  }

  .preset-pill.active {
    background: rgba(139, 92, 246, 0.2);
    color: #c084fc;
    border-color: rgba(139, 92, 246, 0.4);
    box-shadow: 0 0 8px rgba(139, 92, 246, 0.18);
  }

  .stepper-wrap {
    display: flex;
    align-items: center;
    background: linear-gradient(180deg, #121218 0%, #0c0c10 100%);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 2px;
    height: 32px;
    flex-shrink: 0;
    width: 98px;
    box-sizing: border-box;
    transition: border-color 120ms ease, box-shadow 120ms ease;
    box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.3);
  }

  .stepper-wrap:hover {
    border-color: var(--border-medium);
  }

  .stepper-wrap.focused {
    border-color: rgba(139, 92, 246, 0.55);
    box-shadow: 0 0 10px rgba(139, 92, 246, 0.22);
  }

  .btn-step {
    width: 22px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: calc(var(--radius-sm) - 1px);
    transition: all 100ms ease;
    flex-shrink: 0;
    box-sizing: border-box;
  }

  .btn-step:hover:not(:disabled) {
    background: #1e1e28;
    color: var(--text-main);
    border-color: rgba(255, 255, 255, 0.08);
  }

  .btn-step:active:not(:disabled) {
    transform: scale(0.92);
  }

  .btn-step:disabled {
    opacity: 0.25;
    cursor: not-allowed;
  }

  .input-unit-wrap {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0 1px;
    gap: 2px;
    flex: 1;
  }

  .num-input {
    width: 32px;
    background: transparent;
    border: none;
    color: var(--text-main);
    text-align: right;
    font-size: 12.5px;
    font-weight: 700;
    outline: none;
    appearance: textfield;
    -moz-appearance: textfield;
    padding: 0;
  }

  .num-input::-webkit-inner-spin-button,
  .num-input::-webkit-outer-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .unit-badge {
    font-size: 9.5px;
    font-weight: 700;
    color: var(--text-muted);
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 3px;
    border-radius: 3px;
    white-space: nowrap;
  }
</style>
