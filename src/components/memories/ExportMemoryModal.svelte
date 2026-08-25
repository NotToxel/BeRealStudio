<script lang="ts">
  import {
    exportModalState,
    closeExportModal,
    exportPreferences,
    type ExportPreferences,
  } from '$lib/memoriesStore';
  import { exportSingleMemory } from '$lib/tauri';
  import { save } from '@tauri-apps/plugin-dialog';
  import X from 'lucide-svelte/icons/circle-x';
  import Download from 'lucide-svelte/icons/download';
  import Layers from 'lucide-svelte/icons/layers';
  import Columns2 from 'lucide-svelte/icons/columns-2';
  import Camera from 'lucide-svelte/icons/camera';
  import User from 'lucide-svelte/icons/circle-user';
  import Film from 'lucide-svelte/icons/film';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import Check from 'lucide-svelte/icons/check';
  import Loader2 from 'lucide-svelte/icons/loader-circle';

  let isExporting = false;
  let exportSuccess = false;

  $: state = $exportModalState;
  $: memory = state?.memory;
  $: prefs = $exportPreferences;

  let selectedType: ExportPreferences['exportType'] = $exportPreferences?.exportType || 'combined_pip';
  let selectedFormat: ExportPreferences['format'] = $exportPreferences?.format || 'Jpeg';
  let embedExif = $exportPreferences?.embedExif ?? true;
  let embedGps = $exportPreferences?.embedGps ?? true;
  let makeDefault = $exportPreferences?.isDefaultSet ?? false;

  // Sync state when modal is opened
  $: if (state?.isOpen && $exportPreferences) {
    selectedType = $exportPreferences.exportType || 'combined_pip';
    selectedFormat = $exportPreferences.format || 'Jpeg';
    embedExif = $exportPreferences.embedExif ?? true;
    embedGps = $exportPreferences.embedGps ?? true;
    makeDefault = $exportPreferences.isDefaultSet ?? false;
  }

  $: if (memory && !memory.secondaryPath && (selectedType === 'combined_pip' || selectedType === 'combined_sidebyside' || selectedType === 'secondary_only')) {
    selectedType = 'primary_only';
  }

  $: if (memory && !memory.btsPath && (selectedType === 'bts_only' || selectedType === 'motion_photo')) {
    selectedType = 'combined_pip';
  }

  async function handleConfirmExport() {
    if (!memory || !memory.primaryPath) return;

    try {
      isExporting = true;
      exportSuccess = false;

      // Save preference if set as default
      exportPreferences.set({
        exportType: selectedType,
        format: selectedFormat,
        quality: 92,
        embedExif,
        embedGps,
        isDefaultSet: makeDefault,
      });

      const datePrefix = memory.takenAt ? memory.takenAt.slice(0, 10) : 'bereal';
      const isVideoExport = selectedType === 'bts_only';
      const ext = isVideoExport ? 'mp4' : selectedFormat.toLowerCase() === 'png' ? 'png' : selectedFormat.toLowerCase() === 'webp' ? 'webp' : 'jpg';
      const defaultFilename = `${datePrefix}_${selectedType}.${ext}`;

      const filters = isVideoExport
        ? [{ name: 'MP4 Video', extensions: ['mp4'] }]
        : [{ name: 'Image', extensions: [ext] }];

      const savePath = await save({
        defaultPath: defaultFilename,
        filters,
      });

      if (!savePath) {
        isExporting = false;
        return;
      }

      await exportSingleMemory({
        memoryIndex: memory.index,
        primaryPath: memory.primaryPath,
        secondaryPath: memory.secondaryPath,
        btsPath: memory.btsPath,
        outputPath: savePath,
        exportType: selectedType,
        format: selectedFormat,
        quality: 92,
        embedExif,
        takenAt: memory.takenAt,
        latitude: embedGps && memory.location ? memory.location.latitude : undefined,
        longitude: embedGps && memory.location ? memory.location.longitude : undefined,
        caption: memory.caption,
      });

      exportSuccess = true;
      setTimeout(() => {
        closeExportModal();
        exportSuccess = false;
      }, 600);
    } catch (err) {
      console.error('Failed to export memory:', err);
    } finally {
      isExporting = false;
    }
  }
</script>

<svelte:window on:keydown={(e) => e.key === 'Escape' && closeExportModal()} />

