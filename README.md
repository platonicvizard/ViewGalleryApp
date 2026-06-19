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

## Releasing

Pushing a `v*` tag triggers [`.github/workflows/release.yml`](.github/workflows/release.yml), which builds installers for macOS (universal), Windows, and Linux and publishes them as a **public** GitHub Release with the installers attached.

```bash
npm version patch   # or minor / major — bumps + syncs package.json, tauri.conf.json, Cargo.toml, then commits and tags
git push --follow-tags
```

The `npm version` step runs [`scripts/sync-version.mjs`](scripts/sync-version.mjs) automatically so all three version fields stay in sync.

Code-signing and notarization credentials are **not** part of this repo and are entirely optional — without them the workflow still builds and publishes working but *unsigned* installers (macOS shows an "unidentified developer" warning, Windows shows "unknown publisher"). See below if you want to remove those warnings.

## Secrets & Signing

All of this is optional. Skip it if unsigned installers are fine for now.

### What each secret is for

| Secret | Used for | Required for |
| --- | --- | --- |
| `APPLE_CERTIFICATE` | base64-encoded `.p12` export of your Apple Developer ID Application certificate | macOS code signing |
| `APPLE_CERTIFICATE_PASSWORD` | password you set when exporting the `.p12` | macOS code signing |
| `APPLE_SIGNING_IDENTITY` | the identity string, e.g. `Developer ID Application: Jane Doe (ABCDE12345)` | macOS code signing |
| `APPLE_ID` | your Apple ID email | macOS notarization |
| `APPLE_PASSWORD` | an **app-specific password** (not your Apple ID password) | macOS notarization |
| `APPLE_TEAM_ID` | your Apple Developer Team ID | macOS notarization |
| `TAURI_SIGNING_PRIVATE_KEY` | private key for the Tauri updater | only if/when you add the [updater plugin](https://v2.tauri.app/plugin/updater/) — not used yet in this app |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | password for that key | same as above |

Windows builds in CI are unsigned by default (no Microsoft-equivalent of the above is wired up). If you buy a code-signing certificate later, the usual approach is to base64-encode the `.pfx`, import it into the Windows runner's certificate store in a build step, and reference its thumbprint via `bundle.windows.certificateThumbprint` in `src-tauri/tauri.conf.json` — ask if you want this added once you have a certificate.

### Step 1 — Get the Apple values (macOS signing + notarization)

1. Join the [Apple Developer Program](https://developer.apple.com/programs/) (paid, required for Developer ID certificates).
2. In Xcode or on [developer.apple.com](https://developer.apple.com/account/resources/certificates/list), create a **Developer ID Application** certificate and download it.
3. Double-click the downloaded certificate to install it into **Keychain Access**.
4. In Keychain Access, find the certificate under "My Certificates", right-click it → **Export** → save as `cert.p12`, and set an export password — this becomes `APPLE_CERTIFICATE_PASSWORD`.
5. Base64-encode it:
   ```bash
   base64 -i cert.p12 | pbcopy
   ```
   This is the value for `APPLE_CERTIFICATE`.
6. Get the exact signing identity string:
   ```bash
   security find-identity -v -p codesigning
   ```
   Copy the quoted string, e.g. `Developer ID Application: Jane Doe (ABCDE12345)` — this is `APPLE_SIGNING_IDENTITY`.
7. Go to [appleid.apple.com](https://appleid.apple.com/) → **Sign-In and Security** → **App-Specific Passwords** → generate one. This is `APPLE_PASSWORD` (use the Apple ID email itself as `APPLE_ID`).
8. Find your Team ID at [developer.apple.com/account](https://developer.apple.com/account) → **Membership details**. This is `APPLE_TEAM_ID`.

### Step 2 — (Optional, future) Generate a Tauri updater key

Only needed if you later add the updater plugin:

```bash
npx @tauri-apps/cli signer generate -w ~/.tauri/viewgallery.key
```

This prints/saves a private and public key. The private key (or its file contents) becomes `TAURI_SIGNING_PRIVATE_KEY`; the password you choose becomes `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. The **public** key would go into `tauri.conf.json`'s updater config (not secret).

### Step 3 — Add the secrets on GitHub (so CI releases can use them)

1. Open the repo on GitHub → **Settings** tab.
2. In the left sidebar: **Secrets and variables → Actions**.
3. Click **New repository secret**.
4. For each row in the table above that applies to you, set the **Name** exactly as shown (e.g. `APPLE_CERTIFICATE`) and paste the **Value**, then **Add secret**.
5. Repeat for all the secrets you want to configure. They're encrypted at rest, never shown again in the UI, and only readable by GitHub Actions workflows in this repo — not visible to the public even though the repo and its releases are public.
6. Next push of a `v*` tag will pick them up automatically — no workflow changes needed for the Apple/updater ones, since [`release.yml`](.github/workflows/release.yml) already references them via `secrets.*`.

### Step 4 — Use the same secrets locally (optional)

If you want to produce a signed build on your own Mac with `npm run tauri build` (rather than only via CI), export the same values as environment variables in your shell before building — **do not** commit them to any file that's tracked by git:

```bash
export APPLE_CERTIFICATE="$(base64 -i cert.p12)"
export APPLE_CERTIFICATE_PASSWORD="..."
export APPLE_SIGNING_IDENTITY="Developer ID Application: Jane Doe (ABCDE12345)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="...app-specific password..."
export APPLE_TEAM_ID="ABCDE12345"

npm run tauri build
```

If you'd rather not retype these every session, put them in a local `.env` file (already covered by `.gitignore`, so it will never be committed) and load it with `source .env` (using `export VAR=value` lines) before building.

## Project layout

- `src-tauri/src/lib.rs` — Rust commands (`scan_folder`, `parent_dir`, `take_initial_path`), single-instance + "open with" handling, and plugin setup (dialog, fs, opener).
- `src/lib/gallery.svelte.ts` — reactive store holding the current folder, items, filters, sort, and selection.
- `src/lib/Toolbar.svelte` — open folder/file, search, kind filter, sort controls, view toggle.
- `src/lib/Viewer.svelte` — full-size image/video viewer with arrow navigation and neighbor preloading.
- `src/lib/Filmstrip.svelte` — horizontal thumbnail strip shown under the viewer.
- `src/lib/Grid.svelte` — virtualized thumbnail grid.

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
