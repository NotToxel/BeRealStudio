<script lang="ts">
  import { slide } from 'svelte/transition';
  import type { LocationRule } from '$lib/types';
  import { openPath } from '$lib/tauri';
  import Plus from 'lucide-svelte/icons/plus';
  import Trash2 from 'lucide-svelte/icons/trash-2';
  import ArrowUp from 'lucide-svelte/icons/arrow-up';
  import ArrowDown from 'lucide-svelte/icons/arrow-down';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import ChevronDown from 'lucide-svelte/icons/chevron-down';
  import ChevronUp from 'lucide-svelte/icons/chevron-up';
  import Lightbulb from 'lucide-svelte/icons/lightbulb';
  import Globe from 'lucide-svelte/icons/globe';
  import ExternalLink from 'lucide-svelte/icons/external-link';

  export let rules: LocationRule[] = [];

  let isExpanded = false;

  const ISO_COUNTRIES = [
    { code: 'us', name: 'United States' },
    { code: 'gb', name: 'United Kingdom' },
    { code: 'ca', name: 'Canada' },
    { code: 'au', name: 'Australia' },
    { code: 'fr', name: 'France' },
    { code: 'de', name: 'Germany' },
    { code: 'es', name: 'Spain' },
    { code: 'it', name: 'Italy' },
    { code: 'jp', name: 'Japan' },
    { code: 'nl', name: 'Netherlands' },
    { code: 'be', name: 'Belgium' },
    { code: 'ch', name: 'Switzerland' },
    { code: 'at', name: 'Austria' },
    { code: 'se', name: 'Sweden' },
    { code: 'no', name: 'Norway' },
    { code: 'dk', name: 'Denmark' },
    { code: 'fi', name: 'Finland' },
    { code: 'ie', name: 'Ireland' },
    { code: 'pt', name: 'Portugal' },
    { code: 'pl', name: 'Poland' },
    { code: 'gr', name: 'Greece' },
    { code: 'cz', name: 'Czechia' },
    { code: 'nz', name: 'New Zealand' },
    { code: 'br', name: 'Brazil' },
    { code: 'mx', name: 'Mexico' },
    { code: 'ar', name: 'Argentina' },
    { code: 'cl', name: 'Chile' },
    { code: 'kr', name: 'South Korea' },
    { code: 'sg', name: 'Singapore' },
    { code: 'in', name: 'India' },
    { code: 'za', name: 'South Africa' },
    { code: 'ae', name: 'United Arab Emirates' },
    { code: 'tr', name: 'Turkey' },
    { code: 'is', name: 'Iceland' },
    { code: 'hr', name: 'Croatia' },
    { code: 'hu', name: 'Hungary' },
    { code: 'th', name: 'Thailand' },
    { code: 'vn', name: 'Vietnam' },
    { code: 'id', name: 'Indonesia' },
    { code: 'my', name: 'Malaysia' },
    { code: 'ph', name: 'Philippines' },
    { code: 'eg', name: 'Egypt' },
    { code: 'il', name: 'Israel' },
  ];

  async function openIsoGuide() {
    try {
      await openPath('https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2');
    } catch {
      window.open('https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2', '_blank');
    }
  }

  // Ensure there is always a default fallback rule
  $: if (!rules.some((r) => r.condition === 'Default')) {
    rules = [
      ...rules,
      {
        comment: 'Default fallback',
        condition: 'Default',
        format: '{city}, {country}',
      },
    ];
  }

  // Separate match rules and fallback rule
  $: matchRules = rules.filter((r) => r.condition !== 'Default');
  $: defaultRule = rules.find((r) => r.condition === 'Default') || {
    comment: 'Default fallback',
    condition: 'Default' as const,
    format: '{city}, {country}',
  };

  const TOKENS = [
    { label: '{city}', desc: 'City or Town Name' },
    { label: '{suburb}', desc: 'Neighborhood or Suburb' },
    { label: '{state}', desc: 'State / County / Region' },
    { label: '{country}', desc: 'Full Country Name' },
    { label: '{country_code}', desc: 'Two-Letter Country Code' },
  ];

  function addRule() {
    const newRule: LocationRule = {
      comment: 'Custom location rule',
      condition: { country_code: 'us' },
      format: '{city}, {state}',
    };
    const defIdx = rules.findIndex((r) => r.condition === 'Default');
    if (defIdx !== -1) {
      rules = [...rules.slice(0, defIdx), newRule, ...rules.slice(defIdx)];
    } else {
      rules = [...rules, newRule];
    }
  }

  function addExampleSuggestion() {
    const exampleRule: LocationRule = {
      comment: 'United States — city + state',
      condition: { country_code: 'us' },
      format: '{city}, {state}',
    };
    const defIdx = rules.findIndex((r) => r.condition === 'Default');
    if (defIdx !== -1) {
      rules = [...rules.slice(0, defIdx), exampleRule, ...rules.slice(defIdx)];
    } else {
      rules = [...rules, exampleRule];
    }
  }

  function removeRule(globalIndex: number) {
    rules = rules.filter((_, i) => i !== globalIndex);
  }

  function moveRule(fromIdx: number, direction: 'up' | 'down') {
    const toIdx = direction === 'up' ? fromIdx - 1 : fromIdx + 1;
    if (toIdx < 0 || toIdx >= matchRules.length) return;
    const newMatch = [...matchRules];
    const [moved] = newMatch.splice(fromIdx, 1);
    newMatch.splice(toIdx, 0, moved);
    rules = [...newMatch, defaultRule];
  }

  function getConditionField(cond: LocationRule['condition']): string {
    if (cond === 'Default') return 'country_code';
    return Object.keys(cond)[0] || 'country_code';
  }

  function getConditionValue(cond: LocationRule['condition']): string {
    if (cond === 'Default') return '';
    return Object.values(cond)[0] || '';
  }

  function updateCondition(idx: number, field: string, value: string) {
    const target = matchRules[idx];
    if (!target) return;
    target.condition = { [field]: value.toLowerCase().trim() };
    rules = [...matchRules, defaultRule];
  }

  function insertTokenAt(ruleIdx: number, token: string) {
    if (ruleIdx === -1) {
      defaultRule.format += (defaultRule.format.length > 0 && !defaultRule.format.endsWith(' ') ? ' ' : '') + token;
      rules = [...matchRules, defaultRule];
    } else {
      const r = matchRules[ruleIdx];
      if (!r) return;
      r.format += (r.format.length > 0 && !r.format.endsWith(' ') ? ' ' : '') + token;
      rules = [...matchRules, defaultRule];
    }
  }

  // Live simulation helper
  const sampleData: Record<string, string> = {
    city: 'New York',
    suburb: 'Manhattan',
    state: 'New York',
    country: 'United States',
    country_code: 'us',
  };

  function simulateFormatting(rule: LocationRule): string {
    let res = rule.format;
    for (const [k, v] of Object.entries(sampleData)) {
      res = res.replaceAll(`{${k}}`, v);
    }
    return res;
  }