{#if state.isOpen && memory}
  <div
    class="modal-backdrop"
    on:click={(e) => e.target === e.currentTarget && closeExportModal()}
    role="presentation"
  >
    <div class="modal-card">
      <!-- Header -->
      <div class="modal-header">
        <div class="header-title-group">
          <div class="icon-wrap">
            <Download size={18} class="text-sky-400" />
          </div>
          <div>
            <h3 class="modal-title">Export BeReal Memory</h3>
            <p class="modal-subtitle">{memory.dateFormatted} • {memory.timeFormatted}</p>
          </div>
        </div>

        <button
          type="button"
          class="close-btn"
          on:click={closeExportModal}
          title="Close export dialog"
        >
          <X size={16} />
        </button>
      </div>

      <!-- Body: Export Types Matrix -->
      <div class="modal-body">
        <div class="section-label">SELECT EXPORT FORMAT</div>

        <div class="format-options-grid">
          <!-- Option: Combined PIP -->
          <label
            class="format-option-card"
            class:selected={selectedType === 'combined_pip'}
            class:disabled={!memory.secondaryPath}
          >
            <input
              type="radio"
              name="exportType"
              value="combined_pip"
              bind:group={selectedType}
              disabled={!memory.secondaryPath}
            />
            <div class="option-icon-box">
              <Layers size={18} />
            </div>
            <div class="option-text">
              <span class="option-name">Picture-in-Picture</span>
              <span class="option-desc">Main photo + selfie camera inset</span>
            </div>
          </label>

          <!-- Option: Side-by-Side -->
          <label
            class="format-option-card"
            class:selected={selectedType === 'combined_sidebyside'}
            class:disabled={!memory.secondaryPath}
          >
            <input
              type="radio"
              name="exportType"
              value="combined_sidebyside"
              bind:group={selectedType}
              disabled={!memory.secondaryPath}
            />
            <div class="option-icon-box">
              <Columns2 size={18} />
            </div>
            <div class="option-text">
              <span class="option-name">Side-by-Side</span>
              <span class="option-desc">Both cameras side by side</span>
            </div>
          </label>

          <!-- Option: Main Camera -->
          <label
            class="format-option-card"
            class:selected={selectedType === 'primary_only'}
          >
            <input
              type="radio"
              name="exportType"
              value="primary_only"
              bind:group={selectedType}
            />
            <div class="option-icon-box">
              <Camera size={18} />
            </div>
            <div class="option-text">
              <span class="option-name">Main Camera Only</span>
              <span class="option-desc">High-res primary photo</span>
            </div>
          </label>

          <!-- Option: Selfie Camera -->
          <label
            class="format-option-card"
            class:selected={selectedType === 'secondary_only'}
            class:disabled={!memory.secondaryPath}
          >
            <input
              type="radio"
              name="exportType"
              value="secondary_only"
              bind:group={selectedType}
              disabled={!memory.secondaryPath}
            />
            <div class="option-icon-box">
              <User size={18} />
            </div>
            <div class="option-text">
              <span class="option-name">Selfie Camera Only</span>
              <span class="option-desc">Front-facing camera photo</span>
            </div>
          </label>

          <!-- Option: BTS Micro-Video (if available) -->
          {#if memory.btsPath}
            <label
              class="format-option-card"
              class:selected={selectedType === 'bts_only'}
            >
              <input
                type="radio"
                name="exportType"
                value="bts_only"
                bind:group={selectedType}
              />
              <div class="option-icon-box text-amber-400">
                <Film size={18} />
              </div>
              <div class="option-text">
                <span class="option-name">BTS Micro-Video</span>
                <span class="option-desc">Raw video clip (.mp4)</span>
              </div>
            </label>

            <!-- Option: Motion Photo (if available) -->
            <label
              class="format-option-card"
              class:selected={selectedType === 'motion_photo'}
            >
              <input
                type="radio"
                name="exportType"
                value="motion_photo"
                bind:group={selectedType}
              />
              <div class="option-icon-box text-emerald-400">
                <Sparkles size={18} />
              </div>
              <div class="option-text">
                <span class="option-name">Motion Photo (Live)</span>
                <span class="option-desc">JPEG with embedded video</span>
              </div>
            </label>
          {/if}
        </div>

        <!-- Metadata & Location Options -->
        {#if selectedType !== 'bts_only'}
          <div class="metadata-section">
            <div class="section-label">METADATA &amp; LOCATION</div>

            <div class="checkbox-row">
              <label class="toggle-label">
                <input type="checkbox" bind:checked={embedExif} />
                <span>Embed full EXIF timestamps &amp; camera profile</span>
              </label>

              {#if memory.location}
                <label class="toggle-label">
                  <input type="checkbox" bind:checked={embedGps} />
                  <span>Embed GPS coordinates ({memory.location.latitude.toFixed(3)}, {memory.location.longitude.toFixed(3)})</span>
                </label>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Remember as Default Option -->
        <div class="default-preference-row">
          <label class="toggle-label make-default-label">
            <input type="checkbox" bind:checked={makeDefault} />
            <span>Always use these settings for 1-click downloads (don't ask again)</span>
          </label>
        </div>
      </div>

      <!-- Footer Buttons -->
      <div class="modal-footer">
        <button
          type="button"
          class="btn-cancel"
          on:click={closeExportModal}
          disabled={isExporting}
        >
          Cancel
        </button>

        <button
          type="button"
          class="btn-confirm"
          on:click={handleConfirmExport}
          disabled={isExporting}
        >
          {#if isExporting}
            <Loader2 size={15} class="animate-spin" />
            <span>Exporting...</span>
          {:else if exportSuccess}
            <Check size={15} class="text-emerald-400" />
            <span>Saved!</span>
          {:else}
            <Download size={15} />
            <span>Export &amp; Save...</span>
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.82);
    backdrop-filter: blur(16px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1100;
    padding: 16px;
    animation: fadeIn 0.16s ease-out;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .modal-card {
    width: 100%;
    max-width: 480px;
    background: #111118;
    border: 1px solid var(--border-medium);
    border-radius: var(--radius-lg);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.9);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes scaleIn {
    from {
      opacity: 0;
      transform: scale(0.95);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 16px 20px;
    border-bottom: 1px solid var(--border-subtle);
  }

  .header-title-group {
    display: flex;
    align-items: center;
    gap: 12px;
  }

  .icon-wrap {
    width: 36px;
    height: 36px;
    border-radius: var(--radius-md);
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.25);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .modal-title {
    font-size: 15px;
    font-weight: 700;
    color: #ffffff;
  }

  .modal-subtitle {
    font-size: 11.5px;
    color: var(--text-secondary);
  }

  .close-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: var(--radius-full);
    background: transparent;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .close-btn:hover {
    background: #1e1e2c;
    color: #ffffff;
  }

  .modal-body {
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-height: 70vh;
    overflow-y: auto;
  }

  .section-label {
    font-size: 10px;
    font-weight: 800;
    color: var(--text-muted);
    letter-spacing: 0.08em;
  }

  .format-options-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 8px;
  }

  .format-option-card {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: #161622;
    border: 1.5px solid var(--border-subtle);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: all 0.16s ease;
  }

  .format-option-card input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .format-option-card:hover:not(.disabled) {
    background: #1c1c2c;
    border-color: var(--border-medium);
  }

  .format-option-card.selected {
    background: rgba(56, 189, 248, 0.12);
    border-color: #38bdf8;
    box-shadow: 0 0 0 1px #38bdf8;
  }

  .format-option-card.disabled {
    opacity: 0.35;
    cursor: not-allowed;
  }

  .option-icon-box {
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .format-option-card.selected .option-icon-box {
    color: #38bdf8;
  }

  .option-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .option-name {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .option-desc {
    font-size: 10.5px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .metadata-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--border-subtle);
  }

  .checkbox-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .toggle-label {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    font-size: 12px;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .toggle-label input[type="checkbox"] {
    accent-color: #38bdf8;
    cursor: pointer;
  }

  .default-preference-row {
    padding-top: 8px;
    border-top: 1px solid var(--border-subtle);
  }

  .make-default-label {
    font-weight: 600;
    color: #38bdf8;
  }

  .modal-footer {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 10px;
    padding: 14px 20px;
    background: #0d0d14;
    border-top: 1px solid var(--border-subtle);
  }

  .btn-cancel {
    padding: 8px 16px;
    background: transparent;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    color: var(--text-secondary);
    font-size: 12.5px;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .btn-cancel:hover:not(:disabled) {
    background: #181824;
    color: #ffffff;
  }

  .btn-confirm {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    padding: 8px 20px;
    background: #38bdf8;
    border: none;
    border-radius: var(--radius-full);
    color: #09090b;
    font-size: 12.5px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.2s ease;
    box-shadow: 0 2px 10px rgba(56, 189, 248, 0.35);
  }

  .btn-confirm:hover:not(:disabled) {
    background: #7dd3fc;
    box-shadow: 0 4px 16px rgba(56, 189, 248, 0.45);
    transform: translateY(-1px);
  }

  :global(.animate-spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }
</style>
