# 🚀 BeReal Studio v2.0.0 — The Memories Explorer Release

**BeReal Studio v2.0.0** is a major milestone release that introduces the **Native BeReal Memories & Calendar Explorer**, bringing the authentic, fluid mobile BeReal browsing and post exploration experience directly to your desktop alongside the Photo Toolkit and Music Recapper.

---

## 🌟 What's New in v2.0.0

### 📸 1. Native Memories & Calendar Explorer
- **Mobile-Identical Dark UI**: An immersive experience matching the authentic BeReal app design.
- **Segmented View Switcher**: Fluid toggle between `[ Memories ]` and `[ Calendar ]`.
- **Memories Grid**: Responsive multi-column card grid with day overlays (`19`, `20`, `Today`), retake badges `(2)`, and hover elevations.
- **Monthly Calendar Matrix**: 7-column calendar (Mon–Sun) with month navigation (`<` / `>`) and active memory photo cells.
- **Persistent Directory Memory**: Remembers and auto-loads your previously scanned GDPR archives across app launches.

### 📱 2. Continuous Vertical Infinite-Scroll Feed
- **Authentic Feed View**: Clicking any memory card seamlessly opens a continuous, smooth vertical scroll feed of your entire archive.
- **Instant Auto-Scroll**: Jumps directly to the tapped post while allowing uninterrupted scrolling up and down through earlier and newer memories.
- **Dynamic Sticky Top Bar**: Tracks and displays the active visible memory date and post count (e.g. `18 August 2024 • 14 of 420`) as you scroll.
- **Detailed Metadata**: Displays user avatar, username (`toxel`), full name, post time, relative date, late status, retakes, and captions.

### 🤳 3. Interactive Dual-Camera Frame & Movable PIP
- **Click-to-Swap**: Clicking the selfie inset instantly swaps perspective between front and back cameras with animated transitions.
- **Movable / Draggable PIP**: Click and drag the selfie picture-in-picture box anywhere on the photo frame with bounds protection.
- **4-Corner Snap Hotspots**: Instant corner snap controls to cycle between `Top-Left`, `Top-Right`, `Bottom-Right`, and `Bottom-Left`.
- **Integrated BTS Micro-Video Player**: Plays live BTS micro-videos inline when present in the archive.

### 🔍 4. Multi-Dimensional Search & Filtering
- **Live Instant Search**: Search captions, dates, and reverse-geocoded cities/countries.
- **Quick Filter Chips**:
  - `📍 Location` (Posts with GPS tags)
  - `🎬 BTS Clips` (Posts with live BTS video clips)
  - `💬 Captions` (Posts with text captions)
  - `🔄 Retakes Only` (Posts with retakes > 0)
- **Advanced Drawers**: Instant dropdown filtering by Year, Month, City, and Country.

### 💾 5. Contextual Actions & Single-Memory Exports
- **Three-Dot Action Menu**:
  - 📁 *Reveal in File Explorer*: Opens the raw image on disk.
  - 🖼️ *Save Combined Photo (PIP)*: Exports a high-res Picture-in-Picture composite.
  - ↔️ *Save Side-by-Side Photo*: Exports a high-res side-by-side composite.
  - 📷 *Save Main Camera Only* / 🤳 *Save Selfie Camera Only*.
  - 📋 *Copy Details*: Quick actions to copy caption, formatted timestamp, or GPS coordinates.

### ⚡ 6. High-Performance Parallel Engine
- **Rayon Multi-Core Processing**: Parses, formats, and reverse-geocodes 1,000+ posts in under **10 milliseconds**.
- **O(1) Media File Indexing**: Instant filename mapping supporting all BeReal GDPR folder variations (`Photos/.../post/`, `/Photos/.../bereal/`, flat folders).
- **Offline Spatial Reverse Geocoder**: In-memory GeoNames spatial index for instant, offline city/country resolution with zero network pauses.
- **Tauri v2 Asset Protocol & Base64 Fallback**: 100% reliable image and video rendering across all operating systems.

---

## 📦 Downloads & Platform Installers

| Platform | Package | Architecture | Description |
| :--- | :--- | :--- | :--- |
| **Windows** | `.msi` | x64 | Windows Installer Package |
| **Windows** | `.exe` | x64 | NSIS Standalone Setup |
| **macOS** | `.dmg` | Universal | Apple Silicon (M1/M2/M3/M4) & Intel (x86_64) |
| **Linux** | `.AppImage` | x64 | Standalone Universal Linux Binary |
| **Linux** | `.deb` | x64 | Debian / Ubuntu Package |

---

## 🚀 Pushing to GitHub & Publishing Releases

To publish this release to GitHub:
```bash
git push origin master --tags
```
The included GitHub Actions workflow (`.github/workflows/release.yml`) will automatically build, package, and upload binaries for **Windows**, **macOS (Universal)**, and **Linux** directly to your GitHub Releases page!