</script>

<div class="rule-editor card">
  <!-- Collapsible Header -->
  <button
    type="button"
    class="accordion-header"
    on:click={() => (isExpanded = !isExpanded)}
    aria-expanded={isExpanded}
  >
    <div class="header-left">
      <MapPin size={15} class="text-sky-400" />
      <span class="title-sm font-semibold">Advanced Location Formatting Rules</span>
      {#if matchRules.length > 0}
        <span class="badge badge-sky font-mono">{matchRules.length} custom rule{matchRules.length > 1 ? 's' : ''}</span>
      {:else}
        <span class="badge badge-neutral font-mono">Default fallback only</span>
      {/if}
    </div>

    <div class="header-right">
      <span class="toggle-text">{isExpanded ? 'Collapse' : 'Customize rules'}</span>
      {#if isExpanded}
        <ChevronUp size={14} />
      {:else}
        <ChevronDown size={14} />
      {/if}
    </div>
  </button>

  {#if isExpanded}
    <div class="accordion-body" transition:slide={{ duration: 240 }}>
      <div class="header-actions">
        <span class="priority-note">
          Rules evaluate <strong>top-to-bottom</strong>. The first matching rule applies.
        </span>

        <div class="header-tools">
          <button type="button" class="btn-link-help" on:click={openIsoGuide} title="Open ISO 3166-1 Country Codes List on Wikipedia">
            <Globe size={12} class="text-sky-400" />
            <span>ISO Country Codes List</span>
            <ExternalLink size={10} />
          </button>
          <button type="button" class="btn btn-secondary btn-sm" on:click={addRule}>
            <Plus size={13} />
            <span>Add Custom Rule</span>
          </button>
        </div>
      </div>

      <!-- Datalist of Standard ISO Country Codes -->
      <datalist id="iso-country-codes">
        {#each ISO_COUNTRIES as c}
          <option value={c.code}>{c.code.toUpperCase()} — {c.name}</option>
        {/each}
      </datalist>

      <!-- Single Clear Suggestion if no custom rules exist -->
      {#if matchRules.length === 0}
        <div class="suggestion-banner">
          <div class="suggestion-icon">
            <Lightbulb size={16} class="text-amber-400" />
          </div>
          <div class="suggestion-content">
            <span class="suggestion-title">Format Example Suggestion:</span>
            <span class="suggestion-desc">
              By default, locations show as <code>{'{city}, {country}'}</code>. You can add a rule for specific countries like the US to show <code>{'{city}, {state}'}</code>.
            </span>
          </div>
          <button type="button" class="btn btn-secondary btn-sm suggestion-btn" on:click={addExampleSuggestion}>
            <Plus size={12} />
            <span>Add US Example Rule</span>
          </button>
        </div>
      {/if}

      <!-- Match Rules List -->
      <div class="rules-list">
        {#each matchRules as rule, idx}
          {@const globalIdx = rules.indexOf(rule)}
          {@const field = getConditionField(rule.condition)}
          {@const val = getConditionValue(rule.condition)}
          <div class="rule-card match-card">
            <!-- Rule Header Row -->
            <div class="rule-head">
              <div class="priority-indicator">
                <span class="priority-num">{idx + 1}</span>
                <span class="rule-type-tag">Custom Match Rule</span>
              </div>

              <div class="rule-actions">
                <button
                  type="button"
                  class="btn-icon"
                  disabled={idx === 0}
                  on:click={() => moveRule(idx, 'up')}
                  title="Increase Priority (Move Up)"
                >
                  <ArrowUp size={13} />
                </button>
                <button
                  type="button"
                  class="btn-icon"
                  disabled={idx === matchRules.length - 1}
                  on:click={() => moveRule(idx, 'down')}
                  title="Decrease Priority (Move Down)"
                >
                  <ArrowDown size={13} />
                </button>
                <button
                  type="button"
                  class="btn-icon btn-delete"
                  on:click={() => removeRule(globalIdx)}
                  title="Delete Rule"
                >
                  <Trash2 size={13} />
                </button>
              </div>
            </div>

            <!-- Condition Controls Row -->
            <div class="condition-row">
              <span class="cond-prefix">If</span>
              <select
                class="input-select select-field"
                value={field}
                on:change={(e) => updateCondition(idx, e.currentTarget.value, val)}
              >
                <option value="country_code">country_code</option>
                <option value="country">country</option>
                <option value="state">state</option>
                <option value="city">city</option>
              </select>

              <span class="cond-op">equals</span>

              <input
                type="text"
                class="input-text input-val font-mono"
                list={field === 'country_code' ? 'iso-country-codes' : undefined}
                value={val}
                placeholder={field === 'country_code'
                  ? 'e.g. us, gb, ca'
                  : field === 'city'
                  ? 'e.g. New York, London'
                  : field === 'state'
                  ? 'e.g. California, Ontario'
                  : 'e.g. United States'}
                on:input={(e) => updateCondition(idx, field, e.currentTarget.value)}
              />

              <input
                type="text"
                class="input-text input-comment"
                bind:value={rule.comment}
                placeholder="Description (optional)"
              />
            </div>

            <!-- Format Pattern Row with Insertable Token Pills -->
            <div class="format-row">
              <div class="format-input-group">
                <label for="format-{idx}" class="format-label">Format:</label>
                <input
                  id="format-{idx}"
                  type="text"
                  class="input-text input-format font-mono"
                  bind:value={rule.format}
                  placeholder="{'{city}, {state}'}"
                />
              </div>

              <!-- Token Pills -->
              <div class="tokens-group">
                <span class="tokens-label">Insert:</span>
                {#each TOKENS as tok}
                  <button
                    type="button"
                    class="token-pill"
                    title="{tok.desc}"
                    on:click={() => insertTokenAt(idx, tok.label)}
                  >
                    {tok.label}
                  </button>
                {/each}
              </div>
            </div>

            <!-- Live Preview -->
            <div class="preview-line">
              <Sparkles size={11} class="text-sky-400" />
              <span class="preview-label">Sample result ({sampleData.city}, {sampleData.country_code.toUpperCase()}):</span>
              <span class="preview-value font-mono">{simulateFormatting(rule)}</span>
            </div>
          </div>
        {/each}

        <!-- Default Fallback Rule Card -->
        <div class="rule-card fallback-card">
          <div class="rule-head">
            <div class="priority-indicator">
              <span class="priority-num fallback-badge">&infin;</span>
              <span class="rule-type-tag fallback-tag">Default Fallback Rule</span>
            </div>
            <span class="fallback-hint">Applies to all other memories</span>
          </div>

          <div class="format-row">
            <div class="format-input-group">
              <label for="format-default" class="format-label">Format:</label>
              <input
                id="format-default"
                type="text"
                class="input-text input-format font-mono"
                bind:value={defaultRule.format}
                placeholder="{'{city}, {country}'}"
              />
            </div>

            <div class="tokens-group">
              <span class="tokens-label">Insert:</span>
              {#each TOKENS as tok}
                <button
                  type="button"
                  class="token-pill"
                  title="{tok.desc}"
                  on:click={() => insertTokenAt(-1, tok.label)}
                >
                  {tok.label}
                </button>
              {/each}
            </div>
          </div>

          <div class="preview-line">
            <Sparkles size={11} class="text-sky-400" />
            <span class="preview-label">Sample result (fallback):</span>
            <span class="preview-value font-mono">{simulateFormatting(defaultRule)}</span>
          </div>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .rule-editor {
    display: flex;
    flex-direction: column;
    background: #0f0f14;
    padding: 0;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-subtle);
    overflow: hidden;
  }

  .accordion-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 12px 16px;
    background: transparent;
    border: none;
    cursor: pointer;
    width: 100%;
    color: inherit;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .accordion-header:hover {
    background: #14141c;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 11.5px;
    color: var(--text-muted);
  }

  .accordion-body {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 14px 16px 16px 16px;
    border-top: 1px solid var(--border-subtle);
    background: #0b0b10;
  }

  .header-actions {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 10px;
  }

  .header-tools {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .btn-link-help {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    background: rgba(56, 189, 248, 0.08);
    border: 1px solid rgba(56, 189, 248, 0.25);
    color: #38bdf8;
    padding: 4px 9px;
    border-radius: var(--radius-sm);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .btn-link-help:hover {
    background: rgba(56, 189, 248, 0.18);
    border-color: rgba(56, 189, 248, 0.45);
    color: #7dd3fc;
  }

  .priority-note {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .priority-note strong {
    color: #38bdf8;
  }

  /* Suggestion Banner */
  .suggestion-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    background: rgba(255, 230, 0, 0.04);
    border: 1px dashed rgba(255, 230, 0, 0.28);
    border-radius: var(--radius-md);
    padding: 10px 14px;
    flex-wrap: wrap;
  }

  .suggestion-icon {
    flex-shrink: 0;
  }

  .suggestion-content {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 200px;
  }

  .suggestion-title {
    font-size: 11.5px;
    font-weight: 600;
    color: #ffe600;
  }

  .suggestion-desc {
    font-size: 11px;
    color: var(--text-secondary);
    line-height: 1.35;
  }

  .suggestion-desc code {
    background: #181822;
    padding: 1px 4px;
    border-radius: 3px;
    font-family: var(--font-mono);
    color: #38bdf8;
  }

  .suggestion-btn {
    white-space: nowrap;
    border-color: rgba(255, 230, 0, 0.3);
    color: #ffe600;
  }

  .suggestion-btn:hover {
    background: rgba(255, 230, 0, 0.12);
  }

  /* Rules List */
  .rules-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .rule-card {
    background: #0e0e13;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .match-card {
    border-left: 3px solid rgba(56, 189, 248, 0.5);
  }

  .fallback-card {
    border-left: 3px solid rgba(255, 230, 0, 0.5);
    background: #0a0a0e;
  }

  .rule-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .priority-indicator {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .priority-num {
    width: 20px;
    height: 20px;
    background: #181822;
    border: 1px solid var(--border-subtle);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 11px;
    font-weight: 700;
    color: var(--text-main);
  }

  .fallback-badge {
    color: #ffe600;
    border-color: rgba(255, 230, 0, 0.3);
  }

  .rule-type-tag {
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
  }

  .fallback-tag {
    color: #ffe600;
  }

  .fallback-hint {
    font-size: 11px;
    color: var(--text-muted);
    font-style: italic;
  }

  .rule-actions {
    display: flex;
    align-items: center;
    gap: 2px;
  }

  .btn-icon {
    width: 26px;
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }

  .btn-icon:hover:not(:disabled) {
    background: #1c1c24;
    color: var(--text-main);
  }

  .btn-icon:disabled {
    opacity: 0.25;
    cursor: not-allowed;
  }

  .btn-delete:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
  }

  /* Condition row */
  .condition-row {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-wrap: wrap;
  }

  .cond-prefix,
  .cond-op {
    font-size: 11.5px;
    color: var(--text-muted);
    font-weight: 500;
  }

  .select-field {
    padding: 4px 8px;
    font-size: 12px;
    background: #14141a;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-main);
  }

  .input-val {
    width: 90px;
    padding: 4px 8px;
    font-size: 12px;
    background: #14141a;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-main);
  }

  .input-comment {
    flex: 1;
    min-width: 140px;
    padding: 4px 8px;
    font-size: 12px;
    background: transparent;
    border: 1px dashed var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-muted);
  }

  .input-comment:focus {
    color: var(--text-main);
    border-style: solid;
  }

  /* Format row */
  .format-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    flex-wrap: wrap;
  }

  .format-input-group {
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 1;
    min-width: 200px;
  }

  .format-label {
    font-size: 11.5px;
    font-weight: 600;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .input-format {
    flex: 1;
    padding: 5px 10px;
    font-size: 12.5px;
    background: #14141a;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-main);
  }

  .tokens-group {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
  }

  .tokens-label {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .token-pill {
    padding: 2px 6px;
    font-size: 10.5px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    background: #181822;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: all var(--transition-fast);
  }

  .token-pill:hover {
    color: #38bdf8;
    background: rgba(56, 189, 248, 0.12);
    border-color: rgba(56, 189, 248, 0.3);
  }

  /* Preview line */
  .preview-line {
    display: flex;
    align-items: center;
    gap: 6px;
    padding-top: 4px;
    border-top: 1px solid rgba(255, 255, 255, 0.04);
  }

  .preview-label {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .preview-value {
    font-size: 11.5px;
    color: #38bdf8;
    font-weight: 600;
  }

  .badge-neutral {
    background: rgba(255, 255, 255, 0.06);
    color: var(--text-muted);
  }
</style>
