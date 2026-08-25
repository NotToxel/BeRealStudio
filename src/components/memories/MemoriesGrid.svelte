<script lang="ts">
  import { filteredMemories, openFeedAt } from '$lib/memoriesStore';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import Images from 'lucide-svelte/icons/images';
  import Sparkles from 'lucide-svelte/icons/sparkles';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import MessageSquare from 'lucide-svelte/icons/message-square';

  function handleMemoryClick(memory: any) {
    openFeedAt(memory);
  }
</script>

<div class="memories-grid-wrapper">
  {#if $filteredMemories.length === 0}
    <div class="empty-memories-state">
      <div class="empty-icon-wrap">
        <Images size={28} class="text-muted" />
      </div>
      <h3 class="empty-title">No Memories Match Your Filter</h3>
      <p class="empty-desc">Try clearing your search query or adjusting your filters above.</p>
    </div>
  {:else}
    <div class="memories-grid">
      {#each $filteredMemories as memory (memory.id)}
        <div
          class="grid-card-wrap"
          role="button"
          tabindex="0"
          on:click={() => handleMemoryClick(memory)}
          on:keydown={(e) => (e.key === 'Enter' || e.key === ' ') && handleMemoryClick(memory)}
        >
          <DualCameraFrame
            primarySrc={memory.primaryPath}
            secondarySrc={memory.secondaryPath}
            btsSrc={memory.btsPath}
            isVideo={memory.isVideo}
            alt="BeReal {memory.dateFormatted}"
            dayNumberOverlay={memory.dayNumber}
            badgeText={memory.retakeCounter > 0 ? `(${memory.retakeCounter})` : memory.btsPath ? 'BTS' : ''}
            size="md"
            interactive={false}
          />

          <!-- Hover Overlay Subtext info -->
          <div class="card-caption-strip">
            <span class="card-date-text">{memory.dateFormatted}</span>
            {#if memory.locationName}
              <div class="card-loc-pill" title={memory.locationName}>
                <MapPin size={10} />
                <span>{memory.city || memory.locationName}</span>
              </div>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .memories-grid-wrapper {
    width: 100%;
    min-height: 400px;
  }

  .memories-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
    gap: 14px;
    width: 100%;
  }

  .grid-card-wrap {
    position: relative;
    cursor: pointer;
    border-radius: 18px;
    transition: transform 0.22s cubic-bezier(0.34, 1.56, 0.64, 1), box-shadow 0.22s ease;
    outline: none;
  }

  .grid-card-wrap:hover {
    transform: translateY(-4px) scale(1.02);
    box-shadow: 0 12px 28px rgba(0, 0, 0, 0.7);
    z-index: 5;
  }

  .grid-card-wrap:focus-visible {
    box-shadow: 0 0 0 3px #ffe600;
  }

  .card-caption-strip {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 6px;
    padding: 6px 4px 2px 4px;
    font-size: 11px;
    color: var(--text-secondary);
  }

  .card-date-text {
    font-weight: 600;
    color: var(--text-main);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .card-loc-pill {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    font-size: 10px;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 90px;
  }

  .empty-memories-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 60px 20px;
    text-align: center;
    background: #0f0f15;
    border: 1px dashed var(--border-medium);
    border-radius: var(--radius-lg);
  }

  .empty-icon-wrap {
    width: 56px;
    height: 56px;
    border-radius: 50%;
    background: #181822;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 12px;
  }

  .empty-title {
    font-size: 16px;
    font-weight: 700;
    color: #ffffff;
    margin-bottom: 4px;
  }

  .empty-desc {
    font-size: 12.5px;
    color: var(--text-muted);
    max-width: 320px;
  }

  @media (max-width: 600px) {
    .memories-grid {
      grid-template-columns: repeat(2, 1fr);
      gap: 10px;
    }
  }
</style>
