<script lang="ts">
  export let label: string;
  export let description: string = '';
  export let checked: boolean = false;
  export let disabled: boolean = false;
  export let accentColor: 'yellow' | 'violet' | 'emerald' | 'cyan' = 'yellow';
</script>

<label class="toggle-container" class:disabled>
  <div class="text-group">
    <span class="label-text">{label}</span>
    {#if description}
      <span class="desc-text">{description}</span>
    {/if}
  </div>

  <button
    type="button"
    role="switch"
    aria-label={label}
    aria-checked={checked}
    class="switch-btn accent-{accentColor}"
    class:checked
    on:click={() => {
      if (!disabled) checked = !checked;
    }}
    {disabled}
  >
    <span class="thumb"></span>
  </button>
</label>

<style>
  .toggle-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    cursor: pointer;
    user-select: none;
    padding: 6px 0;
  }

  .toggle-container.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .text-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .label-text {
    font-size: 13.5px;
    font-weight: 500;
    color: var(--text-main);
  }

  .desc-text {
    font-size: 12px;
    color: var(--text-secondary);
    line-height: 1.4;
  }

  .switch-btn {
    width: 44px;
    height: 24px;
    background: #202028;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-full);
    padding: 2px;
    position: relative;
    cursor: pointer;
    flex-shrink: 0;
    transition: all var(--transition-fast);
    outline: none;
  }

  .thumb {
    display: block;
    width: 18px;
    height: 18px;
    background: #a1a1aa;
    border-radius: 50%;
    transition: transform var(--transition-fast), background var(--transition-fast), box-shadow var(--transition-fast);
    transform: translateX(0);
  }

  /* Checked States with Color Accents */
  .switch-btn.checked.accent-yellow {
    background: rgba(255, 230, 0, 0.25);
    border-color: rgba(255, 230, 0, 0.6);
  }
  .switch-btn.checked.accent-yellow .thumb {
    transform: translateX(20px);
    background: #ffe600;
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.8);
  }

  .switch-btn.checked.accent-violet {
    background: rgba(139, 92, 246, 0.25);
    border-color: rgba(139, 92, 246, 0.6);
  }
  .switch-btn.checked.accent-violet .thumb {
    transform: translateX(20px);
    background: #a78bfa;
    box-shadow: 0 0 10px rgba(139, 92, 246, 0.8);
  }

  .switch-btn.checked.accent-emerald {
    background: rgba(52, 211, 153, 0.25);
    border-color: rgba(52, 211, 153, 0.6);
  }
  .switch-btn.checked.accent-emerald .thumb {
    transform: translateX(20px);
    background: #34d399;
    box-shadow: 0 0 10px rgba(52, 211, 153, 0.8);
  }

  .switch-btn.checked.accent-cyan {
    background: rgba(56, 189, 248, 0.25);
    border-color: rgba(56, 189, 248, 0.6);
  }
  .switch-btn.checked.accent-cyan .thumb {
    transform: translateX(20px);
    background: #38bdf8;
    box-shadow: 0 0 10px rgba(56, 189, 248, 0.8);
  }
</style>
