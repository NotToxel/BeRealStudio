<script lang="ts">
  import { activeError } from '$lib/stores';

  let showDetails = false;

  function handleClose() {
    activeError.set(null);
    showDetails = false;
  }

  function handleCopy() {
    if (!$activeError) return;
    const text = `${$activeError.title}\n${$activeError.message}\n\nDetails:\n${$activeError.details || 'None'}`;
    navigator.clipboard.writeText(text);
  }
  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeError}
  <div
    class="overlay"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={handleClose}
    on:keydown|self={(e) => (e.key === 'Escape' || e.key === 'Enter') && handleClose()}
  >
    <div class="modal card">
      <div class="modal-header">
        <div class="icon-wrap">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
          </svg>
        </div>
        <h3 class="title-md">{$activeError.title}</h3>
      </div>

      <div class="modal-body">
        <p class="error-msg">{$activeError.message}</p>

        {#if $activeError.details}
          <div class="details-section">
            <button
              type="button"
              class="details-toggle"
              on:click={() => (showDetails = !showDetails)}
            >
              <span>{showDetails ? 'Hide' : 'Show'} Technical Details</span>
              <svg
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                style="transform: rotate({showDetails ? 180 : 0}deg); transition: transform 0.15s ease;"
              >
                <polyline points="6 9 12 15 18 9"></polyline>
              </svg>
            </button>

            {#if showDetails}
              <pre class="details-box">{$activeError.details}</pre>
            {/if}
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button type="button" class="btn btn-ghost btn-sm" on:click={handleCopy}>
          Copy Error
        </button>
        <button type="button" class="btn btn-primary btn-sm" on:click={handleClose}>
          Dismiss
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: var(--bg-overlay);
    backdrop-filter: blur(4px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    animation: fadeIn 0.12s ease;
  }

  .modal {
    width: 90%;
    max-width: 500px;
    background: #16161a;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: 0 16px 40px rgba(0, 0, 0, 0.6);
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
  }

  .modal-header {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon-wrap {
    width: 34px;
    height: 34px;
    background: var(--status-error-bg);
    color: var(--status-error);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .error-msg {
    font-size: 13.5px;
    color: var(--text-main);
    line-height: 1.5;
  }

  .details-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .details-toggle {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: var(--text-muted);
    font-size: 12px;
    cursor: pointer;
    padding: 0;
  }

  .details-toggle:hover {
    color: var(--text-secondary);
  }

  .details-box {
    background: #0d0d10;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 10px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    color: var(--text-secondary);
    max-height: 140px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-all;
  }

  .modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    border-top: 1px solid var(--border-subtle);
    padding-top: 16px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: scale(0.98); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
