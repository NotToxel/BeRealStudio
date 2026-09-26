# Memories map: product and implementation plan

Status: implemented in working tree · 25 September 2026

Implementation: `src/components/memories/MemoriesMap.svelte`, `src/components/memories/MemoryMapThumbnail.svelte`, `src/lib/memoriesMap.ts`, the Map tab in `src/views/MemoriesView.svelte`, bundled Natural Earth 1:110m country outlines, and a browser smoke check in `scripts/verify_memories_map.mjs`. MapTiler Streets v4 Dark is available after entering a key in the map; no key is embedded in the repository. The map code loads when the tab opens. The live results panel is throttled, with at most 100 rendered rows and 12 photo pins; remaining individual points stay visible as small map dots. Marker reconciliation runs during map rendering so clusters split and combine while zooming, and cluster camera moves use a shorter transition. Cluster size reflects the number of memories, including repeated coordinates, and the grouped photo pin displays its count. A stronger weighted heatmap and legend show broad density. Dense groups stay clustered through high zoom, and selecting one opens a paginated photo gallery. Nearby rows, place covers, and gallery cards use the BeReal viewer's 3:4 frame and global camera perspective, with a larger inset to keep the second camera legible at preview size. Places follows the explorer's country → city → suburb filters. The browser check covers anchoring through a pan, live zoom transitions, dual-camera preview size and swapping, feed opening, place levels, and 120 photos at one location. Actual MapTiler requests require the user's key; cross-platform Tauri rendering and large-archive profiling remain release checks.

## Goal

Add **Map** beside Memories and Calendar in the Memories Explorer. A person should be able to see where their geotagged BeReals were taken, zoom from a worldwide overview to individual posts, and open a post in the existing memory feed. The map should work with the existing archive and filters and preserve the app's local-first character.

The user has chosen a comparable map rather than requiring Google Maps. Use **MapLibre GL JS** for the map renderer. Keep the basemap source configurable so the map is not tied to one tile vendor.

Confirmed product choices: online detailed map tiles are acceptable, with a basic offline overview; grouping is automatic by place and map proximity, with no personal collections; the visible-area results panel should update while the map moves when performance permits.

## What we learned

- `ExplorerMemory.location` already contains numeric latitude and longitude from the archive. `load_explorer_memories` passes that location through, and the explorer already has a location filter plus year, month, country, city, suburb, caption, video, BTS, and text filters. No new archive parser or geocoding service is needed for the first map release.
- The explorer already has `filteredMemories`, `getSafeImageSrc`, `DualCameraFrame`, and `openFeedAt`. The map should reuse these rather than own a second filter or full-screen viewer.
- Google Photos' current map pairs the visible map area with a photo grid, lets a person tap a heat area to focus it, and offers Places grouping. Its timeline uses separate Location History data, which BeReal Studio does **not** have. We can group by the BeReal capture date but should not draw a continuous journey or imply tracked travel.
- Google Maps' built-in `HeatmapLayer` became unavailable in May 2026. MapLibre has native GeoJSON clustering and heatmap layers, which fit the proposed zoom behavior. Google Maps would additionally require a billed key and platform policy work.
- BeReal's help material confirms Memories are private to the account holder. The map should retain that expectation: location coordinates and photo files stay local; an online basemap may expose only requested map tiles/viewport to its provider, with that disclosed in the UI.

