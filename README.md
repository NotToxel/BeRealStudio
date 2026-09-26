# BeReal Studio 📸 🎬

<div align="center">
  <h3>Unified, Local-First Desktop Suite for BeReal GDPR Data Exports</h3>
  <p>Explore your memories in a feed, calendar, and map; restore and export photos; and create music-synchronized recap videos on your own computer.</p>
  <p>
    <a href="https://github.com/NotToxel/BeRealStudio/releases/latest"><img src="https://img.shields.io/github/v/release/NotToxel/BeRealStudio?label=Latest%20Release&logo=github&color=blue" alt="Latest Release" /></a>
    <a href="https://github.com/NotToxel/BeRealStudio/actions/workflows/release.yml"><img src="https://img.shields.io/github/actions/workflow/status/NotToxel/BeRealStudio/release.yml?label=Release%20Build&logo=github" alt="Release Build Status" /></a>
    <a href="https://github.com/NotToxel/BeRealStudio/blob/master/LICENSE"><img src="https://img.shields.io/badge/License-GPL--3.0--or--later-blue?style=flat&logo=gnu" alt="License" /></a>
    <img src="https://img.shields.io/badge/Windows-0078D6?style=flat&logo=data:image/svg%2bxml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNTYgMjU2Ij48cGF0aCBmaWxsPSIjZmZmZmZmIiBkPSJNMCAwaDEyMS4zMjl2MTIxLjMyOUgwem0xMzQuNjcxIDBIMjU2djEyMS4zMjlIMTM0LjY3MXpNMCAxMzQuNjcxaDEyMS4zMjlWMjU2SDB6bTEzNC42NzEgMEgyNTZWMjU2SDEzNC42NzF6Ii8+PC9zdmc+" alt="Windows" />
    <img src="https://img.shields.io/badge/macOS-000000?style=flat&logo=apple&logoColor=white" alt="macOS" />
    <img src="https://img.shields.io/badge/Linux-FCC624?style=flat&logo=linux&logoColor=black" alt="Linux" />
    <img src="https://img.shields.io/badge/Privacy-100%25%20Local-emerald?style=flat" alt="Privacy" />
  </p>
</div>

---

## 📥 Downloads & Installation

