<script lang="ts">
  import { globalPerspective } from '$lib/memoriesStore';

  function setPerspective(p: 'primary' | 'secondary') {
    globalPerspective.set(p);
  }
</script>

<div class="perspective-floating-pill" role="group" aria-label="Toggle camera perspective">
  <!-- Primary / Standard Mode Button -->
  <button
    type="button"
    class="perspective-btn"
    class:is-active={$globalPerspective === 'primary'}
    on:click={() => setPerspective('primary')}
    title="Standard View (Main / Back Camera)"
    aria-label="Standard View (Main Camera Large)"
    aria-pressed={$globalPerspective === 'primary'}
  >
    <div class="pictogram-box standard-box">
      <div class="pictogram-canvas"></div>
      <div class="pictogram-pip pip-top-left"></div>
    </div>
  </button>

  <div class="pill-divider"></div>

  <!-- Secondary / Reversed Mode Button -->
  <button
    type="button"
    class="perspective-btn"
    class:is-active={$globalPerspective === 'secondary'}
    on:click={() => setPerspective('secondary')}
    title="Reversed View (Selfie / Front Camera)"
    aria-label="Reversed View (Selfie Camera Large)"
    aria-pressed={$globalPerspective === 'secondary'}
  >
    <div class="pictogram-box reversed-box">
      <div class="pictogram-canvas canvas-filled"></div>
      <div class="pictogram-pip pip-top-left pip-hollow"></div>
    </div>
  </button>
</div>

<style>
  .perspective-floating-pill {
    position: fixed;
    bottom: 24px;
    left: 28px;
    display: inline-flex;
    align-items: center;
    background: rgba(14, 14, 22, 0.92);
    backdrop-filter: blur(18px);
    border: 1px solid rgba(255, 255, 255, 0.18);
    border-radius: 14px;
    padding: 3px 4px;
    gap: 3px;
    box-shadow: 0 10px 32px rgba(0, 0, 0, 0.8), 0 0 0 1px rgba(255, 255, 255, 0.05);
    z-index: 100;
    user-select: none;
    transition: transform var(--transition-fast), border-color var(--transition-fast);
  }

  .perspective-floating-pill:hover {
    border-color: rgba(255, 255, 255, 0.3);
    transform: translateY(-2px);
  }

  .perspective-btn {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
    width: 38px;
    height: 38px;
    padding: 0;
    background: transparent;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    transition: background 0.15s ease, transform 0.15s ease;
  }

  .perspective-btn:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .perspective-btn.is-active {
    background: rgba(255, 255, 255, 0.18);
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.35);
  }

  .perspective-btn.is-active .pictogram-box {
    filter: drop-shadow(0 0 4px rgba(255, 255, 255, 0.5));
  }

  .pill-divider {
    width: 1px;
    height: 20px;
    background: rgba(255, 255, 255, 0.12);
  }

  /* Pictogram Box Styling matching official BeReal interface */
  .pictogram-box {
    position: relative;
    width: 20px;
    height: 26px;
    border: 1.75px solid #ffffff;
    border-radius: 4.5px;
    background: transparent;
    box-sizing: border-box;
    display: flex;
  }

  .pictogram-canvas {
    width: 100%;
    height: 100%;
    border-radius: 2.5px;
  }

  .canvas-filled {
    background: rgba(255, 255, 255, 0.35);
  }

  .pictogram-pip {
    position: absolute;
    width: 7px;
    height: 9px;
    background: #ffffff;
    border-radius: 2px;
  }

  .pip-top-left {
    top: 2px;
    left: 2px;
  }

  .pip-hollow {
    background: #000000;
    border: 1px solid #ffffff;
  }
</style>
