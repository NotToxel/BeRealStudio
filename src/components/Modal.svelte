<script lang="ts">
  import { fade, scale } from 'svelte/transition';

  export let open: boolean = false;
  export let title: string = '';
  export let maxWidth: string = '480px';
  export let showCloseButton: boolean = true;

  function handleBackdropClick(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      open = false;
    }
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      open = false;
    }
  }
</script>

<svelte:window on:keydown={handleKeyDown} />

{#if open}
  <div
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    transition:fade={{ duration: 160 }}
    on:click={handleBackdropClick}
    on:keydown={(e) => e.key === 'Escape' && (open = false)}
  >
    <div
      class="modal-card"
      style="max-width: {maxWidth};"
      transition:scale={{ duration: 200, start: 0.94 }}
    >
      {#if title || showCloseButton}
        <div class="modal-header">
          <div class="modal-title-slot">
            <slot name="title">
              <h3 class="title-sm font-bold text-white">{title}</h3>
            </slot>
          </div>

          {#if showCloseButton}
            <button
              type="button"
              class="btn-modal-close"
              on:click={() => (open = false)}
              aria-label="Close dialog"
            >
              <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </button>
          {/if}
        </div>
      {/if}

      <div class="modal-body">
        <slot />
      </div>

      {#if $$slots.footer}
        <div class="modal-footer">
          <slot name="footer" />
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 999;
    padding: 20px;
  }

  .modal-card {
    background: #121217;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    padding: 22px;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 14px;
    box-shadow: 0 20px 48px rgba(0, 0, 0, 0.75);
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
  }

  .modal-title-slot {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }

  .btn-modal-close {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    padding: 6px;
    border-radius: var(--radius-sm);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all var(--transition-fast);
  }

  .btn-modal-close:hover {
    color: var(--text-main);
    background: var(--border-subtle);
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    font-size: 13px;
    color: var(--text-secondary);
    line-height: 1.5;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 4px;
    padding-top: 10px;
    border-top: 1px solid var(--border-subtle);
  }
</style>
