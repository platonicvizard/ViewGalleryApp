# ViewGallery

A fast, cross-platform media gallery viewer for macOS, Windows, and Linux. Open a folder (or a single image/video) and browse everything inside it with keyboard arrows, like the built-in Windows Photos viewer — plus filtering, sorting, and a grid view.

## Features

- **Folder & file opening** — open a folder, open a single file, drag and drop a folder/file onto the window, or register ViewGallery as the default viewer for images/videos and double-click a file in Finder/Explorer/file manager.
- **Keyboard navigation** — `←` / `→` to move between media in the current folder, `Esc` to switch to grid view.
- **Filtering** — live filename search, plus quick filters for All / Images / Videos.
- **Sorting** — sort by Name, Date, or Size, with an ascending/descending toggle, right next to the filters.
- **Two views** — a full-size **Viewer** (with a filmstrip of thumbnails) and a virtualized **Grid** that stays smooth even with large folders.
- **Image + video support** — JPG, PNG, GIF, WEBP, BMP, TIFF, AVIF, HEIC/HEIF, SVG, MP4, MOV, WEBM, MKV, AVI, M4V, WMV, FLV, MPG/MPEG.

## Tech stack

- [Tauri 2](https://tauri.app/) (Rust) for the native shell — folder scanning, file dialogs, and OS file-association handling live in [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs).
- [SvelteKit](https://svelte.dev/) (Svelte 5 runes, SPA mode) + [Tailwind CSS v4](https://tailwindcss.com/) for the UI.

## Development

```bash
npm install
npm run tauri dev
```

This starts the Vite dev server and launches the native app window with hot reload.

## Building

```bash
npm run tauri build
```

Produces a native installer/bundle for the current OS (`.app`/`.dmg` on macOS, `.msi`/`.exe` on Windows, `.deb`/`.AppImage` on Linux) in `src-tauri/target/release/bundle/`.

## Project layout

- `src-tauri/src/lib.rs` — Rust commands (`scan_folder`, `parent_dir`, `take_initial_path`), single-instance + "open with" handling, and plugin setup (dialog, fs, opener).
- `src/lib/gallery.svelte.ts` — reactive store holding the current folder, items, filters, sort, and selection.
- `src/lib/Toolbar.svelte` — open folder/file, search, kind filter, sort controls, view toggle.
- `src/lib/Viewer.svelte` — full-size image/video viewer with arrow navigation and neighbor preloading.
- `src/lib/Filmstrip.svelte` — horizontal thumbnail strip shown under the viewer.
- `src/lib/Grid.svelte` — virtualized thumbnail grid.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
