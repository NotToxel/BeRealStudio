<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    activeFeedMemory,
    closeFeed,
    filteredMemories,
    explorerData,
  } from '$lib/memoriesStore';
  import type { ExplorerMemory } from '$lib/types';
  import DualCameraFrame from './DualCameraFrame.svelte';
  import MemoryActionMenu from './MemoryActionMenu.svelte';
  import ArrowLeft from 'lucide-svelte/icons/arrow-left';
  import MapPin from 'lucide-svelte/icons/map-pin';
  import Clock from 'lucide-svelte/icons/clock';
  import Calendar from 'lucide-svelte/icons/calendar';

  let feedScrollContainer: HTMLElement | null = null;
  let activeVisibleMemory: ExplorerMemory | null = null;
  let hasScrolledInitial = false;

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      closeFeed();
    }
  }

  // When activeFeedMemory changes, auto-scroll to that memory item in the feed
  $: if ($activeFeedMemory && feedScrollContainer) {
    scrollToMemory($activeFeedMemory.id);
  }

  async function scrollToMemory(id: string) {
    await tick();
    const el = document.getElementById(`feed-item-${id}`);
    if (el && feedScrollContainer) {
      el.scrollIntoView({ behavior: 'auto', block: 'start' });
      activeVisibleMemory = $activeFeedMemory;
    }
  }

  function handleScroll() {
    if (!feedScrollContainer) return;
    // Find item currently closest to top
    const containerTop = feedScrollContainer.getBoundingClientRect().top;
    for (const m of $filteredMemories) {
      const el = document.getElementById(`feed-item-${m.id}`);
      if (el) {
        const rect = el.getBoundingClientRect();
        if (rect.top <= containerTop + 140 && rect.bottom >= containerTop + 40) {
          activeVisibleMemory = m;
          break;
        }
      }
    }
  }

  $: currentMemory = activeVisibleMemory || $activeFeedMemory || $filteredMemories[0];
  $: currentIndex = currentMemory ? $filteredMemories.findIndex((m) => m.id === currentMemory.id) : 0;

  $: userName = $explorerData?.userName || 'toxel';
  $: userFullname = $explorerData?.userFullname || '';
  $: profilePic = $explorerData?.profilePictureDataUrl || '';
</script>

<svelte:window on:keydown={handleKeydown} />

