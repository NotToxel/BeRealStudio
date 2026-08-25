<script lang="ts">
  import ShieldCheck from 'lucide-svelte/icons/shield-check';
  import Cpu from 'lucide-svelte/icons/cpu';
  import Layers from 'lucide-svelte/icons/layers';
  import Download from 'lucide-svelte/icons/download';
  import Award from 'lucide-svelte/icons/award';
  import ExternalLink from 'lucide-svelte/icons/external-link';
  import Sparkles from 'lucide-svelte/icons/sparkles';
</script>

<div class="about-view">
  <!-- Hero Section -->
  <div class="about-hero">
    <div class="logo-circle">
      <span class="dot"></span>
    </div>
    <h1 class="title-lg font-bold">BeReal Studio</h1>
    <span class="version-tag">Version 1.0.0 &bull; Open Source Desktop Suite</span>
    <p class="subtitle text-secondary">
      The all-in-one, privacy-first desktop suite to restore, process, composite, and recap your personal BeReal GDPR archives.
    </p>
  </div>

  <div class="sections-list">
    <!-- 1. Privacy & Architecture -->
    <div class="card section-card">
      <div class="card-title-row">
        <ShieldCheck size={20} class="text-emerald-400" />
        <h2 class="title-sm">100% Private &amp; Local-First</h2>
      </div>

      <p class="text-secondary text-body">
        BeReal Studio processes all your memories entirely on your local machine using a native Rust engine. No photos, metadata, audio files, or personal information are ever uploaded to the cloud or third-party servers.
      </p>
    </div>

    <!-- 2. Tech Stack -->
    <div class="card section-card">
      <div class="card-title-row">
        <Cpu size={20} class="text-purple-400" />
        <h2 class="title-sm">Built with Modern Systems &amp; Web Tech</h2>
      </div>

      <div class="tech-grid">
        <div class="tech-item">
          <strong>Tauri v2 + Rust</strong>
          <span class="text-secondary">Ultra-lightweight native desktop shell and multi-threaded processing</span>
        </div>
        <div class="tech-item">
          <strong>Svelte 5 + Tailwind CSS v4</strong>
          <span class="text-secondary">Reactive, high-performance UI with customizable modern themes</span>
        </div>
        <div class="tech-item">
          <strong>Rayon Concurrency</strong>
          <span class="text-secondary">Multi-core parallel photo conversion delivering 5–10&times; faster batch exports</span>
        </div>
        <div class="tech-item">
          <strong>Symphonia Audio</strong>
          <span class="text-secondary">Pure Rust multi-format audio decoding for exact music-synced transitions</span>
        </div>
        <div class="tech-item">
          <strong>img-parts EXIF Muxer</strong>
          <span class="text-secondary">Lossless JPEG segment injection for timestamps, GPS, and IPTC captions</span>
        </div>
        <div class="tech-item">
          <strong>FFmpeg Streaming Pipe</strong>
          <span class="text-secondary">Raw video frame streaming for H.264 recap video encoding</span>
        </div>
      </div>
    </div>

    <!-- 3. GDPR Data Portability Guide -->
    <div class="card section-card">
      <div class="card-title-row">
        <Download size={20} class="text-sky-400" />
        <h2 class="title-sm">How to Download Your Archive from BeReal</h2>
      </div>

      <ol class="steps-list text-secondary">
        <li>Open the <strong>BeReal</strong> mobile app and tap your <strong>Profile icon</strong> in the top-right corner.</li>
        <li>Tap <strong>Help</strong> &rarr; Select <strong>Contact Us</strong>.</li>
        <li>Select <strong>Ask a Question</strong> &rarr; Tap <strong>Troubleshooting</strong> &rarr; Select <strong>Other</strong>.</li>
        <li>Tap <strong>Contact Us</strong> at the bottom of the article &rarr; Tap the <strong>Topic</strong> dropdown.</li>
        <li>Select the topic: <strong>"I'd like to request a copy of my data"</strong>.</li>
        <li>Type a short message with at least <strong>10 characters</strong> (e.g., <em>"Please provide a full copy of my account data"</em>) and submit your request.</li>
        <li>BeReal support will email you a secure download link containing your raw archive ZIP with <code>posts.json</code> and all full-resolution memories.</li>
      </ol>
    </div>

    <!-- 4. Credits & Acknowledgments -->
    <div class="card section-card">
      <div class="card-title-row">
        <Award size={20} class="text-amber-400" />
        <h2 class="title-sm">Credits &amp; Open Source Lineage</h2>
      </div>

      <p class="text-secondary text-body">
        BeReal Studio unites and builds upon the foundational architecture of two popular open-source projects:
      </p>

      <div class="credits-list">
        <a
          href="https://github.com/theOneAndOnlyOne/BeReel"
          target="_blank"
          rel="noreferrer"
          class="credit-card-link"
        >
          <div class="credit-card-head">
            <div class="credit-brand-title">
              <svg class="github-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
              </svg>
              <strong class="text-purple-300">BeReel (BeReal-Recapper)</strong>
            </div>
            <ExternalLink size={13} class="text-secondary" />
          </div>
          <span class="text-secondary text-desc">
            Music-synchronized video slideshow and reverse geocoding rules engine by <em>theOneAndOnlyOne</em>.
          </span>
          <span class="github-subtext font-mono text-purple-400">github.com/theOneAndOnlyOne/BeReel &rarr;</span>
        </a>

        <a
          href="https://github.com/hatobi/bereal-gdpr-photo-toolkit"
          target="_blank"
          rel="noreferrer"
          class="credit-card-link"
        >
          <div class="credit-card-head">
            <div class="credit-brand-title">
              <svg class="github-icon" width="16" height="16" viewBox="0 0 24 24" fill="currentColor">
                <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
              </svg>
              <strong class="text-amber-300">BeReal-GDPR-Photo-Toolkit</strong>
            </div>
            <ExternalLink size={13} class="text-secondary" />
          </div>
          <span class="text-secondary text-desc">
            JSON metadata parsing, EXIF injection, and Picture-in-Picture compositing by <em>hatobi</em>.
          </span>
          <span class="github-subtext font-mono text-amber-400">github.com/hatobi/bereal-gdpr-photo-toolkit &rarr;</span>
        </a>
      </div>
    </div>
  </div>
