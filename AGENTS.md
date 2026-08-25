# BeReal Studio — Agent Instructions & Knowledge Base (AGENTS.md)

## 📌 Project Overview
**BeReal Studio** is a unified, cross-platform desktop application built with **Tauri v2 (Rust backend)** and **Svelte 5 + Vite (Frontend)**. It provides an all-in-one suite to import, parse, process, composite, restore metadata on, and create music-synchronized recap videos from personal BeReal GDPR data archives.

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
│   │   └── fonts.ts                        # Curated built-in font definitions
│   ├── components/                         # Reusable UI components
│   │   ├── NavBar.svelte                   # Top navigation bar (Home, Photos, Recap, Settings, About)
│   │   ├── Toggle.svelte                   # Animated on/off switch
│   │   ├── Slider.svelte                   # Value range slider with value pill
│   │   ├── FilePicker.svelte               # Native folder & file dialog wrapper
│   │   ├── DateRangePicker.svelte          # Dual date pickers with monthly density histogram
│   │   ├── ProgressBar.svelte              # Streaming progress indicator
│   │   ├── LogConsole.svelte               # Color-coded live terminal log
│   │   ├── ErrorModal.svelte               # Categorized error overlay
│   │   ├── FontPicker.svelte               # Curated 7-font dropdown selector
│   │   └── RuleEditor.svelte               # Geocoded location rules editor
│   ├── views/                              # Primary Application Views
│   │   ├── Home.svelte                     # Main dashboard with hero & feature cards
│   │   ├── ToolkitConfig.svelte            # Photo processing configuration view
│   │   ├── RecapperConfig.svelte           # Recap video configuration view with live preview
│   │   ├── Processing.svelte               # Real-time progress & live streaming log view
│   │   ├── Complete.svelte                 # Summary metrics, output opener & log exporter
│   │   ├── Settings.svelte                 # Global defaults, FFmpeg detection & reset
│   │   └── About.svelte                    # Privacy manifesto, GDPR guide & open source credits
│   └── routes/+page.svelte                 # SPA root page router & settings sync
│
├── src-tauri/                              # Rust Backend (Tauri v2)
│   ├── Cargo.toml                          # Native dependencies (image, img-parts, symphonia, rayon, etc.)
│   ├── tauri.conf.json                     # Desktop window & plugin configuration
│   ├── capabilities/default.json           # Tauri v2 security capabilities
│   └── src/
│       ├── main.rs & lib.rs                # Tauri entry & command registration
│       ├── state.rs                        # Global state, ProgressEmitter & log buffer
│       ├── assets/fonts/                   # 7 Embedded TTF fonts (Inter, Roboto, Outfit, Bebas, etc.)
│       ├── commands/                       # IPC Command Handlers
│       │   ├── archive.rs                  # scan_archive, extract_zip
│       │   ├── toolkit.rs                  # start_toolkit, cancel_toolkit (Rayon multi-core)
│       │   ├── recapper.rs                 # start_recapper, cancel_recapper
│       │   ├── settings.rs                 # load_settings, save_settings, reset_settings
│       │   ├── system.rs                   # check_ffmpeg, list_system_fonts
│       │   └── debug.rs                    # export_debug_log, get_debug_logs
│       ├── pipeline/                       # Photo Processing Logic
│       │   ├── parser.rs                   # JSON parsing & monthly distribution histogram
│       │   ├── image_ops.rs                # Format conversion, PIP & Side-by-Side compositing
│       │   ├── exif_writer.rs              # Lossless EXIF & IPTC JPEG segment injection
│       │   ├── motion_photo.rs             # Samsung SEFH/SEFT binary muxer & GCamera XMP
│       │   ├── video_ops.rs                # FFmpeg dual-video PIP overlay
│       │   ├── date_filter.rs              # Range filtering & density distribution
│       │   └── cleanup.rs                  # Intermediate artifact cleanup
│       └── recapper/                       # Recap Video Engine
│           ├── audio.rs                    # Symphonia audio decoding & duration
│           ├── timing.rs                   # Quadratic ramp / even timing curves
│           ├── geocoder.rs                 # Nominatim reverse geocoding with cache
│           ├── location_rules.rs           # Country-specific location formatting engine
│           ├── font_resolver.rs            # Built-in font resolver & disk loader
│           ├── frame_renderer.rs           # Image resize & text overlay with shadows
│           └── video_encoder.rs            # Raw RGB frame piping to FFmpeg stdin
│
├── static/fonts/                           # Static font files for browser preview parity
├── archive/                                # Legacy Python scripts & specifications (git-ignored)
├── package.json                            # App manifest & dependencies (v0.1.0)
└── README.md                               # User documentation & GDPR guide
```

---

## 🚦 Semantic Versioning & Commit Protocol

1. **Explicit Confirmation Required:**
   - **DO NOT** create git commits automatically or proactively.
   - Only execute `git commit` when given **explicit confirmation / instruction** by the user.

2. **When Confirmation is Given:**
   - **Semantic Version Bump:** Increment the version number in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json` in accordance with SemVer:
     - `PATCH` (`1.0.0` &rarr; `1.0.1`): Bug fixes, UI polish, minor styling updates.
     - `MINOR` (`1.0.0` &rarr; `1.1.0`): New features, major UI additions, new pipeline capabilities.
     - `MAJOR` (`1.0.0` &rarr; `2.0.0`): Breaking architectural shifts or milestone releases.
   - **Clean & Readable Commit Message:** Follow a structured, human-readable format:
     ```
     <type>(<scope>): <concise summary in present imperative tense>

     - Overview: <1-2 sentence high-level summary>
     - Backend (Rust):
       - <Concise bullet points for core logic / commands / crates>
     - Frontend (Svelte):
       - <Concise bullet points for UI, stores, and components>
     ```
     *(Avoid verbose logs, raw command dumps, or test verification outputs).*

---

## 🛠️ Development & Build Commands (Powered by Bun)

- **Development Server:** `bun run tauri dev`
- **Frontend Check / Build:** `bun run build`
- **Backend Tests:** `cargo test` (run in `src-tauri/`)
- **Production Installer Build:** `bun run tauri build`