{#if $activeFeedMemory}
  <div
    class="feed-modal-backdrop"
    role="dialog"
    aria-modal="true"
    tabindex="-1"
    on:click={(e) => e.target === e.currentTarget && closeFeed()}
    on:keydown={(e) => e.key === 'Escape' && closeFeed()}
  >
    <div
      class="feed-modal-shell"
      role="document"
    >
      <!-- Sticky Top Bar (shows active visible memory date) -->
      <div class="feed-top-bar">
        <button
          type="button"
          class="back-nav-btn"
          on:click={closeFeed}
          title="Back to Grid / Calendar (Esc)"
          aria-label="Close feed"
        >
          <ArrowLeft size={18} />
          <span>Memories</span>
        </button>

        <div class="top-date-indicator">
          {#if currentMemory}
            <span class="top-date-text">{currentMemory.dateFormatted}</span>
            <span class="top-index-text">{currentIndex + 1} of {$filteredMemories.length}</span>
          {/if}
        </div>

        <div class="top-actions">
          {#if currentMemory}
            <MemoryActionMenu memory={currentMemory} />
          {/if}
        </div>
      </div>

      <!-- Continuous Vertical Feed Scroll Container -->
      <div
        bind:this={feedScrollContainer}
        class="feed-scroll-container"
        on:scroll={handleScroll}
      >
        {#each $filteredMemories as memory (memory.id)}
          <article
            id="feed-item-{memory.id}"
            class="feed-post-card"
          >
            <!-- Post Header -->
            <div class="post-header-row">
              <div class="user-avatar-wrap">
                {#if profilePic}
                  <img src={profilePic} alt={userName} class="user-avatar-img" />
                {:else}
                  <div class="user-avatar-placeholder">
                    <span>{userName.charAt(0).toUpperCase()}</span>
                  </div>
                {/if}
              </div>

              <div class="user-meta-column">
                <div class="user-name-row">
                  <span class="user-name">{userName}</span>
                  {#if userFullname}
                    <span class="user-fullname">• {userFullname}</span>
                  {/if}
                </div>
                <div class="post-time-row">
                  <span class="date-badge">{memory.dateFormatted}</span>
                  <span class="separator">•</span>
                  <Clock size={11} class="text-muted" />
                  <span class="time-text">{memory.timeFormatted}</span>
                  {#if memory.retakeCounter > 0}
                    <span class="retake-tag">• {memory.retakeCounter} retakes</span>
                  {/if}
                </div>
              </div>

              <div class="post-header-actions">
                <MemoryActionMenu {memory} />
              </div>
            </div>

            <!-- BeReal Dual Camera Frame (Click to swap, drag to move PIP) -->
            <div class="dual-frame-wrapper">
              <DualCameraFrame
                primarySrc={memory.primaryPath}
                secondarySrc={memory.secondaryPath}
                btsSrc={memory.btsPath}
                isVideo={memory.isVideo}
                alt="BeReal {memory.dateFormatted}"
                size="lg"
                interactive={true}
              />
            </div>

            <!-- Post Footer: Location, Caption, and Metadata -->
            <div class="post-footer-section">
              {#if memory.locationName}
                <div class="location-badge-pill" title="GPS: {memory.location?.latitude.toFixed(4)}, {memory.location?.longitude.toFixed(4)}">
                  <MapPin size={13} class="text-emerald-400" />
                  <span>{memory.locationName}</span>
                </div>
              {/if}

              {#if memory.caption}
                <div class="caption-block">
                  <p class="caption-text">{memory.caption}</p>
                </div>
              {/if}
            </div>

            <!-- Subtle Post Divider -->
            <div class="feed-post-divider"></div>
          </article>
        {/each}
      </div>
    </div>
  </div>
{/if}

<style>
  .feed-modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.9);
    backdrop-filter: blur(20px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 500;
    animation: fadeIn 0.16s ease-out;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }

  .feed-modal-shell {
    position: relative;
    width: 100%;
    max-width: 500px;
    height: 96vh;
    display: flex;
    flex-direction: column;
    background: #000000;
    border: 1px solid var(--border-medium);
    border-radius: 28px;
    overflow: hidden;
    box-shadow: 0 24px 64px rgba(0, 0, 0, 0.95);
    animation: scaleIn 0.22s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes scaleIn {
    from {
      opacity: 0;
      transform: scale(0.94);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }

  .feed-top-bar {
    position: sticky;
    top: 0;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 18px;
    background: rgba(0, 0, 0, 0.88);
    backdrop-filter: blur(14px);
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    z-index: 40;
    flex-shrink: 0;
  }

  .back-nav-btn {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    color: #ffffff;
    font-size: 13px;
    font-weight: 600;
    cursor: pointer;
    padding: 6px 10px;
    border-radius: var(--radius-full);
    transition: background var(--transition-fast);
  }

  .back-nav-btn:hover {
    background: rgba(255, 255, 255, 0.12);
  }

  .top-date-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .top-date-text {
    font-size: 13px;
    font-weight: 700;
    color: #ffffff;
  }

  .top-index-text {
    font-size: 10.5px;
    color: var(--text-muted);
  }

  .top-actions {
    display: flex;
    align-items: center;
  }

  /* Continuous Scroll Container */
  .feed-scroll-container {
    flex: 1;
    overflow-y: auto;
    scroll-behavior: smooth;
    display: flex;
    flex-direction: column;
    padding: 16px 18px 40px 18px;
    gap: 28px;
  }

  .feed-post-card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    width: 100%;
  }

  .post-header-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .user-avatar-wrap {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    overflow: hidden;
    background: #181824;
    border: 1px solid var(--border-subtle);
    flex-shrink: 0;
  }

  .user-avatar-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .user-avatar-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #ffffff;
    font-weight: 700;
    font-size: 14px;
    background: linear-gradient(135deg, #38bdf8 0%, #a855f7 100%);
  }

  .user-meta-column {
    display: flex;
    flex-direction: column;
    gap: 2px;
    flex: 1;
  }

  .user-name-row {
    display: flex;
    align-items: baseline;
    gap: 5px;
  }

  .user-name {
    font-size: 13.5px;
    font-weight: 700;
    color: #ffffff;
  }

  .user-fullname {
    font-size: 12px;
    color: var(--text-muted);
  }

  .post-time-row {
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 11px;
    color: var(--text-muted);
  }

  .date-badge {
    color: #ffffff;
    font-weight: 600;
  }

  .separator {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .retake-tag {
    color: #fbbf24;
    font-weight: 500;
  }

  .post-header-actions {
    margin-left: auto;
  }

  .dual-frame-wrapper {
    width: 100%;
  }

  .post-footer-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .location-badge-pill {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    padding: 4px 10px;
    background: rgba(52, 211, 153, 0.1);
    border: 1px solid rgba(52, 211, 153, 0.25);
    border-radius: var(--radius-full);
    font-size: 11.5px;
    font-weight: 600;
    color: #34d399;
    align-self: flex-start;
  }

  .caption-block {
    padding: 4px 0;
  }

  .caption-text {
    font-size: 14px;
    line-height: 1.45;
    color: #ffffff;
    font-weight: 500;
    margin: 0;
  }

  .feed-post-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.07);
    margin-top: 14px;
    width: 100%;
  }

  @media (max-width: 600px) {
    .feed-modal-shell {
      height: 100vh;
      max-height: 100vh;
      border-radius: 0;
      border: none;
    }
  }
</style>
