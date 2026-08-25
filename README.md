# BeReal Studio 📸 🎬

<div align="center">
  <h3>Unified, Local-First Desktop Suite for BeReal GDPR Data Exports</h3>
  <p>Restore metadata, composite dual-camera memories, mux motion photos, and generate music-synchronized recap videos — 100% locally and privately.</p>
  <p>
    <a href="https://github.com/NotToxel/BeRealStudio"><img src="https://img.shields.io/badge/GitHub-NotToxel%2FBeRealStudio-yellow?style=flat&logo=github" alt="GitHub Repo" /></a>
    <img src="https://img.shields.io/badge/Version-1.4.0-blue?style=flat" alt="Version" />
    <img src="https://img.shields.io/badge/License-MIT-green?style=flat" alt="License" />
    <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-purple?style=flat" alt="Cross Platform" />
  </p>
</div>

---

## ⚡ Key Highlights & Features

### 📸 Photo Processing Suite
- **Metadata Restoration & EXIF Synchronization:** Reads `posts.json` timestamps and GPS coordinates, embedding `DateTimeOriginal`, `GPSLatitude`, `GPSLongitude`, and caption descriptions into EXIF/IPTC image headers without lossy re-encoding.
- **Picture-in-Picture & Dual-Camera Compositing:** Recreates BeReal's authentic in-app look with smooth rounded corners (60px radius) and crisp black borders (7px) overlaid at `(55, 55)`, with optional Side-by-Side and Reversed (secondary as background) layouts.
- **Reverse-Angled Dual Camera Export:** 3-way perspective selector lets you export **Standard Only**, **Reversed Only** (selfie as full-bleed canvas with landscape lens inset), or **Both Angles** concurrently.
- **Samsung & Google Motion Photos:** Muxes BTS (Behind-the-Scenes) video into standard JPEG containers via pure Rust binary tag generation (`SEFH`/`SEFT`) and GCamera XMP injection.
- **Format Flexibility:** Convert to JPEG (with quality control 50–100%), preserve WebP, or export to lossless PNG.
- **Visual Date Range Filter:** Interactive monthly density curve histogram with range sliders to select and batch-process specific periods.
- **Multi-Core Rayon Concurrency:** Parallel image conversion and compositing utilizing all available CPU threads for 5–10× throughput gains.

### 🎬 Recap Video Generator
- **Zero-Copy Frame Streaming:** Renders frames directly to raw RGB buffers piped directly into FFmpeg `stdin`, keeping peak memory usage at ~30MB even for large 4K video exports.
- **Symphonia Audio Decoding & Waveform Visualization:** Pure Rust multi-format audio decoding (MP3, WAV, M4A, AAC, FLAC, OGG) with real-time 100+ bin peak amplitude waveform visualization.
- **Dynamic Timing Curves:** Quadratic ramp (slower opening and finale, dynamic middle), even pacing, accelerate, decelerate, or rhythmic wave pacing.
- **Offline & Online Geocoding Engine:** Resolves coordinates using Nominatim API or 3 downloadable offline GeoNames tiers (Lite 25k cities, Standard 55k towns, Ultra Detailed 140k+ villages) with configurable country formatting rules.
- **Custom Visual Overlays:** System font enumeration, customizable date stamps, text shadow effects, and multiple resolution presets (9:16 vertical, 1080p, 4K).

### ⚡ Architecture & Efficiency
- **Streaming ZIP Parser:** Extracts and parses massive multi-gigabyte BeReal GDPR archives with buffered streams, scanning 120+ posts in under 20ms.
- **Multi-Job Parallel Queue:** Run photo batch exports and video renders simultaneously in the background without UI blocking.
- **Native File Explorer Integration:** Cross-platform `show_in_folder` command accurately reveals output files and folders in Windows Explorer, macOS Finder, or Linux file managers.
- **Developer Demo Mode:** Press `Ctrl+Shift+D` or toggle **Dev Mode** in development to instantly hydrate mock GDPR archives, parallel jobs, waveforms, and activity history for rapid UI testing.

---

## 📊 Performance Benchmarks

Automated end-to-end integration benchmarks measured with synthetic BeReal GDPR archives (`cargo test --test benchmark_suite`):

