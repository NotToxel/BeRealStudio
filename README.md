# BeReal Studio 📸 🎬

<div align="center">
  <h3>Unified, Local-First Desktop Suite for BeReal GDPR Data Exports</h3>
  <p>Explore your memories in an authentic mobile feed & calendar, restore metadata, composite dual-camera memories, mux motion photos, and generate music-synchronized recap videos — 100% locally and privately.</p>
  <p>
    <a href="https://github.com/NotToxel/BeRealStudio/releases/latest"><img src="https://img.shields.io/github/v/release/NotToxel/BeRealStudio?label=Latest%20Release&logo=github&color=blue" alt="Latest Release" /></a>
    <a href="https://github.com/NotToxel/BeRealStudio/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/NotToxel/BeRealStudio/release.yml?branch=master&label=Release%20Build&logo=github" alt="Release Build Status" /></a>
    <a href="https://github.com/NotToxel/BeRealStudio/blob/master/LICENSE"><img src="https://img.shields.io/badge/License-MIT-green?style=flat" alt="License" /></a>
    <img src="https://img.shields.io/badge/Platform-Windows%20%7C%20macOS%20%7C%20Linux-purple?style=flat" alt="Cross Platform" />
  </p>
</div>

---

## 📥 Downloads & Installation

Download the official standalone release for your platform from the [Latest Release Page](https://github.com/NotToxel/BeRealStudio/releases/latest):

| Platform | Format | Direct Download Link |
| :--- | :--- | :--- |
| **Windows** (x64) | `.exe` (Installer) | [⬇️ **Download for Windows (Installer)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.2.1_x64-setup.exe) |
| **Windows** (x64) | `.msi` (Package) | [⬇️ **Download for Windows (.msi)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.2.1_x64_en-US.msi) |
| **macOS** (Apple Silicon) | `.dmg` (M1/M2/M3/M4) | [⬇️ **Download for macOS (Apple Silicon)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.2.1_aarch64.dmg) |
| **macOS** (Intel) | `.dmg` (x86_64) | [⬇️ **Download for macOS (Intel)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.2.1_x64.dmg) |
| **Linux** (x64) | `.AppImage` (Universal) | [⬇️ **Download for Linux (.AppImage)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/bereal-studio_2.2.1_amd64.AppImage) |
| **Linux** (x64) | `.deb` (Debian / Ubuntu) | [⬇️ **Download for Linux (.deb)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/bereal-studio_2.2.1_amd64.deb) |

> 💡 *Looking for earlier releases, source archives, or release notes? Explore all [GitHub Releases](https://github.com/NotToxel/BeRealStudio/releases).*

---

## 🖼️ Application Showcase

<div align="center">

| 🏠 Home Dashboard & Archive Scanner | 📱 Native Memories & Calendar Explorer |
|:---:|:---:|
| ![Home Dashboard](docs/screenshots/01_home_dashboard.png) | ![Memories Explorer](docs/screenshots/02_memories_explorer.png) |

| 📸 Photo Processing Suite & Dual Perspectives | 🎬 Recap Video Generator & Audio Waveforms |
|:---:|:---:|
| ![Photo Processing](docs/screenshots/03_photo_toolkit_config.png) | ![Recap Video Generator](docs/screenshots/04_recap_video_config.png) |

| ⚡ Active Operations & Generation History Queue |
|:---:|
| ![Activity History](docs/screenshots/05_activity_history.png) |

</div>

---

## ✨ Key Features

### 📱 1. Native Memories & Calendar Explorer
- **Authentic BeReal Experience**: Mobile-identical dark aesthetic designed to browse your entire GDPR archive seamlessly.
- **Dual View Modes**: Switch effortlessly between a responsive **Memories Card Grid** (with viewport-filling timeline scrubber) and an interactive **Monthly Calendar Matrix** (with pinned sticky navigation).
- **Continuous Vertical Infinite Feed**: Tap any post to open a smooth, continuous vertical feed with instant auto-scroll to the selected memory.
- **Dynamic Sticky Header**: Tracks active post date and position (e.g. `18 August 2024 • 14 of 420`) as you scroll.
- **Interactive Dual-Camera Frame**:
  - **Click-to-Swap**: Flip front and back cameras instantly.
  - **Movable PIP**: Drag and reposition the selfie PIP anywhere or snap to the 4 corners.
  - **Inline BTS Player**: Stream Behind-the-Scenes live video micro-clips with a single click.
- **Smart Search & Live Compound Filtering**:
  - Filter posts by text query, GPS location, BTS clips, captions, retakes, year, month, city, and country.
  - Live dynamic count tags on all filter chips and dimension selectors update continuously as multi-level filters are applied.
- **Single-Memory Instant Export Dialog**:
  - **Picture-in-Picture & Side-by-Side**: High-resolution dual-camera composites with lossless EXIF restoration.
  - **Apple Live Photo (.jpg + .mov pair)**: Generates matching Apple Content Identifier UUID metadata (`MakerApple:17` and `com.apple.quicktime.content.identifier`) recognized natively by macOS/iOS Apple Photos and iCloud.
  - **Samsung & Google Motion Photos**: Muxes Behind-the-Scenes (BTS) videos directly into JPEG headers via Samsung SEFH binary trailers and Google MicroVideo XMP.
  - **Raw Media Clips & Camera Isolations**: Export primary camera, selfie camera, or raw MP4 video clips independently.
- **Configurable Header Display**: Customize location formatting (City/Country, Suburb, Full) and timestamp/late tag display in Settings.

---

### 📸 2. Photo Processing Suite
- **Metadata Restoration & EXIF Synchronization**: Losslessly embeds original capture dates, times, GPS coordinates, and caption descriptions into EXIF/IPTC photo headers.
- **BeReal Moment Registry & True Cycle Date Anchoring**: Accurately anchors posts taken late or past midnight to their true BeReal notification cycle dates.
- **Dual-Camera Compositing**: Recreates BeReal's signature aesthetic with rounded corners and clean borders in Picture-in-Picture and Side-by-Side layouts.
- **Dual-Angle Perspective Export**: Choose between **Standard** (primary background), **Reversed** (selfie background), or export **Both Angles** concurrently.
- **Samsung & Google Motion Photos**: Muxes Behind-the-Scenes (BTS) videos into motion photos compatible with Samsung Gallery and Google Photos *(requires JPEG format)*.
- **Visual Timeline & Date Range Filter**: Interactive monthly activity density curve and calendar picker to easily filter memories by year, month, or custom dates.
- **Fast Batch Processing**: Multi-threaded Rayon pipeline processes hundreds of archive photos in seconds.

---

### 🎬 3. Recap Video Generator
- **Music-Synchronized Recap Slideshows**: Automatically paces memories to your chosen soundtrack (MP3, WAV, M4A, AAC, FLAC) with real-time waveform visualization.
- **Dynamic Timing Curves**: Customize video pacing with quadratic ramp, even timing, accelerate, decelerate, or wave timing curves.
- **Smart Location & Date Stamps**: Formats reverse-geocoded location stamps and custom date typography on each memory slide.
- **Offline Spatial Reverse Geocoder**: Built-in in-memory GeoNames spatial index for instant, offline city/country resolution with zero network pauses.
- **Live Video Preview**: Interactive player to preview your recap sequence before rendering.
- **Background Multi-Job Queue**: Render videos and process photo batches simultaneously in the background without UI interruption.

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

## 🛠️ Prerequisites (For Building from Source)

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
- **Windows:** `src-tauri/target/release/bundle/msi/BeReal Studio_2.2.1_x64_en-US.msi` or `.exe`
- **macOS:** `src-tauri/target/release/bundle/dmg/BeReal Studio_2.2.1_universal.dmg`
- **Linux:** `src-tauri/target/release/bundle/deb/bereal-studio_2.2.1_amd64.deb` or `appimage`

---

## 🧪 Testing & Verification

```bash
# Run Svelte & TypeScript diagnostics
bun run check
# or: npm run check

# Run Rust unit tests and benchmark suite
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## 🏗️ Architecture & Directory Structure

```
BeRealStudio/
├── src/                                    # Frontend (SvelteKit SPA + TypeScript)
│   ├── app.html                            # Root HTML & Inter font imports
│   ├── styles/global.css                   # Custom dark design system tokens & font-faces
│   ├── lib/
│   │   ├── types.ts                        # TypeScript models & IPC interfaces
│   │   ├── tauri.ts                        # Tauri IPC bridge & typed event listeners
│   │   ├── stores.ts                       # Svelte reactive state stores
│   │   ├── memoriesStore.ts                # Memories explorer state & live compound filtering
│   │   └── fonts.ts                        # Curated built-in font definitions
│   ├── components/                         # Reusable UI Component Suite
│   │   ├── NavBar.svelte                   # Top navigation bar (Home, Photos, Recap, Settings, About)
│   │   ├── Toggle.svelte                   # Animated on/off switch
│   │   ├── Slider.svelte                   # Value range slider with value pill
│   │   ├── FilePicker.svelte               # Native folder & file dialog wrapper
│   │   ├── DateRangePicker.svelte          # Dual date pickers with monthly density histogram
│   │   ├── ProgressBar.svelte              # Streaming progress indicator
│   │   ├── LogConsole.svelte               # Color-coded live terminal log
│   │   ├── ErrorModal.svelte               # Categorized error overlay
│   │   ├── FontPicker.svelte               # Curated 7-font dropdown selector
│   │   ├── RuleEditor.svelte               # Reverse geocoding rules editor
│   │   └── memories/                       # Memories & Explorer Component Suite
│   │       ├── MemoriesGrid.svelte         # Responsive memory card grid & full-height scrubber
│   │       ├── CalendarGrid.svelte         # Interactive monthly calendar with sticky navigation
│   │       ├── DualCameraFrame.svelte      # Dual-camera frame with click-to-swap & movable PIP
│   │       ├── MemoryFeedModal.svelte      # Fullscreen continuous vertical scroll feed
│   │       ├── MemoryFilterBar.svelte      # Live compound search & dimension filters
│   │       ├── MemoryActionMenu.svelte     # Memory 3-dot dropdown menu
│   │       ├── MemoryContextMenu.svelte    # Right-click context menu
│   │       └── ExportMemoryModal.svelte    # Single-memory export dialog (Apple Live Photo & Motion Photo)
│   ├── views/                              # Application Primary Views
│   │   ├── Home.svelte                     # Main dashboard with hero & feature cards
│   │   ├── MemoriesView.svelte             # Native Memories & Calendar Explorer View
│   │   ├── ToolkitConfig.svelte            # Photo processing configuration view
│   │   ├── RecapperConfig.svelte           # Recap video configuration view with live preview
│   │   ├── Activity.svelte                 # Parallel active operations & generation history
│   │   ├── Processing.svelte               # Real-time progress & live streaming log view
│   │   ├── Complete.svelte                 # Summary metrics, output opener & log exporter
│   │   ├── Settings.svelte                 # Global defaults, FFmpeg detection & inspector tools
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
│       │   ├── explorer.rs                 # load_memories, export_single_memory (Live Photo & Motion Photo)
│       │   ├── settings.rs                 # load_settings, save_settings, reset_settings
│       │   ├── system.rs                   # show_in_folder, check_ffmpeg, offline geodb, analyze_audio
│       │   └── debug.rs                    # export_debug_log, get_debug_logs
│       ├── pipeline/                       # Photo Processing Logic
│       │   ├── parser.rs                   # Authoritative dataset fusion & moment registry
│       │   ├── image_ops.rs                # Format conversion, PIP & Side-by-Side compositing
│       │   ├── exif_writer.rs              # Lossless EXIF & IPTC JPEG segment injection
│       │   ├── live_photo.rs               # Apple Photos Live Photo pair (.jpg + .mov) generator
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
├── package.json                            # App manifest & dependencies (v2.2.1)
└── README.md                               # User documentation & GDPR guide
```

---

## 🗺️ Future Roadmap & Upcoming Features

- [x] **📅 Native BeReal-Style Memories & Calendar Viewer** *(Completed in v2.0.0)*:
  - Monthly Memories Calendar Matrix with day thumbnails, late badges, and retake counters.
  - Interactive Feed & Lightbox with front/back camera click-to-swap and movable PIP.
  - Multi-dimensional search, hierarchical location drawers, and single-memory exports.
- [x] **🍏 Apple Photos Live Photos Compatibility** *(Completed in v2.2.1)*:
  - Export paired still image (`.jpg`) and video (`.mov`) files with matching Apple Content Identifier UUID (`MakerApple:17` and `com.apple.quicktime.content.identifier`) for native drag-and-drop Live Photo recognition in Apple Photos and iCloud.
- [ ] **🏷️ Direct Caption Burn-In on Exported Photos**:
  - Optional setting to burn original BeReal captions in authentic semi-transparent rounded pill styling directly onto composited images or recap slides.
- [ ] **🎬 Recap Video Library & Gallery Viewer**:
  - In-app gallery indexing all rendered recap MP4s with playback preview, waveform scrubber, and quick actions ("Open in Player", "Show in Explorer").

---

## 💖 Credits & Open Source Lineage

**BeReal Studio** is authored and maintained by **[NotToxel](https://github.com/NotToxel)** ([GitHub Repository](https://github.com/NotToxel/BeRealStudio)).

It unifies, rewrites, and modernizes the core capabilities of three pioneer open-source projects into a single, high-performance desktop application:

- **[BeReel](https://github.com/theOneAndOnlyOne/BeReel)** *(by [@theOneAndOnlyOne](https://github.com/theOneAndOnlyOne))* — Creator of the music-synchronized BeReal recap video generator and reverse-geocoding rules engine.
- **[BeReal-GDPR-Photo-Toolkit](https://github.com/hatobi/bereal-gdpr-photo-toolkit)** *(by [@hatobi](https://github.com/hatobi))* — Pioneer of BeReal GDPR archive extraction, EXIF metadata restoration, and Picture-in-Picture photo compositing.
- **[makelive](https://github.com/mifi/makelive)** *(by [@mifi](https://github.com/mifi))* — Pioneer of Apple Photos compatible Live Photo generation with matching Content Identifier metadata synchronization on paired image and video assets.

---

## 📜 License

MIT License &copy; 2026 NotToxel and BeReal Studio Contributors. Free and open-source.
