# BeReal Studio changelog

This page highlights changes people will notice when using BeReal Studio. For the full story behind a release, see its [release notes](docs/releases/).

## 2.6.0 — Memories across the map

### Find memories by place

- A new **Map** view shows memories with saved locations as photo pins. Nearby posts gather into clusters, and a soft glow helps show where you have taken the most BeReals.
- Browse from country to city to suburb, then open the memories from a place in a gallery or the familiar feed.
- The map follows your existing search and filters. Its nearby list changes as you move around, so it is easier to explore one area at a time.
- Place names on memory cards and in the feed can take you straight to the matching spot on the map.
- The included world overview works offline. If you want streets and place labels, you can add your own MapTiler key and switch detailed maps on or off.
- Map controls, pin counts, and location jumps have been refined so crowded places are easier to explore.

### Save and share photos more easily

- **Apple Live Photo package** is a new export choice for a BeReal with a behind-the-scenes clip. It makes a `.pvt` folder containing a photo, a short movie, and the information macOS Photos needs to pair them.
- Apple Live Photo exports now report missing clips or FFmpeg problems instead of quietly leaving an incomplete pair.
- Batch photo processing can create the same packages when Apple Live Photos are enabled.
- Picture-in-picture exports more closely match the position, border, and rounded corners of an original BeReal. The on-screen preview has been adjusted to match.
- Export menus make it easier to find the Apple Live Photo package option and explain how to bring it into Photos.

### Everyday improvements

- Search understands more forms of dates and times, as well as captions and place names.
- Moving between a memory and its map location is more direct. Photo cards, feed controls, and map controls have received small navigation and layout improvements.
- The release checks now keep build examples in the README aligned with the app version while leaving download links pointed at the most recently published release.

### Before you use these features

- A memory needs a saved location to appear on the map. Memories without one remain available in the grid, calendar, and feed.
- Detailed street maps need an internet connection and your own MapTiler key. The offline world view and your memory locations work without one.
- Apple Live Photo export needs a behind-the-scenes clip, JPEG output, and FFmpeg. For the tested Mac import route, transfer the `.pvt` folder, then open it in Finder. To get the Live Photo onto iPhone, sync it from Mac Photos or AirDrop the Photos asset.

[Read the complete v2.6.0 release notes](docs/releases/v2.6.0.md) · [See all changes since v2.3.2](https://github.com/NotToxel/BeRealStudio/compare/v2.3.2...v2.6.0)

## 2.3.2 — Performance polish and smoother navigation

The previous release made the app reopen its existing window when launched twice, refined timeline browsing, improved the full-screen memory feed, and strengthened handling of unusual archives and recap settings. [Read the v2.3.2 release notes](docs/releases/v2.3.2.md).

## Earlier releases

- **[2.3.0](docs/releases/v2.3.0.md):** Added video BeReal playback and dual-camera video exports, smoother photo zooming, separate quick-save choices for different kinds of media, and easier calendar browsing.
- **2.2.0:** Improved speed, layout, and the day-to-day feel of the Memories view.
- **2.1.0:** Refined stability and export choices.
- **[2.0.0](docs/releases/v2.0.0.md):** Introduced the Memories explorer, calendar, continuous feed, and interactive dual-camera photo view.
- **1.0.0:** The first unified desktop release.