Download the official standalone release for your platform from the [Latest Release Page](https://github.com/NotToxel/BeRealStudio/releases/latest):

| Platform | Format / Architecture | Direct Download Link |
| :--- | :--- | :--- |
| <img src="docs/icons/windows.svg" height="15" width="15" alt="Windows" style="vertical-align: middle;" /> **Windows** | `.exe` (64-bit NSIS Setup) | [⬇️ **Download for Windows (Installer)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.6.0_x64-setup.exe) |
| <img src="docs/icons/windows.svg" height="15" width="15" alt="Windows" style="vertical-align: middle;" /> **Windows** | `.exe` (64-bit Standalone) | [⬇️ **Download for Windows (Portable)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/bereal-studio.exe) |
| <img src="docs/icons/apple.svg" height="15" width="15" alt="macOS" style="vertical-align: middle;" /> **macOS** | `.dmg` (Apple Silicon M1/M2/M3/M4) | [⬇️ **Download for macOS (Apple Silicon .dmg)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.6.0_aarch64.dmg) |
| <img src="docs/icons/apple.svg" height="15" width="15" alt="macOS" style="vertical-align: middle;" /> **macOS** | `.app.tar.gz` (Apple Silicon .app Bundle) | [⬇️ **Download for macOS (.app Bundle)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal-Studio-macOS.app.tar.gz) |
| <img src="docs/icons/linux.svg" height="15" width="15" alt="Linux" style="vertical-align: middle;" /> **Linux** | `.AppImage` (x86_64 Universal) | [⬇️ **Download for Linux (.AppImage)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.6.0_amd64.AppImage) |
| <img src="docs/icons/linux.svg" height="15" width="15" alt="Linux" style="vertical-align: middle;" /> **Linux** | `.deb` (Debian / Ubuntu x86_64) | [⬇️ **Download for Linux (.deb)**](https://github.com/NotToxel/BeRealStudio/releases/latest/download/BeReal.Studio_2.6.0_amd64.deb) |

> 💡 *Looking for earlier releases, source archives, or release notes? Explore all [GitHub Releases](https://github.com/NotToxel/BeRealStudio/releases).*

Read the [plain-language changelog](CHANGELOG.md) or the [full v2.6.0 release notes](docs/releases/v2.6.0.md).

---

## ⚡ Prerequisites & External Tools

BeReal Studio is engineered with native Rust algorithms to minimize dependencies. Depending on the features you use:

| Tool | Status | Purpose | Installation |
| :--- | :--- | :--- | :--- |
| **FFmpeg** | **Required for Videos** | Rendering recap videos (`.mp4`), audio sync, and video PIP compositing. | • <img src="docs/icons/windows.svg" height="13" width="13" alt="Windows" style="vertical-align: middle;" /> **Windows**: `winget install Gyan.FFmpeg`<br>• <img src="docs/icons/apple.svg" height="13" width="13" alt="macOS" style="vertical-align: middle;" /> **macOS**: `brew install ffmpeg`<br>• <img src="docs/icons/linux.svg" height="13" width="13" alt="Linux" style="vertical-align: middle;" /> **Linux**: `sudo apt install ffmpeg` |
| **ExifTool** | **Optional / Recommended** | Extended photo and video metadata. Apple Live Photo pairing metadata does not require ExifTool; FFmpeg is required to create its paired `.mov`. | • <img src="docs/icons/windows.svg" height="13" width="13" alt="Windows" style="vertical-align: middle;" /> **Windows**: `winget install OliverBetz.ExifTool`<br>• <img src="docs/icons/apple.svg" height="13" width="13" alt="macOS" style="vertical-align: middle;" /> **macOS**: `brew install exiftool`<br>• <img src="docs/icons/linux.svg" height="13" width="13" alt="Linux" style="vertical-align: middle;" /> **Linux**: `sudo apt install libimage-exiftool-perl` |

> 🔍 *You can check and verify your system's FFmpeg and ExifTool status anytime inside BeReal Studio under **Settings ⚙️ &rarr; System & Dependencies**.*

---

## 🖼️ Application Showcase

| 🏠 Home | 📱 Memories |
|:---:|:---:|
| ![Home Dashboard](docs/screenshots/01_home_dashboard.png) | ![Memories Explorer](docs/screenshots/02_memories_explorer.png) |

| 📅 Memories calendar | 🗺️ Map overview |
|:---:|:---:|
| ![Memories Calendar](docs/screenshots/10_memories_calendar.png) | ![Offline Memories map](docs/screenshots/06_memories_map.png) |

| 📍 Photo pins on the map | 🖼️ Gallery for a busy place |
|:---:|:---:|
| ![Memories map with photo pins](docs/screenshots/07_memories_map_detail.png) | ![Map gallery for a busy location](docs/screenshots/08_memories_map_dense_gallery.png) |

![Memories map density](docs/screenshots/09_memories_map_density.png)

| 📸 Photo processing | 🎬 Recap videos |
|:---:|:---:|
| ![Photo Processing](docs/screenshots/03_photo_toolkit_config.png) | ![Recap Video Generator](docs/screenshots/04_recap_video_config.png) |

![Activity history and active jobs](docs/screenshots/05_activity_history.png)

---

## ✨ Key Features

### 📱 Memories Explorer

- Browse your archive as photo cards, a monthly calendar, or a continuous feed.
- Swap the two cameras, move the selfie inset, and play behind-the-scenes clips.
- Search by caption, date, or place; filter by media type and location.
- Save one memory as a combined photo or video, an individual camera view, or a motion photo.

---

### 🗺️ Map Viewer

- See memories with saved locations as photo pins, nearby groups, and a density view. Open a place to browse its photos.
- Start from a location label on a memory to jump to its spot on the map.
- The world map works offline. Detailed street tiles are optional with your own MapTiler key; MapTiler receives the viewed area and your IP address, while your photos stay on your device.

---

### 📸 Photo Processing Suite

- Restore capture dates, locations, and captions; keep late posts with their correct BeReal day.
- Export both cameras in picture-in-picture or side-by-side layouts, with the main or selfie camera as the background.
- Make Samsung or Google motion photos, or Apple Live Photo packages for Mac Photos. Live Photo export needs a behind-the-scenes clip and FFmpeg; see the [Mac import guide](docs/releases/v2.6.0.md#apple-live-photos-for-mac-photos).
- Choose a date range and process a batch of photos together.

---

### 🎬 Recap Video Generator

- Turn selected memories into a video paced to your music, with adjustable timing and a waveform preview.
- Add dates and locations to slides and preview the sequence before rendering.
- Keep photo batches and video renders running in the background.

---

## 📋 How to Download Your Archive from BeReal
 
1. Open the **BeReal** mobile app and tap your **Profile icon** (bottom-right).
2. Tap the **Settings icon** (top-right).
3. Tap **Help** &rarr; Select **Contact Us**.
4. Select **Ask a Question** &rarr; Tap **Troubleshooting** &rarr; Tap **Other**.
5. Tap **Contact Us** at the bottom &rarr; Select the **Topic** dropdown.
6. Select **"I'd like to request a copy of my data"**.
7. Type a message with at least **10 characters** (e.g., *"Please provide a copy of my account data"*) and submit.
8. BeReal will deliver a secure download link via email containing your official archive ZIP (including `posts.json` and all media).
9. Once downloaded, select the ZIP or unzipped folder directly in **BeReal Studio**.

---

## 🛠️ Prerequisites (For Building from Source)

1. **Rust Toolchain:**
   - Install via [rustup.rs](https://rustup.rs) (Rust 1.78+ recommended).
2. **Package Manager & JavaScript Runtime:**
   - **Bun (Recommended for ultra-fast startup):** Install via [bun.sh](https://bun.sh) (`powershell -c "irm bun.sh/install.ps1 | iex"`).
   - **NPM / PNPM / Yarn (Fully Supported):** Standard Node.js v18+ environment works out-of-the-box.
3. **FFmpeg (For Recap Video & Apple Live Photos):**
   - Required for video slideshow encoding, dual-video PIP overlays, and Apple Live Photo MOV creation.
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
- **Windows:** `src-tauri/target/release/bundle/nsis/BeReal Studio_2.6.0_x64-setup.exe` or `src-tauri/target/release/bereal-studio.exe`
- **macOS:** `src-tauri/target/release/bundle/dmg/BeReal Studio_2.6.0_aarch64.dmg` or `src-tauri/target/release/bundle/macos/BeReal Studio.app`
- **Linux:** `src-tauri/target/release/bundle/deb/BeReal Studio_2.6.0_amd64.deb` or `appimage/BeReal Studio_2.6.0_amd64.AppImage`

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
├── package.json                            # App manifest & dependencies (v2.6.0)
└── README.md                               # User documentation & GDPR guide
```

---

## 🗺️ Future Roadmap & Upcoming Features

- [x] **📅 Native BeReal-Style Memories & Calendar Viewer** *(Completed in v2.0.0 & v2.3.0)*:
  - Monthly Memories Calendar Matrix with day thumbnails, late badges, and retake counters.
  - Interactive Feed & Lightbox with front/back camera click-to-swap and movable PIP.
  - Multi-dimensional search, hierarchical location drawers, and single-memory exports.
  - Smooth 1:1 pointer drag panning and zoom controls for high-res photo and video exploration.
  - Full Video BeReal support with synchronized dual-video playback and Side-by-Side MP4 combining.
- [x] **🍏 Apple Photos Live Photos Compatibility**:
  - Export paired still image (`.jpg`) and video (`.mov`) files with matching Apple Content Identifier UUID (`MakerApple:17` and `com.apple.quicktime.content.identifier`). A packaged export imported as one Live Photo on macOS 26.
- [ ] **🎬 Recap Video Library & Gallery Viewer**:
  - In-app gallery indexing all rendered recap MP4s with playback preview, waveform scrubber, and quick actions ("Open in Player", "Show in Explorer").

---

## 💖 Credits & Open Source Lineage

**BeReal Studio** is authored and maintained by **[NotToxel](https://github.com/NotToxel)** ([GitHub Repository](https://github.com/NotToxel/BeRealStudio)).

It unifies, rewrites, and modernizes the core capabilities of three pioneer open-source projects into a single, high-performance desktop application:

- **[BeReel](https://github.com/theOneAndOnlyOne/BeReel)** *(by [@theOneAndOnlyOne](https://github.com/theOneAndOnlyOne))* — Creator of the music-synchronized BeReal recap video generator.
- **[BeReal-GDPR-Photo-Toolkit](https://github.com/hatobi/bereal-gdpr-photo-toolkit)** *(by [@hatobi](https://github.com/hatobi))* — Pioneer of BeReal GDPR archive extraction, EXIF metadata restoration, and Picture-in-Picture photo compositing.
- **[makelive (Make Live)](https://github.com/RhetTbull/makelive)** *(by [@RhetTbull](https://github.com/RhetTbull))* — Reference implementation for pairing still photos and videos using matching Apple content identifiers.

---

## 📜 License

Copyright &copy; 2026 **NotToxel**.

BeReal Studio is licensed under the GNU General Public License v3.0 or later (GPL-3.0-or-later). See the [full license](LICENSE).

You can use, share, and modify the app under that license.
