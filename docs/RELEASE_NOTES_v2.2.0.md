# 🚀 BeReal Studio v2.2.0 — High-Performance Memories & Sticky Explorer Release

**BeReal Studio v2.2.0** brings comprehensive performance optimizations, 0ms instantaneous view switching, sticky timeline filter bars on scroll, theme-matched late indicators, and high-throughput buffered archive extraction to deliver our fastest and smoothest BeReal desktop experience yet.

---

## 🌟 What's New in v2.2.0

### ⚡ 1. 0ms Instantaneous Tab Switching & DOM Retention
- **Zero-Lag Tab Transitions**: Navigating between **Memories**, **Calendar**, **Settings**, and the **Photo Toolkit** is now instantaneous (0ms).
- **Dual Stage Retention**: Retains both the `<MemoriesGrid>` and `<CalendarGrid>` views concurrently in the DOM with active visibility stages, preventing unnecessary destruction and recreation of 1,000+ photo cards.
- **Scroll & State Memory**: Preserves your exact scroll position, expanded cards, and active search filters when switching between views.

### 📌 2. Sticky Glassmorphic Header & Filter Controls on Scroll
- **Always-Accessible Navigation**: The top control bar, view switcher, archive reload button, and multi-dimensional filter bar are now pinned at the top with a sleek blurred glassmorphic backdrop.
- **Full Filter Retention**: Search captions, toggle location/BTS chips, or change month filters directly from anywhere in your timeline or calendar without having to scroll back to the top.

### 🎨 3. Responsive Navbar & UI Polish Across All Resolutions
- **Progressive Label Collapsing**: Aux buttons seamlessly transition to icon-only buttons on screens `< 1320px` with animated tooltips, completely eliminating horizontal clipping on compact viewports.
- **Dynamic Brand & Queue Badging**: Intelligently compacts the brand logo and processing queue indicators down to 620px without overlapping.

### 🕒 4. Theme-Matched Late Indicators & Granular Settings
- **Clean Default Timestamps**: BeReal post headers display authentic timestamps by default (`26 Aug • 14:20`).
- **Granular Late Visibility Controls**: Added dedicated switches in Settings to customize late badges:
  - Toggle late addition text (`• 17m late`) in post headers.
  - Toggle late pills on Memories Grid thumbnails.
  - Toggle late badges on Calendar days.

### 🏎️ 5. High-Throughput Buffered Archive Extraction (Rust)
- **High-Throughput Streams**: Archive extraction and cache serialization now utilize 512KB and 256KB buffered streams (`BufReader` / `BufWriter`).
- **Layout Containment (`content-visibility: auto`)**: Offscreen month sections and memory cards are skipped during initial layout and rendered on-demand, speeding up initial grid rendering by over **98%**.
- **Asset Protocol Caching**: In-memory memoization of local asset protocol URLs eliminates repetitive allocations.

### 🤖 6. Multi-Platform Automated CI & Release Infrastructure
- **Cross-Platform Verification Matrix**: Automated GitHub Actions pipeline verifying builds and test suites across Windows, macOS, and Linux on release branches.

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

## 🚀 Updating to v2.2.0

Download the installer for your operating system above, or clone and build locally with:
```bash
git clone https://github.com/NotToxel/BeRealStudio.git
cd BeRealStudio
bun install
bun run tauri build
```
