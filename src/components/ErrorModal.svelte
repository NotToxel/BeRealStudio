<script lang="ts">
  import { activeError } from '$lib/stores';
  import { diagnoseError, type ErrorDiagnosis } from '$lib/errorHelper';
  import AlertTriangle from 'lucide-svelte/icons/triangle-alert';
  import Lightbulb from 'lucide-svelte/icons/lightbulb';
  import Terminal from 'lucide-svelte/icons/terminal';
  import Copy from 'lucide-svelte/icons/copy';
  import Check from 'lucide-svelte/icons/check';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';

  let showDetails = false;
  let copied = false;

  $: diagnosis = $activeError
    ? diagnoseError($activeError.details ? `${$activeError.message}\n${$activeError.details}` : $activeError.message, $activeError.title)
    : null;

  function handleClose() {
    activeError.set(null);
    showDetails = false;
    copied = false;
  }

  function handleCopyReport() {
    if (!$activeError || !diagnosis) return;
    const report = [
      `=== BeReal Studio Error Diagnostic Report ===`,
      `Time: ${new Date().toISOString()}`,
      `Title: ${$activeError.title}`,
      `Category: ${diagnosis.categoryLabel}`,
      `Summary: ${diagnosis.explanation}`,
      ``,
      `--- Suggested Solutions ---`,
      diagnosis.suggestion,
      ``,
      `--- Raw Technical Error ---`,
      $activeError.details || $activeError.message,
    ].join('\n');

    navigator.clipboard.writeText(report);
    copied = true;
    setTimeout(() => (copied = false), 2000);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      handleClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeError && diagnosis}
  <div
    class="overlay"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click|self={handleClose}
    on:keydown|self={(e) => (e.key === 'Escape' || e.key === 'Enter') && handleClose()}
  >
    <div class="modal card">
      <!-- Modal Header -->
      <div class="modal-header">
        <div class="header-left">
          <div class="icon-wrap">
            <AlertTriangle size={20} class="text-rose-400" />
          </div>
          <div>
            <h3 class="title-md font-bold text-white">{$activeError.title}</h3>
            <span class="badge badge-rose font-mono text-xs mt-0.5">
              {diagnosis.categoryLabel}
            </span>
          </div>
        </div>
        <button type="button" class="btn-close-x" on:click={handleClose} aria-label="Close dialog">
          <svg width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
          </svg>
        </button>
      </div>

      <!-- Modal Body -->
      <div class="modal-body">
        <!-- Explanation -->
        <p class="error-msg text-secondary text-sm">
          {diagnosis.explanation}
        </p>

        <!-- Suggested Resolution Steps -->
        {#if diagnosis.suggestion}
          <div class="suggestion-box">
            <div class="suggestion-head">
              <Lightbulb size={16} class="text-amber-400" />
              <strong class="text-white text-xs font-semibold">How to Resolve This:</strong>
            </div>
            <div class="suggestion-content font-mono text-xs text-secondary">
              {diagnosis.suggestion}
            </div>
          </div>
        {/if}

        <!-- Collapsible Technical Details -->
        <div class="details-section">
          <button
            type="button"
            class="details-toggle"
            on:click={() => (showDetails = !showDetails)}
          >
            <Terminal size={13} />
            <span>{showDetails ? 'Hide' : 'Show'} Raw Diagnostic Log</span>
            {#if showDetails}
              <ChevronUp size={12} />
            {:else}
              <ChevronDown size={12} />
            {/if}
          </button>

          {#if showDetails}
            <pre class="details-box font-mono">{diagnosis.rawDetails}</pre>
          {/if}
        </div>
      </div>

      <!-- Modal Footer -->
      <div class="modal-footer">
        <button type="button" class="btn btn-secondary btn-sm" on:click={handleCopyReport}>
          {#if copied}
            <Check size={13} class="text-emerald-400" />
            <span>Copied to Clipboard!</span>
          {:else}
            <Copy size={13} />
            <span>Copy Diagnostic Report</span>
          {/if}
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
    background: rgba(0, 0, 0, 0.75);
    backdrop-filter: blur(8px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 9999;
    animation: fadeIn 0.12s ease;
    padding: 20px;
  }

  .modal {
    width: 100%;
    max-width: 540px;
    background: #141419;
    border: 1px solid rgba(244, 63, 94, 0.35);
    border-radius: var(--radius-lg);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.7), 0 0 30px rgba(244, 63, 94, 0.15);
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 24px;
  }

  .modal-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon-wrap {
    width: 38px;
    height: 38px;
    background: rgba(244, 63, 94, 0.15);
    border: 1px solid rgba(244, 63, 94, 0.3);
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .btn-close-x {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 4px;
    border-radius: var(--radius-sm);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .btn-close-x:hover {
    color: var(--text-main);
    background: rgba(255, 255, 255, 0.08);
  }

  .modal-body {
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  .error-msg {
    line-height: 1.5;
  }

  .suggestion-box {
    background: rgba(245, 158, 11, 0.06);
    border: 1px solid rgba(245, 158, 11, 0.25);
    border-radius: var(--radius-md);
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .suggestion-head {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .suggestion-content {
    white-space: pre-line;
    line-height: 1.6;
    color: #e2e8f0;
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
    background: #0a0a0d;
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: var(--radius-md);
    padding: 12px;
    font-size: 11.5px;
    color: #94a3b8;
    max-height: 160px;
    overflow-y: auto;
    white-space: pre-wrap;
    word-break: break-all;
    line-height: 1.5;
  }

  .modal-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 8px;
    border-top: 1px solid var(--border-subtle);
    padding-top: 16px;
  }

  @keyframes fadeIn {
    from { opacity: 0; transform: scale(0.98); }
    to { opacity: 1; transform: scale(1); }
  }
</style>
