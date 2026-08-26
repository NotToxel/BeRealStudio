<script lang="ts">
  import {
    exportModalState,
    closeExportModal,
    photoExportPreferences,
    videoExportPreferences,
    btsExportPreferences,
    isMemoryVideo,
    type PhotoExportPreferences,
    type VideoExportPreferences,
    type BtsExportPreferences,
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
  import Lock from 'lucide-svelte/icons/lock';

  let isExporting = false;
  let exportSuccess = false;

  $: state = $exportModalState;
  $: memory = state?.memory;
  $: isVideo = isMemoryVideo(memory);

  type AnyExportType = 'combined_pip' | 'combined_sidebyside' | 'primary_only' | 'secondary_only' | 'bts_only' | 'motion_photo' | 'apple_live_photo';

  let selectedType: AnyExportType = 'combined_pip';
  let selectedFormat: 'Jpeg' | 'Png' | 'WebP' = 'Jpeg';
  let imageQuality: number = 95;
  let embedExif = true;
  let embedGps = true;
  let makeDefault = false;

  // Sync state when modal is opened for a specific memory
  $: if (state?.isOpen && memory) {
    if (isVideo) {
      selectedType = $videoExportPreferences.exportType || 'combined_pip';
      embedGps = $videoExportPreferences.embedGps ?? true;
      makeDefault = $videoExportPreferences.isDefaultSet ?? false;
    } else {
      selectedType = $photoExportPreferences.exportType || 'combined_pip';
      selectedFormat = $photoExportPreferences.format || 'Jpeg';
      imageQuality = $photoExportPreferences.quality || 95;
      embedExif = $photoExportPreferences.embedExif ?? true;
      embedGps = $photoExportPreferences.embedGps ?? true;
      makeDefault = $photoExportPreferences.isDefaultSet ?? false;
    }
  }

  $: if (memory && !memory.secondaryPath && (selectedType === 'combined_pip' || selectedType === 'combined_sidebyside' || selectedType === 'secondary_only')) {
    selectedType = 'primary_only';
  }

  $: if (memory && isVideo && (selectedType === 'motion_photo' || selectedType === 'apple_live_photo')) {
    selectedType = 'combined_pip';
  }

  $: if (memory && !memory.btsPath && (selectedType === 'bts_only' || selectedType === 'motion_photo' || selectedType === 'apple_live_photo')) {
    selectedType = 'combined_pip';
  }

  // Force JPEG for motion photos and Apple Live Photos
  $: if (selectedType === 'motion_photo' || selectedType === 'apple_live_photo') {
    if (selectedFormat !== 'Jpeg') {
      selectedFormat = 'Jpeg';
    }
  }

  async function handleConfirmExport() {
    if (!memory || !memory.primaryPath) return;

    try {
      isExporting = true;
      exportSuccess = false;

      // Save preference partitioned specifically by media kind
      if (selectedType === 'bts_only') {
        btsExportPreferences.set({
          exportType: 'bts_only',
          isDefaultSet: makeDefault,
        });
      } else if (isVideo) {
        videoExportPreferences.set({
          exportType: selectedType as any,
          embedGps,
          isDefaultSet: makeDefault,
        });
      } else {
        photoExportPreferences.set({
          exportType: selectedType as any,
          format: selectedFormat,
          quality: imageQuality,
          embedExif,
          embedGps,
          isDefaultSet: makeDefault,
        });
      }

      const datePrefix = memory.takenAt ? memory.takenAt.slice(0, 10) : 'bereal';
      const isVideoExport = isVideo || selectedType === 'bts_only';
      const ext = isVideoExport ? 'mp4' : selectedFormat.toLowerCase() === 'png' ? 'png' : selectedFormat.toLowerCase() === 'webp' ? 'webp' : 'jpg';
      const defaultFilename = `${datePrefix}_${selectedType}.${ext}`;

      const filters = isVideoExport
        ? [{ name: 'MP4 Video', extensions: ['mp4'] }]
        : selectedType === 'apple_live_photo'
          ? [{ name: 'Apple Live Photo (.jpg + .mov)', extensions: ['jpg'] }]
          : [{ name: `${selectedFormat} Image`, extensions: [ext] }];

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
        format: isVideoExport ? 'Jpeg' : selectedFormat,
        quality: imageQuality,
        embedExif: isVideoExport ? true : embedExif,
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
              <span class="option-name">{isVideo ? 'PIP Video' : 'Picture-in-Picture'}</span>
              <span class="option-desc">{isVideo ? 'Main video with selfie inset (.mp4)' : 'Main photo + selfie camera inset'}</span>
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
              <span class="option-name">{isVideo ? 'Side-by-Side Video' : 'Side-by-Side'}</span>
              <span class="option-desc">{isVideo ? 'Both video angles side by side' : 'Both cameras side by side'}</span>
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
              <span class="option-name">{isVideo ? 'Main Video Only' : 'Main Camera Only'}</span>
              <span class="option-desc">{isVideo ? 'Primary camera video clip (.mp4)' : 'High-res primary photo'}</span>
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
              <span class="option-name">{isVideo ? 'Selfie Video Only' : 'Selfie Camera Only'}</span>
              <span class="option-desc">{isVideo ? 'Front-facing selfie video (.mp4)' : 'Front-facing camera photo'}</span>
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
                <span class="option-desc">Behind-the-scenes clip (.mp4)</span>
              </div>
            </label>

            <!-- Option: Motion Photo (Samsung & Google) (Only for photo memories) -->
            {#if !isVideo}
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
                  <span class="option-name">Motion Photo (Android)</span>
                  <span class="option-desc">Samsung SEFH &amp; Google XMP</span>
                </div>
              </label>

              <!-- Option: Apple Live Photo (Pair) -->
              <label
                class="format-option-card"
                class:selected={selectedType === 'apple_live_photo'}
              >
                <input
                  type="radio"
                  name="exportType"
                  value="apple_live_photo"
                  bind:group={selectedType}
                />
                <div class="option-icon-box text-sky-400">
                  <Sparkles size={18} />
                </div>
                <div class="option-text">
                  <span class="option-name">Apple Live Photo (iOS)</span>
                  <span class="option-desc">Paired .jpg + .mov Live Photo</span>
                </div>
              </label>
            {/if}
          {/if}
        </div>

        <!-- Format & Quality Section -->
        {#if isVideo || selectedType === 'bts_only'}
          <div class="format-quality-section">
            <div class="section-label-row">
              <span class="section-label">VIDEO OUTPUT FORMAT</span>
              <span class="format-req-badge">H.264 Video • AAC Audio</span>
            </div>

            <div class="video-format-banner">
              <div class="video-format-icon">
                <Film size={16} class="text-sky-400" />
              </div>
              <div class="video-format-info">
                <span class="video-format-title">MP4 Video Format</span>
                <span class="video-format-desc">Native high-definition video with original audio synchronization</span>
              </div>
            </div>
          </div>
        {:else}
          <div class="format-quality-section">
            <div class="section-label-row">
              <span class="section-label">OUTPUT FORMAT &amp; QUALITY</span>
              {#if selectedType === 'motion_photo' || selectedType === 'apple_live_photo'}
                <span class="format-req-badge">JPEG Required for Live Media</span>
              {/if}
            </div>

            <div class="format-pills-row">
              <button
                type="button"
                class="format-pill-btn"
                class:active={selectedFormat === 'Jpeg'}
                on:click={() => (selectedFormat = 'Jpeg')}
              >
                JPEG
              </button>
              <button
                type="button"
                class="format-pill-btn"
                class:active={selectedFormat === 'WebP'}
                disabled={selectedType === 'motion_photo' || selectedType === 'apple_live_photo'}
                title={selectedType === 'motion_photo' || selectedType === 'apple_live_photo' ? 'WebP format is not supported for Live & Motion Photos (JPEG required)' : 'WebP image'}
                on:click={() => (selectedFormat = 'WebP')}
              >
                <span>WEBP</span>
                {#if selectedType === 'motion_photo' || selectedType === 'apple_live_photo'}
                  <Lock size={11} class="lock-icon" />
                {/if}
              </button>
              <button
                type="button"
                class="format-pill-btn"
                class:active={selectedFormat === 'Png'}
                disabled={selectedType === 'motion_photo' || selectedType === 'apple_live_photo'}
                title={selectedType === 'motion_photo' || selectedType === 'apple_live_photo' ? 'PNG format is not supported for Live & Motion Photos (JPEG required)' : 'Lossless PNG'}
                on:click={() => (selectedFormat = 'Png')}
              >
                <span>PNG</span>
                {#if selectedType === 'motion_photo' || selectedType === 'apple_live_photo'}
                  <Lock size={11} class="lock-icon" />
                {/if}
              </button>
            </div>

            {#if selectedFormat === 'Jpeg'}
              <div class="quality-number-row">
                <span class="quality-label">JPEG Quality</span>
                <div class="number-stepper">
                  <button
                    type="button"
                    class="stepper-btn"
                    on:click={() => (imageQuality = Math.max(50, imageQuality - 5))}
                    disabled={imageQuality <= 50}
                  >-</button>
                  <input
                    type="number"
                    min="50"
                    max="100"
                    step="1"
                    bind:value={imageQuality}
                    class="quality-number-input"
                  />
                  <button
                    type="button"
                    class="stepper-btn"
                    on:click={() => (imageQuality = Math.min(100, imageQuality + 5))}
                    disabled={imageQuality >= 100}
                  >+</button>
                  <span class="unit-text">%</span>
                </div>
              </div>
            {/if}
          </div>
        {/if}

          <!-- Metadata & Location Options -->
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
    max-width: 540px;
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

  .section-label-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
  }

  .section-label {
    font-size: 10px;
    font-weight: 800;
    color: var(--text-muted);
    letter-spacing: 0.08em;
  }

  .format-req-badge {
    font-size: 9.5px;
    font-weight: 700;
    padding: 2px 7px;
    border-radius: var(--radius-full);
    background: rgba(56, 189, 248, 0.12);
    border: 1px solid rgba(56, 189, 248, 0.3);
    color: #38bdf8;
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
    line-height: 1.3;
    white-space: normal;
  }

  .format-quality-section {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding-top: 6px;
    border-top: 1px solid var(--border-subtle);
  }

  .video-format-banner {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    background: #14141e;
    border: 1px solid rgba(56, 189, 248, 0.25);
    border-radius: var(--radius-md);
  }

  .video-format-icon {
    width: 32px;
    height: 32px;
    border-radius: var(--radius-sm);
    background: rgba(56, 189, 248, 0.12);
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .video-format-info {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .video-format-title {
    font-size: 12px;
    font-weight: 700;
    color: #ffffff;
  }

  .video-format-desc {
    font-size: 10.5px;
    color: var(--text-secondary);
  }

  .format-pills-row {
    display: flex;
    gap: 8px;
  }

  .format-pill-btn {
    flex: 1;
    padding: 7px 12px;
    background: #161622;
    border: 1.5px solid var(--border-subtle);
    border-radius: var(--radius-sm);
    color: var(--text-secondary);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 5px;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .format-pill-btn:hover:not(:disabled) {
    background: #1f1f2e;
    color: #ffffff;
  }

  .format-pill-btn.active {
    background: rgba(56, 189, 248, 0.15);
    border-color: #38bdf8;
    color: #38bdf8;
  }

  .format-pill-btn:disabled {
    opacity: 0.28;
    cursor: not-allowed;
    background: rgba(14, 14, 20, 0.4);
    border: 1.5px dashed rgba(255, 255, 255, 0.08);
    color: var(--text-muted);
    text-decoration: line-through;
    pointer-events: none;
  }

  :global(.lock-icon) {
    color: var(--text-muted);
    opacity: 0.7;
  }

  .quality-number-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: #14141e;
    padding: 8px 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--border-subtle);
  }

  .quality-label {
    font-size: 12px;
    font-weight: 600;
    color: #ffffff;
  }

  .number-stepper {
    display: flex;
    align-items: center;
    gap: 6px;
  }

  .stepper-btn {
    width: 26px;
    height: 26px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    border: 1px solid var(--border-subtle);
    color: #ffffff;
    font-weight: 700;
    font-size: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .stepper-btn:hover:not(:disabled) {
    background: rgba(56, 189, 248, 0.2);
    border-color: #38bdf8;
    color: #38bdf8;
  }

  .stepper-btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }

  .quality-number-input {
    width: 48px;
    height: 26px;
    background: #09090e;
    border: 1px solid var(--border-medium);
    border-radius: 6px;
    color: #38bdf8;
    font-weight: 700;
    font-size: 13px;
    text-align: center;
    outline: none;
    appearance: textfield;
    -moz-appearance: textfield;
  }

  .quality-number-input::-webkit-outer-spin-button,
  .quality-number-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    margin: 0;
  }

  .unit-text {
    font-size: 12px;
    font-weight: 700;
    color: var(--text-muted);
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