Sources: [Google Photos map help](https://support.google.com/photos/answer/6153599?co=GENIE.Platform%3DAndroid&hl=en), [MapLibre clusters](https://maplibre.org/maplibre-gl-js/docs/examples/cluster/), [MapLibre heatmap](https://maplibre.org/maplibre-gl-js/docs/examples/create-a-heatmap-layer/), [Google heatmap deprecation](https://developers.google.com/maps/deprecations), [Google Maps key setup](https://developers.google.com/maps/documentation/javascript/get-api-key), [BeReal Memories help](https://help.bereal.com/hc/en-us/articles/7531349180829-Memories).

## Experience

### Visual reference from the supplied BeReal screenshots

The two 25 September 2026 screenshots show the same memory pin over western Europe and then southern England. Preserve the useful visual grammar: a deep navy sea, muted teal land, quiet place labels, a small rounded dual-camera thumbnail with a white border and pointer, and an unobstructed map canvas. The thumbnail remains approximately the same on-screen size as the map zoom changes; its location anchor stays fixed. Isolated posts should therefore remain photo pins at broad zoom. Density and count clusters replace **crowded** photo pins, rather than hiding every photo at a fixed zoom threshold. The blue dot/arrow in the screenshots represents the phone's current location, separate from the memory pin; desktop geolocation should be an optional control invoked by the user and must not be required for archive browsing. Adapt the phone's sparse full-screen layout to the desktop explorer's existing header and filter bar.

1. **Entry and scope.** Add a third `Map` tab to `MemoriesView.svelte`. Existing filters apply immediately to map points and the results panel. Show `N mapped · M without location` based on the current filter. No-location posts remain available in Memories and Calendar; the map explains why they do not appear. Initial camera fits valid filtered coordinates with sensible padding, capped to avoid zooming into a single building.
2. **Overview.** At country/continent scale, show isolated posts as the reference-style photo pins. For crowded regions, replace overlapping pins with a restrained density layer and count clusters. Density answers “where did I take BeReals?” without placing hundreds of photo thumbnails. The legend says that colour represents number of posts, not frequency of visits or time spent. Clicking a hotspot zooms to it.
3. **Approach.** As the camera zooms in, fade density out and make numbered clusters the primary marks. Clusters show the count and a representative dual-camera thumbnail only when it stays legible. Clicking a cluster expands it; repeated coordinates open a grouped post list rather than endlessly zooming.
4. **Detail.** At street scale, show a limited number of individual photo pins in the viewport. Selecting a pin opens a compact preview with the dual-camera composition, date, place/caption if present, and `Open memory`. That action calls the existing `openFeedAt` path. Hover is optional; click and keyboard access are essential.
5. **Linked results.** A collapsible side panel lists the memories in the visible map area, newest first, grouped by capture day with optional month headings. It updates during pan and zoom, throttled to animation frames or a modest interval, with a limited/virtualised row set so movement remains smooth. An accurate count updates alongside the list. If large archives make live updates visibly slow on a supported device, update the list on `moveend` while the map itself remains responsive. Selecting a row highlights and pans to its pin. A `Fit results` control returns to all filtered locations.
6. **View controls.** Keep the global filter bar visible. Add map-specific controls for density/cluster display, map style (standard and terrain if the chosen tiles support it), fit results, and legend. Save only harmless view preferences locally; do not persist exact last-viewed coordinates by default.

### Photo grouping

Spatial clustering is an automatic display technique. Offer **place groups** as a separate browsing feature: a drawer of countries/cities already derived by the offline geocoder, each with count, cover image, and `Show on map`. Include an `Unknown place name` group for posts with coordinates whose offline name cannot be resolved. The map also clusters nearby coordinates automatically as zoom changes. Neither grouping needs user-managed collections.

Do not infer a “trip” from sparse BeReal dates: users could have travelled between daily posts. Personal collections are outside the agreed map scope.

## Technical design

- Add `MemoriesMap.svelte` and a small map-data adapter. Derive GeoJSON only from `filteredMemories` with finite latitude in `[-90, 90]` and longitude in `[-180, 180]`. Report invalid and missing positions separately. `(0, 0)` is valid input but should be flagged as suspicious in diagnostics, not silently discarded.
- Keep only `{id, coordinates, date, minimal display fields}` in GeoJSON properties. Resolve media for pins/previews only when visible or selected. Never put image data URLs or raw archive JSON into map features.
- Use one clustered GeoJSON source for points and MapLibre's native cluster expansion for zoom. Use a heatmap layer only where broad-zoom point density warrants it. Choose thresholds by visual testing and viewport density, not rigidly by country/city labels or zoom alone. Render isolated unclustered points as white-bordered dual-camera HTML thumbnail markers, retaining their geographic anchor as the camera moves. Cap rendered HTML thumbnail markers; use map layers for the rest. Handle exact-coordinate stacks with a list/spider layout.
- Derive the side-panel set from the current viewport bounds and filtered IDs. Throttle live camera updates and virtualise or cap rendered rows; avoid reconstructing thumbnails during every drag frame. Measure interaction smoothness with a large archive and fall back to updating on `moveend` if needed. Guard the antimeridian when testing bounds. Filter changes update source, counts, panel, and selection together. If a selected memory is filtered out, close its preview.
- Keep the selected memory ID and map camera state in the component/store. Lazy-load the Map tab on first use. Because the explorer retains inactive views in the DOM, pause map rendering while hidden and call `resize()` when shown.
- Use a licensed tile source with explicit attribution and a documented quota/terms check. Ship a small local world outline or low-detail basemap so the overview remains useful offline; load detailed online tiles only when the user opens the map and the network is available. This online/offline split is confirmed. Do not use the public `tile.openstreetmap.org` service as an unlimited product backend or offer prohibited bulk prefetch. The app's current `csp: null` should be tightened to allow only chosen tile endpoints and local assets before release.
- No Rust change is needed for the first interactive map. Rust/local storage work may be needed for a bundled offline tile pack or archive-aware settings later.

## Delivery sequence

### Slice 1 — Functional map

Add Map tab, valid-coordinate adapter, MapLibre basemap and attribution, clustered marks, fit-results, map-specific empty/offline/error states, and direct `Open memory`. Integrate existing filters and show mapped/unmapped counts. This is the minimum usable release.

### Slice 2 — Rich exploration

Add broad-zoom density, constrained dual-camera photo pins, repeated-coordinate group list, linked viewport results panel, place drawer, and keyboard-friendly selection. Tune transitions and thumbnail budget using a large archive.

### Slice 3 — Refinement and scale

Tune large-archive performance and consider a “this day/this year” date scrubber only if it remains truthful to the available post timestamps. Do not create a continuous travel route without actual route history. Personal collections are not planned.

## Release checks

- Works for 0, 1, hundreds, and several thousand located posts; remains usable with an archive containing no GPS, missing media, invalid coordinates, duplicate coordinates, and locations across the antimeridian.
- Every visible number agrees with the active explorer filters. A selected map memory opens the same feed item as the grid; switching tabs preserves filters and does not leak stale selection.
- Map and panel are usable by mouse, keyboard, and screen reader; markers have date/place labels, clusters announce counts, controls have names, and the density palette has an accessible alternative in the results list.
- Verify Windows WebView2, macOS WebKit, and Linux WebKitGTK rendering, map resize after tab activation, offline behavior, tile attribution, and request destination. Exact photo coordinates and media must never be sent to the tile provider.
- Run `bun run check`, `bun run build`, and the repository's release preflight before a release. No release or commit is part of this plan.

## Implementation choices to close before Slice 1

1. Select a production tile source and license/attribution terms. Prefer a provider with a dark style matching the current UI, reasonable usage limits, and a stable URL configuration. The renderer remains MapLibre.
2. Choose a compact offline overview asset within the installer-size budget. An outline-only offline fallback is sufficient for the first release; detailed offline street maps are a separate feature.
3. Profile live panel updates on the supported desktop WebViews with a large archive and set a throttle/virtualisation budget. Preserve live updates if smooth; use `moveend` only where measurements justify it.