</div>

<style>
  .about-view {
    display: flex;
    flex-direction: column;
    gap: 28px;
    padding-bottom: 40px;
  }

  .about-hero {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
    gap: 8px;
    padding: 10px 0;
  }

  .logo-circle {
    width: 52px;
    height: 52px;
    background: #18181d;
    border: 1px solid rgba(255, 230, 0, 0.3);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-bottom: 4px;
    box-shadow: 0 0 16px rgba(255, 230, 0, 0.15);
  }

  .logo-circle .dot {
    width: 18px;
    height: 18px;
    background: #ffe600;
    border-radius: 50%;
    box-shadow: 0 0 10px rgba(255, 230, 0, 0.8);
  }

  .version-tag {
    font-size: 11.5px;
    font-family: var(--font-mono);
    color: var(--text-muted);
  }

  .subtitle {
    font-size: 14px;
    max-width: 580px;
    line-height: 1.5;
  }

  .sections-list {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .section-card {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .card-title-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .text-body {
    font-size: 13px;
    line-height: 1.6;
  }

  .tech-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .tech-item {
    background: #0e0e11;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 12px 14px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    font-size: 13px;
  }

  .tech-item strong {
    color: var(--text-main);
  }

  .tech-item span {
    font-size: 11.5px;
    line-height: 1.4;
  }

  .steps-list {
    margin-left: 20px;
    font-size: 13px;
    line-height: 1.7;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .credits-list {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .credit-card-link {
    background: #0e0e11;
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 14px;
    display: flex;
    flex-direction: column;
    gap: 6px;
    font-size: 13px;
    text-decoration: none;
    transition: all var(--transition-fast);
  }

  .credit-card-link:hover {
    background: #14141a;
    border-color: rgba(255, 255, 255, 0.2);
    transform: translateY(-2px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
  }

  .credit-card-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .credit-brand-title {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .github-icon {
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .credit-card-head strong {
    color: var(--text-main);
  }

  .text-desc {
    font-size: 12px;
    line-height: 1.45;
  }

  .github-subtext {
    font-size: 11.5px;
    font-weight: 500;
    margin-top: 4px;
  }

  @media (max-width: 768px) {
    .tech-grid, .credits-list {
      grid-template-columns: 1fr;
    }
  }
</style>
