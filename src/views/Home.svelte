<script lang="ts">
  import {
    currentView,
    activeFeature,
    toolkitConfig,
    recapperConfig,
    getPreferredRecapInputFolder,
    getSensibleRecapOutputPath,
  } from '$lib/stores';
  import Camera from 'lucide-svelte/icons/camera';
  import Film from 'lucide-svelte/icons/film';
  import ArrowRight from 'lucide-svelte/icons/arrow-right';

  function openToolkit() {
    activeFeature.set('toolkit');
    currentView.set('toolkit-config');
  }

  function openRecapper() {
    if (!$recapperConfig.inputFolder && $toolkitConfig.outputPath) {
      $recapperConfig.inputFolder = getPreferredRecapInputFolder($toolkitConfig.outputPath, $toolkitConfig.createCombined);
      if (!$recapperConfig.outputPath) {
        $recapperConfig.outputPath = getSensibleRecapOutputPath($recapperConfig.inputFolder);
      }
    }
    activeFeature.set('recapper');
    currentView.set('recapper-config');
  }
</script>

<div class="home-container">
  <!-- Hero Section -->
  <div class="hero">
    <h1 class="hero-title">
      Your BeReal Archive,<br />
      <span class="gradient-text-yellow">Unified &amp; Perfected.</span>
    </h1>

    <p class="hero-subtitle text-secondary">
      Restore authentic timestamps, composite dual-camera memories, mux motion photos, and render music-synchronized recap videos.
    </p>
  </div>

  <!-- Dual Main Action Cards -->
  <div class="cards-grid">
    <!-- Card 1: Photo Processing -->
    <button type="button" class="feature-card card-toolkit card-clickable text-left" on:click={openToolkit}>
      <div class="card-top-row">
        <div class="card-icon icon-yellow">
          <Camera size={24} />
        </div>
      </div>

      <div class="card-body">
        <h2 class="title-md font-bold text-white">Photo Processing Suite</h2>
        <p class="card-desc text-secondary">
          Extract EXIF timestamps, convert formats, composite Picture-in-Picture photos, and mux motion photos.
        </p>
      </div>

      <div class="card-footer">
        <span class="btn btn-accent-yellow btn-sm">
          Process Photos <ArrowRight size={14} />
        </span>
      </div>
    </button>

    <!-- Card 2: Recap Video -->
    <button type="button" class="feature-card card-recapper card-clickable text-left" on:click={openRecapper}>
      <div class="card-top-row">
        <div class="card-icon icon-violet">
          <Film size={24} />
        </div>
      </div>

      <div class="card-body">
        <h2 class="title-md font-bold text-white">Recap Video Generator</h2>
        <p class="card-desc text-secondary">
          Render a music-synchronized vertical video slideshow with live typography preview and reverse geocoding.
        </p>
      </div>

      <div class="card-footer">
        <span class="btn btn-accent-violet btn-sm">
          Create Recap <ArrowRight size={14} />
        </span>
      </div>
    </button>
  </div>
</div>

<style>
  .home-container {
    display: flex;
    flex-direction: column;
    gap: 36px;
    padding: 18px 0 40px 0;
  }

  .hero {
    display: flex;
    flex-direction: column;
    gap: 12px;
    max-width: 740px;
  }

  .hero-title {
    font-size: 38px;
    font-weight: 800;
    line-height: 1.15;
    letter-spacing: -0.035em;
    color: #ffffff;
  }

  .hero-subtitle {
    font-size: 15px;
    line-height: 1.5;
    max-width: 600px;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 22px;
  }

  .feature-card {
    display: flex;
    flex-direction: column;
    gap: 16px;
    padding: 28px;
    background: #111116;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-lg);
    transition: all var(--transition-normal);
    text-align: left;
    color: inherit;
    cursor: pointer;
    position: relative;
  }

  .card-toolkit:hover {
    border-color: rgba(255, 230, 0, 0.4);
    box-shadow: 0 8px 30px rgba(255, 230, 0, 0.08);
    transform: translateY(-2px);
    background: #15151c;
  }

  .card-recapper:hover {
    border-color: rgba(139, 92, 246, 0.45);
    box-shadow: 0 8px 30px rgba(139, 92, 246, 0.12);
    transform: translateY(-2px);
    background: #15151c;
  }

  .card-top-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .card-icon {
    width: 50px;
    height: 50px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-yellow {
    background: rgba(255, 230, 0, 0.12);
    color: #ffe600;
    border: 1px solid rgba(255, 230, 0, 0.25);
  }

  .icon-violet {
    background: rgba(139, 92, 246, 0.12);
    color: #c084fc;
    border: 1px solid rgba(139, 92, 246, 0.25);
  }

  .card-body {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
  }

  .card-desc {
    font-size: 13.5px;
    line-height: 1.5;
  }

  .card-footer {
    padding-top: 8px;
  }

  @media (max-width: 768px) {
    .cards-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
