<script lang="ts">
  import Minus from 'lucide-svelte/icons/minus';
  import Plus from 'lucide-svelte/icons/plus';

  export let label: string;
  export let value: number;
  export let min: number = 0;
  export let max: number = 100;
  export let step: number = 1;
  export let unit: string = '';
  export let presets: { label: string; value: number }[] = [];
  export let accentColor: 'yellow' | 'violet' | 'emerald' | 'cyan' = 'yellow';

  function increment() {
    value = Math.min(+(value + step).toFixed(2), max);
  }

  function decrement() {
    value = Math.max(+(value - step).toFixed(2), min);
  }
</script>

<div class="stepper-component accent-{accentColor}">
  <label class="label" for="num-input">{label}</label>

  <div class="controls-row">
    {#if presets.length > 0}
      <div class="presets-group">
        {#each presets as p}
          <button
            type="button"
            class="preset-pill"
            class:active={value === p.value}
            on:click={() => (value = p.value)}
          >
            {p.label}
          </button>
        {/each}
      </div>
    {/if}

    <div class="stepper-box">
      <button type="button" class="btn-step" on:click={decrement} title="Decrease">
        <Minus size={13} />
      </button>

      <div class="input-wrap">
        <input
          id="num-input"
          type="number"
          class="num-input font-mono"
          bind:value
          {min}
          {max}
          {step}
        />
        {#if unit}
          <span class="unit-text">{unit}</span>
        {/if}
      </div>

      <button type="button" class="btn-step" on:click={increment} title="Increase">
        <Plus size={13} />
      </button>
    </div>
  </div>
</div>

<style>
  .stepper-component {
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

  .accent-yellow .preset-pill.active {
    background: rgba(255, 230, 0, 0.18);
    color: #ffe600;
    border: 1px solid rgba(255, 230, 0, 0.35);
  }

  .accent-violet .preset-pill.active {
    background: rgba(139, 92, 246, 0.2);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.4);
  }

  .accent-cyan .preset-pill.active {
    background: rgba(56, 189, 248, 0.2);
    color: #38bdf8;
    border: 1px solid rgba(56, 189, 248, 0.4);
  }

  .accent-emerald .preset-pill.active {
    background: rgba(16, 185, 129, 0.2);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.4);
  }

  .stepper-box {
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

  .input-wrap {
    display: flex;
    align-items: center;
    padding: 0 4px;
  }

  .num-input {
    width: 46px;
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
    font-size: 11.5px;
    color: var(--text-muted);
    margin-left: 2px;
  }
</style>