| Operation | Scale / Dataset | Measured Throughput | Peak RAM |
|:---|:---|:---|:---|
| **Archive Scan & JSON Parse** | 50 Posts | `17.13 ms` ⚡ | < 10 MB |
| **Large Archive Scan & Histogram** | 120 Posts (Full Year) | `19.97 ms` ⚡ | < 12 MB |
| **Side-by-Side Compositing** | 1080p Dual Lenses | `139.94 ms / image` | ~25 MB |
| **Picture-in-Picture Compositing** | 1080p + 60px Radius Lens | `359.21 ms / image` | ~30 MB |
| **Zero-Copy Recap Video Stream** | 1080x1920 @ 30 FPS (H.264) | `~45 FPS direct pipe` | **~30 MB (Flat)** |

---

## 📋 How to Download Your Archive from BeReal
 
1. Open the **BeReal** mobile app and tap your **Profile icon** (top-right).
2. Tap **Help** &rarr; Select **Contact Us**.
3. Select **Ask a Question** &rarr; Tap **Troubleshooting** &rarr; Tap **Other**.
4. Tap **Contact Us** at the bottom &rarr; Select the **Topic** dropdown.
5. Select **"I'd like to request a copy of my data"**.
6. Type a message with at least **10 characters** (e.g., *"Please provide a copy of my account data"*) and submit.
7. BeReal will deliver a secure download link via email containing your official archive ZIP (including `posts.json` and all media).
8. Once downloaded, select the ZIP or unzipped folder directly in **BeReal Studio**.

---

## 🛠️ Prerequisites

