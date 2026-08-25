# BeReal Studio 📸 🎬

**BeReal Studio** is a unified, high-performance, cross-platform desktop application built with **Tauri v2 (Rust)** and **Svelte** to import, process, fix metadata on, composite, and recap your personal BeReal GDPR data exports.

---

## Key Features

### 📸 Photo Processing Suite
- **Metadata Restoration & EXIF Synchronization:** Automatically reads `posts.json` timestamps and GPS coordinates, embedding `DateTimeOriginal`, `GPSLatitude`, `GPSLongitude`, and caption descriptions into EXIF/IPTC image headers without lossy re-encoding.
- **Picture-in-Picture & Dual-Camera Compositing:** Recreates BeReal's authentic in-app look with rounded corners (60px radius) and crisp black borders (7px) overlaid at `(55, 55)`, with optional Side-by-Side and Reversed (secondary as background) layouts.
- **Samsung / Google Motion Photos:** Muxes BTS (Behind-the-Scenes) video into standard JPEG containers via pure Rust binary tag generation (`SEFH`/`SEFT`) and GCamera XMP injection.
- **Format Flexibility:** Convert to JPEG (with quality control 50–100%), preserve WebP, or export to lossless PNG.
- **Date Range Filtering:** Visual monthly density histogram with quick presets (Last 30 Days, 6 Months, 1 Year, All Time) to filter specific batches.
- **Parallel Processing:** Multi-threaded throughput powered by **Rayon**, delivering 5–10× speedups over sequential scripts.

### 🎬 BeReal Recapper Video Generator
- **Music Synchronization:** Decodes audio tracks (MP3, WAV, M4A, FLAC, OGG) using **Symphonia** to accurately pace image transitions.
- **Speed Transitions:** Quadratic speed curve ramp (slower start and finale, fast middle) or even pacing with configurable start/end padding.
- **Reverse Geocoding & Rules Engine:** Resolves GPS coordinates to city, neighborhood, or state via OpenStreetMap's Nominatim API (with in-memory cache) or optional offline datasets, with a configurable country-by-country formatting engine.
- **Custom Visual Overlays:** System font enumeration, customizable date stamps, text shadow effects, and multiple resolution presets (9:16 vertical, 1080p, 4K).
- **Direct FFmpeg Pipe:** Pure Rust frame rendering streamed directly into FFmpeg's `stdin` for fast H.264/AAC MP4 video generation.

---

## 📋 How to Request Your Data from BeReal

1. Open the **BeReal** mobile app on your phone.
2. Go to **Profile** &rarr; Tap the **Three Dots (...)** in the top-right corner to open **Settings**.
3. Tap **Privacy** &rarr; Tap **Request My Data** (Article 15 GDPR Data Portability).
4. Enter your email address and submit.
5. Within 24–48 hours, BeReal will email you a secure download link for your archive (`.zip`).
6. Download the archive and either select the unzipped folder or upload the zip directly into **BeReal Studio**.

---

## 🛠️ Prerequisites
 
