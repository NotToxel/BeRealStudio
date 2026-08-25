<script lang="ts">
  import type { ComponentType } from 'svelte';
  import Info from 'lucide-svelte/icons/info';

  export let label: string;
  export let description: string = '';
  export let tooltip: string = '';
  export let icon: any = null;
  export let checked: boolean = false;
  export let disabled: boolean = false;
  export let accentColor: 'yellow' | 'violet' | 'emerald' | 'cyan' = 'yellow';
  export let onChange: ((val: boolean) => void) | undefined = undefined;

  let showTooltip = false;

  function toggle() {
    if (disabled) return;
    checked = !checked;
    if (onChange) onChange(checked);
  }
</script>

<div class="toggle-container" class:disabled>
  <div
    class="toggle-left"
    role="button"
    tabindex="0"
    on:click={toggle}
    on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && toggle()}
  >
    {#if icon}
      <div class="toggle-icon-wrap accent-{accentColor}">
        <svelte:component this={icon} size={16} />
      </div>
    {/if}
    <div class="text-group">
      <div class="label-row">
        <span class="label-text">{label}</span>
        {#if tooltip}
          <div class="info-popover-wrapper">
            <button
              type="button"
              class="info-btn"
              class:active={showTooltip}
              on:click|stopPropagation|preventDefault={() => (showTooltip = !showTooltip)}
              on:mouseenter={() => (showTooltip = true)}
              on:mouseleave={() => (showTooltip = false)}
              on:focus={() => (showTooltip = true)}
              on:blur={() => (showTooltip = false)}
              aria-label="Information about {label}"
            >
              <Info size={13} />
            </button>

            {#if showTooltip}
              <div class="rich-tooltip" role="tooltip">
                <div class="tooltip-head">
                  <span class="tooltip-title">{label}</span>
                </div>
                <p class="tooltip-desc">{tooltip}</p>
                <div class="tooltip-arrow"></div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
      {#if description}
        <span class="desc-text">{description}</span>
      {/if}
    </div>
  </div>

  <button
    type="button"
    role="switch"
    aria-label={label}
    aria-checked={checked}
    class="switch-btn accent-{accentColor}"
    class:checked
    on:click|stopPropagation={toggle}
    {disabled}
  >
    <span class="thumb"></span>
  </button>
</div>

<style>
  .toggle-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
    user-select: none;
    padding: 6px 0;
    position: relative;
  }

  .toggle-container.disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .toggle-left {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    cursor: pointer;
    flex: 1;
  }

  .toggle-icon-wrap {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
    background: #181822;
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
    transition: all var(--transition-fast);
  }

  .toggle-icon-wrap.accent-emerald {
    color: #34d399;
    border-color: rgba(52, 211, 153, 0.25);
    background: rgba(52, 211, 153, 0.08);
  }

  .toggle-icon-wrap.accent-cyan {
    color: #38bdf8;
    border-color: rgba(56, 189, 248, 0.25);
    background: rgba(56, 189, 248, 0.08);
  }

  .toggle-icon-wrap.accent-yellow {
    color: #ffe600;
    border-color: rgba(255, 230, 0, 0.25);
    background: rgba(255, 230, 0, 0.08);
  }

  .toggle-icon-wrap.accent-violet {
    color: #c084fc;
    border-color: rgba(192, 132, 252, 0.25);
    background: rgba(192, 132, 252, 0.08);
  }

  .text-group {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .label-row {
    display: flex;
    align-items: center;
    gap: 6px;
    position: relative;
  }

  .label-text {
    font-size: 13.5px;
    font-weight: 600;
    color: var(--text-main);
  }

  .desc-text {
    font-size: 11.5px;
    color: var(--text-muted);
    line-height: 1.35;
  }

  /* Info Popover & Rich Tooltip */
  .info-popover-wrapper {
    position: relative;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .info-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    cursor: pointer;
    color: var(--text-muted);
    padding: 2px 4px;
    border-radius: 4px;
    transition: all var(--transition-fast);
  }

  .info-btn:hover,
  .info-btn.active {
    color: #ffffff;
    background: rgba(255, 255, 255, 0.12);
  }

  .rich-tooltip {
    position: absolute;
    bottom: calc(100% + 8px);
    left: 50%;
    transform: translateX(-50%);
    width: 260px;
    background: #14141c;
    border: 1px solid var(--border-medium);
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.7);
    border-radius: var(--radius-md);
    padding: 10px 12px;
    z-index: 300;
    pointer-events: auto;
    animation: tooltipFade 0.15s ease-out;
  }

  .tooltip-head {
    display: flex;
    align-items: center;
    margin-bottom: 4px;
  }

  .tooltip-title {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
  }

  .tooltip-desc {
    font-size: 11.5px;
    color: var(--text-secondary);
    line-height: 1.4;
    margin: 0;
  }

  .tooltip-arrow {
    position: absolute;
    top: 100%;
    left: 50%;
    transform: translateX(-50%);
    border-width: 5px;
    border-style: solid;
    border-color: #14141c transparent transparent transparent;
  }

  @keyframes tooltipFade {
    from {
      opacity: 0;
      transform: translate(-50%, 4px);
    }
    to {
      opacity: 1;
      transform: translate(-50%, 0);
    }
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
    transition: background 0.24s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.24s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.24s cubic-bezier(0.16, 1, 0.3, 1);
    outline: none;
  }

  .switch-btn:hover:not(:disabled) {
    border-color: var(--border-focus);
  }

  .thumb {
    display: block;
    width: 18px;
    height: 18px;
    background: #a1a1aa;
    border-radius: 50%;
    transition: transform 0.28s cubic-bezier(0.34, 1.56, 0.64, 1), background 0.22s ease, box-shadow 0.22s ease;
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