1. **Rust Toolchain:**
   - Install via [rustup.rs](https://rustup.rs) (Rust 1.78+ recommended).
2. **Package Manager & JavaScript Runtime:**
   - **Bun (Recommended for ultra-fast startup):** Install via [bun.sh](https://bun.sh) (`powershell -c "irm bun.sh/install.ps1 | iex"`).
   - **NPM / PNPM / Yarn (Fully Supported):** Standard Node.js v18+ environment works out-of-the-box.
3. **FFmpeg (For Recap Video & Motion Photos):**
   - Required for video slideshow encoding and dual-video PIP overlays.
   - **Windows:** `winget install Gyan.FFmpeg` or download from [ffmpeg.org](https://ffmpeg.org/download.html).
   - **macOS:** `brew install ffmpeg`
   - **Linux:** `sudo apt install ffmpeg`

---

## 🚀 Running Locally

You can use **Bun** (recommended) or **NPM / PNPM / Yarn**:

### Option A: Using Bun (Fastest)
```bash
# Install dependencies
bun install

# Start local desktop development server
bun run tauri dev
```

### Option B: Using NPM
```bash
# Install dependencies
npm install

# Start local desktop development server
npm run tauri dev
```

---

## 📦 Building & Packaging

To compile a self-contained release executable and installer for your operating system:

```bash
# With Bun
bun run tauri build

# With NPM
npm run tauri build
```

### Build Artifact Locations:
- **Windows:** `src-tauri/target/release/bundle/msi/BeReal Studio_1.3.1_x64_en-US.msi`
- **macOS:** `src-tauri/target/release/bundle/dmg/BeReal Studio_1.3.1_x64.dmg`
- **Linux:** `src-tauri/target/release/bundle/deb/bereal-studio_1.3.1_amd64.deb` or `appimage`

---

## 🧪 Testing & Verification

```bash
# Run Svelte & TypeScript type checks
bun run check
# or: npm run check

# Run Rust unit tests and benchmark suite
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 🏗️ Project Architecture

```
BeRealStudio/
├── src/                                    # Frontend (Svelte 5 + SvelteKit SPA + TypeScript)
│   ├── app.html                            # Root HTML & Inter typography
│   ├── styles/global.css                   # Custom dark design system tokens
│   ├── lib/
│   │   ├── types.ts                        # TypeScript data models & IPC interfaces
│   │   ├── tauri.ts                        # Typed Tauri IPC bridge & event subscribers
│   │   ├── stores.ts                       # Reactive stores & parallel multi-job queue
│   │   ├── devMode.ts                      # Demo data generator & developer mode
│   │   └── fonts.ts                        # Curated built-in font definitions
│   ├── components/                         # Reusable UI Components
│   │   ├── NavBar.svelte                   # Global top navigation with Activity badge
│   │   ├── Toggle.svelte                   # Animated on/off switch
│   │   ├── Slider.svelte                   # Value range slider with value pill
│   │   ├── Stepper.svelte                  # Numeric stepper input
│   │   ├── FilePicker.svelte               # Native folder & file dialog wrapper
│   │   ├── DateRangePicker.svelte          # Dual date pickers with monthly density curve
│   │   ├── SpeedCurvePreview.svelte        # Visual speed curve & audio waveform canvas
│   │   ├── Modal.svelte                    # Accessible modal dialog
│   │   ├── LogConsole.svelte               # Color-coded live terminal console
│   │   ├── ErrorModal.svelte               # Categorized error overlay
│   │   └── RuleEditor.svelte               # Reverse geocoding rules editor
│   ├── views/                              # Application Primary Views
│   │   ├── Home.svelte                     # Main dashboard with hero & feature cards
│   │   ├── ToolkitConfig.svelte            # Photo processing configuration view
│   │   ├── RecapperConfig.svelte           # Recap video configuration view with 9:16 preview
│   │   ├── Activity.svelte                 # Parallel active operations & generation history
│   │   ├── Processing.svelte               # Real-time progress & live streaming log view
│   │   ├── Complete.svelte                 # Summary metrics, output opener & log exporter
│   │   ├── Settings.svelte                 # Global defaults, FFmpeg detection & offline GeoDB
│   │   └── About.svelte                    # Privacy manifesto, authoring & open source credits
│   └── routes/+page.svelte                 # SPA root page router
│
├── src-tauri/                              # Rust Backend (Tauri v2)
│   ├── Cargo.toml                          # Native dependencies (image, img-parts, symphonia, rayon, etc.)
│   ├── tauri.conf.json                     # Desktop window & plugin configuration
│   ├── capabilities/default.json           # Tauri v2 security capabilities
│   ├── tests/
│   │   └── benchmark_suite.rs              # End-to-end performance benchmarking harness
│   └── src/
│       ├── main.rs & lib.rs                # Tauri entry & command registration
│       ├── state.rs                        # Global state, ProgressEmitter & log buffer
│       ├── commands/                       # IPC Command Handlers
│       │   ├── archive.rs                  # scan_archive, extract_zip (streaming)
│       │   ├── toolkit.rs                  # start_toolkit, cancel_toolkit (Rayon multi-core)
│       │   ├── recapper.rs                 # start_recapper, cancel_recapper
│       │   ├── settings.rs                 # load_settings, save_settings, reset_settings
│       │   ├── system.rs                   # show_in_folder, check_ffmpeg, offline geodb, analyze_audio
│       │   └── debug.rs                    # export_debug_log, get_debug_logs
│       ├── pipeline/                       # Photo Processing Logic
│       │   ├── parser.rs                   # Streaming JSON parsing & monthly histogram
│       │   ├── image_ops.rs                # Format conversion, PIP & Side-by-Side compositing
│       │   ├── exif_writer.rs              # Lossless EXIF & IPTC JPEG segment injection
│       │   ├── motion_photo.rs             # Samsung SEFH/SEFT binary muxer & GCamera XMP
│       │   ├── video_ops.rs                # FFmpeg dual-video PIP overlay
│       │   ├── date_filter.rs              # Range filtering & density distribution
│       │   └── cleanup.rs                  # Intermediate artifact cleanup
│       └── recapper/                       # Recap Video Engine
│           ├── audio.rs                    # Symphonia audio decoding & waveform analysis
│           ├── timing.rs                   # Quadratic ramp / even timing curves
│           ├── geocoder.rs                 # Nominatim reverse geocoding & offline GeoDB
│           ├── location_rules.rs           # Country-specific location formatting engine
│           ├── font_resolver.rs            # Built-in font resolver & disk loader
│           ├── frame_renderer.rs           # Image resize & text overlay with shadows
│           └── video_encoder.rs            # Zero-copy raw RGB frame piping to FFmpeg stdin
│
└── package.json                            # App manifest & dependencies (v1.4.0)
```

---

## 💖 Credits & Open Source Lineage

**BeReal Studio** is authored and maintained by **[NotToxel](https://github.com/NotToxel)** ([GitHub Repository](https://github.com/NotToxel/BeRealStudio)).

It unifies, rewrites, and modernizes the core capabilities of two pioneer open-source projects into a single, high-performance desktop application:

- **[BeReel](https://github.com/theOneAndOnlyOne/BeReel)** *(by [@theOneAndOnlyOne](https://github.com/theOneAndOnlyOne))* — Creator of the music-synchronized BeReal recap video generator and reverse-geocoding rules engine.
- **[BeReal-GDPR-Photo-Toolkit](https://github.com/hatobi/bereal-gdpr-photo-toolkit)** *(by [@hatobi](https://github.com/hatobi))* — Pioneer of BeReal GDPR archive extraction, EXIF metadata restoration, and Picture-in-Picture photo compositing.

---

## 📜 License

MIT License &copy; 2026 NotToxel and BeReal Studio Contributors. Free and open-source.