1. **Rust Toolchain:**
   - Install via [rustup.rs](https://rustup.rs) (Rust 1.78+ recommended).
2. **Bun (Fast All-in-One JavaScript Runtime & Package Manager):**
   - Install via `powershell -c "irm bun.sh/install.ps1 | iex"` or [bun.sh](https://bun.sh).
3. **FFmpeg (For Video Features):**
   - Required for video PIP combining and Recap slideshow rendering.
   - **Windows:** `winget install Gyan.FFmpeg` or download from [ffmpeg.org](https://ffmpeg.org/download.html).
   - **macOS:** `brew install ffmpeg`
   - **Linux:** `sudo apt install ffmpeg`

---

## 🚀 Running Locally

```bash
# Navigate to project directory
cd bereal-studio

# Install dependencies
npm install

# Run frontend + Tauri desktop app in dev mode
npm run tauri dev
```

---

## 📦 Building & Packaging

To compile a self-contained release executable and installer for your current operating system:

```bash
# Build production bundle
npm run tauri build
```

### Build Artifact Locations:
- **Windows:** `src-tauri/target/release/bundle/msi/BeReal Studio_0.1.0_x64_en-US.msi`
- **macOS:** `src-tauri/target/release/bundle/dmg/BeReal Studio_0.1.0_x64.dmg`
- **Linux:** `src-tauri/target/release/bundle/deb/bereal-studio_0.1.0_amd64.deb` or `appimage`

---

## 🏗️ Project Architecture

```
bereal-studio/
├── src-tauri/                              # Rust Backend (Tauri v2)
│   ├── Cargo.toml                          # Native dependencies (image, img-parts, symphonia, rayon, etc.)
│   ├── tauri.conf.json                     # Tauri configuration & capabilities
│   └── src/
│       ├── main.rs                         # Desktop binary entry
│       ├── lib.rs                          # Tauri builder & IPC command registration
│       ├── state.rs                        # Global app state & live ProgressEmitter
│       ├── commands/                       # IPC Command Handlers
│       │   ├── archive.rs                  # scan_archive, extract_zip
│       │   ├── toolkit.rs                  # start_toolkit, cancel_toolkit
│       │   ├── recapper.rs                 # start_recapper, cancel_recapper
│       │   ├── settings.rs                 # load_settings, save_settings
│       │   ├── system.rs                   # check_ffmpeg, list_system_fonts
│       │   └── debug.rs                    # export_debug_log, get_debug_logs
│       ├── pipeline/                       # Photo Processing Engine
│       │   ├── parser.rs                   # posts.json parsing & histogram computation
│       │   ├── image_ops.rs                # WebP/JPEG/PNG conversion & PIP/SbS compositing
│       │   ├── exif_writer.rs              # Zero-copy EXIF & IPTC JPEG segment writing
│       │   ├── motion_photo.rs             # Samsung SEFH/SEFT binary muxer & XMP injection
│       │   ├── video_ops.rs                # FFmpeg dual-video PIP overlay
│       │   ├── date_filter.rs              # Range filtering & density distribution
│       │   └── cleanup.rs                  # Intermediate artifact cleanup
│       └── recapper/                       # Recap Video Engine
│           ├── audio.rs                    # Symphonia audio decoding & duration
│           ├── timing.rs                   # Quadratic ramp / even timing curves
│           ├── geocoder.rs                 # Nominatim reverse geocoding with cache
│           ├── location_rules.rs           # Country-specific location formatting engine
│           ├── font_resolver.rs            # Cross-platform system font enumeration
│           ├── frame_renderer.rs           # Image resize & text overlay with shadows
│           └── video_encoder.rs            # Raw RGB frame piping to FFmpeg stdin
│
└── src/                                    # Frontend (SvelteKit SPA + TypeScript)
    ├── app.html                            # Root HTML & Inter font loading
    ├── styles/global.css                   # Custom dark design system
    ├── lib/
    │   ├── types.ts                        # TypeScript interfaces
    │   ├── tauri.ts                        # Tauri IPC bridge & typed event listeners
    │   └── stores.ts                       # Svelte reactive stores
    ├── components/
    │   ├── Toggle.svelte                   # Clean on/off switch
    │   ├── Slider.svelte                   # Value range slider
    │   ├── FilePicker.svelte               # Native folder & file dialog wrapper
    │   ├── DateRangePicker.svelte          # Dual date pickers with monthly density histogram
    │   ├── ProgressBar.svelte              # Streaming progress indicator
    │   ├── LogConsole.svelte               # Color-coded live terminal log
    │   ├── ErrorModal.svelte               # Categorized error overlay
    │   ├── FontPicker.svelte               # System font selector
    │   └── RuleEditor.svelte               # Geocoded location rules editor
    ├── views/
    │   ├── Home.svelte                     # Main dashboard with dual feature cards
    │   ├── ToolkitConfig.svelte            # Photo processing configuration view
    │   ├── RecapperConfig.svelte           # Recap video configuration view with live preview
    │   ├── Processing.svelte               # Real-time progress & live streaming log view
    │   ├── Complete.svelte                 # Summary metrics, output opener & log exporter
    │   ├── Settings.svelte                 # Global defaults, FFmpeg detection & log clearing
    │   └── About.svelte                    # Privacy manifesto, GDPR guide & open-source credits
    └── routes/+page.svelte                 # Router & persistent settings manager
```

---

## 💖 Credits & Open Source Lineage

BeReal Studio unifies and modernizes the core capabilities of two popular open-source projects into a single, high-performance desktop application:

- **[BeReel](https://github.com/theOneAndOnlyOne/BeReel)** *(by [@theOneAndOnlyOne](https://github.com/theOneAndOnlyOne))* — Creator of the music-synchronized BeReal recap video generator and reverse-geocoding rules engine.
- **[BeReal-GDPR-Photo-Toolkit](https://github.com/hatobi/bereal-gdpr-photo-toolkit)** *(by [@hatobi](https://github.com/hatobi))* — Pioneer of BeReal GDPR archive extraction, EXIF metadata restoration, and Picture-in-Picture photo compositing.

---

## 📜 License

MIT License &copy; 2026 BeReal Studio Contributors. Free and open-source.
