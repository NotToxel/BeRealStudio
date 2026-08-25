<script lang="ts">
  import type { LocationRule } from '$lib/types';

  export let rules: LocationRule[] = [];

  function addRule() {
    rules = [
      ...rules,
      {
        comment: 'New rule',
        condition: { country_code: 'us' },
        format: '{city}, {state}',
      },
    ];
  }

  function removeRule(index: number) {
    rules = rules.filter((_, i) => i !== index);
  }

  function formatCondition(condition: LocationRule['condition']): string {
    if (condition === 'Default') return 'Default (fallback)';
    return Object.entries(condition)
      .map(([k, v]) => `${k} = "${v}"`)
      .join(', ');
  }
</script>

<div class="rule-editor card">
  <div class="header">
    <div class="title-group">
      <span class="title-sm">Location Formatting Rules</span>
      <span class="badge">{rules.length} active</span>
    </div>
    <button type="button" class="btn btn-secondary btn-sm" on:click={addRule}>
      <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <line x1="12" y1="5" x2="12" y2="19"></line>
        <line x1="5" y1="12" x2="19" y2="12"></line>
      </svg>
      Add Rule
    </button>
  </div>

  <p class="description text-secondary">
    Rules are evaluated in order. Placeholders: <code>{'{city}'}</code>, <code>{'{suburb}'}</code>, <code>{'{state}'}</code>, <code>{'{country}'}</code>.
  </p>

  <div class="rules-list">
    {#each rules as rule, idx}
      <div class="rule-item">
        <div class="rule-top">
          <span class="rule-badge">{idx + 1}</span>
          <span class="rule-comment">{rule.comment || 'Rule'}</span>
          <span class="rule-cond">{formatCondition(rule.condition)}</span>
          {#if rule.condition !== 'Default'}
            <button
              type="button"
              class="btn-icon-delete"
              on:click={() => removeRule(idx)}
              title="Delete rule"
            >
              <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <polyline points="3 6 5 6 21 6"></polyline>
                <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path>
              </svg>
            </button>
          {/if}
        </div>

        <div class="rule-format-row">
          <label for="rule-format-{idx}">Format:</label>
          <input
            id="rule-format-{idx}"
            type="text"
            class="input-text input-format"
            bind:value={rule.format}
            placeholder="{'{city}, {country}'}"
          />
        </div>
      </div>
    {/each}
  </div>
</div>

<style>
  .rule-editor {
    display: flex;
    flex-direction: column;
    gap: 12px;
    background: var(--bg-card-subtle);
    padding: 16px;
    border-radius: var(--radius-md);
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .title-group {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .description {
    font-size: 12px;
    line-height: 1.4;
  }

  .description code {
    background: var(--bg-card);
    padding: 1px 4px;
    border-radius: 4px;
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .rules-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .rule-item {
    background: var(--bg-card);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    padding: 10px 12px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .rule-top {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rule-badge {
    width: 18px;
    height: 18px;
    background: var(--border-medium);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 10px;
    font-weight: 700;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .rule-comment {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-main);
  }

  .rule-cond {
    font-size: 11.5px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    margin-left: auto;
  }

  .btn-icon-delete {
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    padding: 2px;
    display: flex;
    align-items: center;
  }

  .btn-icon-delete:hover {
    color: var(--status-error);
  }

  .rule-format-row {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .rule-format-row label {
    font-size: 11.5px;
    color: var(--text-secondary);
    font-weight: 500;
  }

  .input-format {
    padding: 4px 8px;
    font-size: 12px;
    font-family: var(--font-mono);
  }
</style>
