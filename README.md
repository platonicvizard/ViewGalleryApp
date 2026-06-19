# ViewGallery

A fast, cross-platform media gallery viewer for macOS, Windows, and Linux. Open a folder (or a single image/video) and browse everything inside it with keyboard arrows, like the built-in Windows Photos viewer — plus filtering, sorting, and a grid view.

## Features

- **Folder & file opening** — open a folder, open a single file, drag and drop a folder/file onto the window, or register ViewGallery as the default viewer for images/videos and double-click a file in Finder/Explorer/file manager.
- **Keyboard navigation** — `←` / `→` to move between media in the current folder, `Esc` to switch to grid view.
- **Filtering** — live filename search, plus quick filters for All / Images / Videos.
- **Sorting** — sort by Name, Date, or Size, with an ascending/descending toggle, right next to the filters.
- **Two views** — a full-size **Viewer** (with a filmstrip of thumbnails) and a virtualized **Grid** that stays smooth even with large folders.
- **Image + video support** — JPG, PNG, GIF, WEBP, BMP, TIFF, AVIF, HEIC/HEIF, SVG, MP4, MOV, WEBM, MKV, AVI, M4V, WMV, FLV, MPG/MPEG.

## Installing

Download the installer for your operating system from the [Releases page](../../releases):

- **macOS** — download the `.dmg`, open it, and drag ViewGallery into Applications. The first time you open it, macOS may show an "unidentified developer" warning unless the build is signed — right-click the app and choose **Open** to bypass it once.
- **Windows** — download the `.msi` or `.exe` and run it. Windows may show a "Windows protected your PC" SmartScreen prompt unless the build is signed — click **More info → Run anyway**.
- **Linux** — download the `.AppImage` (make it executable with `chmod +x ViewGallery*.AppImage`, then run it) or the `.deb` (install with `sudo dpkg -i ViewGallery*.deb`).

## Tech stack

- [Tauri 2](https://tauri.app/) (Rust) for the native shell — folder scanning, file dialogs, and OS file-association handling live in [`src-tauri/src/lib.rs`](src-tauri/src/lib.rs).
- [SvelteKit](https://svelte.dev/) (Svelte 5 runes, SPA mode) + [Tailwind CSS v4](https://tailwindcss.com/) for the UI.

## Developing

```bash
npm install
npm run tauri dev
```

This starts the Vite dev server and launches the native app window with hot reload.

## Building from source

```bash
npm run tauri build
```

Produces a native installer/bundle for the current OS (`.app`/`.dmg` on macOS, `.msi`/`.exe` on Windows, `.deb`/`.AppImage` on Linux) in `src-tauri/target/release/bundle/`.

## Releasing a new version

Pushing a `v*` tag triggers [`.github/workflows/release.yml`](.github/workflows/release.yml), which builds installers for macOS (universal), Windows, and Linux and publishes them as a public GitHub Release with the installers attached.

1. Bump the version and sync it across `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml`:
   ```bash
   npm version patch   # or: minor / major
   ```
   This runs [`scripts/sync-version.mjs`](scripts/sync-version.mjs) automatically, then commits and tags the result (e.g. `v0.2.0`).
2. Push the commit and the tag:
   ```bash
   git push --follow-tags
   ```
3. Open the **Actions** tab on GitHub and watch the `Release` workflow run. When it finishes, the new version appears on the **Releases** page with installers for all three platforms attached.

Code-signing and notarization credentials are not part of this repository and are entirely optional. Without them, the workflow still builds and publishes working but *unsigned* installers (see the warnings mentioned in **Installing** above). The next section walks through adding them.

## Secrets & signing (optional)

Signing removes the "unidentified developer" / "unknown publisher" warnings users see when installing. This is optional — skip this whole section if unsigned installers are fine for now.

### What each secret does

| Secret name | What it is | Enables |
| --- | --- | --- |
| `APPLE_CERTIFICATE` | base64-encoded `.p12` export of an Apple Developer ID Application certificate | macOS code signing |
| `APPLE_CERTIFICATE_PASSWORD` | the password set when exporting the `.p12` | macOS code signing |
| `APPLE_SIGNING_IDENTITY` | the identity string, e.g. `Developer ID Application: Jane Doe (ABCDE12345)` | macOS code signing |
| `APPLE_ID` | the Apple ID email used to sign in to the Apple Developer account | macOS notarization |
| `APPLE_PASSWORD` | an app-specific password for that Apple ID (not the regular account password) | macOS notarization |
| `APPLE_TEAM_ID` | the Apple Developer Team ID | macOS notarization |
| `WINDOWS_CERTIFICATE` | base64-encoded `.pfx` export of a Windows code-signing certificate | Windows code signing |
| `WINDOWS_CERTIFICATE_PASSWORD` | the password set when exporting the `.pfx` | Windows code signing |
| `TAURI_SIGNING_PRIVATE_KEY` | private key for the Tauri auto-updater | only needed if the [updater plugin](https://v2.tauri.app/plugin/updater/) is added later — this app doesn't use it yet |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | password for that key | same as above |

### Setting up macOS signing and notarization

1. Join the [Apple Developer Program](https://developer.apple.com/programs/) (a paid membership is required for Developer ID certificates).
2. On [developer.apple.com → Certificates](https://developer.apple.com/account/resources/certificates/list), create a **Developer ID Application** certificate and download it.
3. Double-click the downloaded certificate file to install it into the macOS **Keychain Access** app.
4. In Keychain Access, find the certificate under "My Certificates", right-click it, choose **Export**, save it as `cert.p12`, and set an export password. That password is the value for `APPLE_CERTIFICATE_PASSWORD`.
5. Base64-encode the exported file:
   ```bash
   base64 -i cert.p12 | pbcopy
   ```
   The copied text is the value for `APPLE_CERTIFICATE`.
6. Get the exact signing identity string:
   ```bash
   security find-identity -v -p codesigning
   ```
   Copy the quoted string (e.g. `Developer ID Application: Jane Doe (ABCDE12345)`). That is the value for `APPLE_SIGNING_IDENTITY`.
7. Go to [appleid.apple.com](https://appleid.apple.com/) → **Sign-In and Security** → **App-Specific Passwords**, and generate one. That password is the value for `APPLE_PASSWORD`; the Apple ID email itself is the value for `APPLE_ID`.
8. Find the Team ID at [developer.apple.com/account](https://developer.apple.com/account) under **Membership details**. That is the value for `APPLE_TEAM_ID`.

### Setting up Windows code signing

1. Buy a code-signing certificate from a certificate authority (e.g. DigiCert, Sectigo, SSL.com) or use an organization-issued one. It will be provided as, or convertible to, a `.pfx` file with a password.
2. Base64-encode the `.pfx` file.
   - On Windows (PowerShell):
     ```powershell
     certutil -encode cert.pfx cert_base64.txt
     ```
     Open `cert_base64.txt` and copy everything between the `-----BEGIN CERTIFICATE-----` and `-----END CERTIFICATE-----` lines.
   - On macOS/Linux:
     ```bash
     base64 -i cert.pfx
     ```
   The result is the value for `WINDOWS_CERTIFICATE`. The export password is the value for `WINDOWS_CERTIFICATE_PASSWORD`.
3. Add the following step to the `windows-latest` job in [`.github/workflows/release.yml`](.github/workflows/release.yml), placed before the "Build and release" step, to import the certificate and capture its thumbprint:
   ```yaml
   - name: Import Windows signing certificate
     if: matrix.platform == 'windows-latest'
     shell: pwsh
     run: |
       $bytes = [Convert]::FromBase64String("${{ secrets.WINDOWS_CERTIFICATE }}")
       [IO.File]::WriteAllBytes("cert.pfx", $bytes)
       $cert = Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\CurrentUser\My `
         -Password (ConvertTo-SecureString -String "${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}" -AsPlainText -Force)
       echo "CERT_THUMBPRINT=$($cert.Thumbprint)" >> $env:GITHUB_ENV
   ```
4. Add `certificateThumbprint` to the `bundle.windows` section of `src-tauri/tauri.conf.json` so the bundler signs with that certificate:
   ```json
   "bundle": {
     "windows": {
       "certificateThumbprint": "${CERT_THUMBPRINT}"
     }
   }
   ```
   Since this value is only known at build time on the Windows runner, replace the placeholder with the real thumbprint using a small substitution step right after the import step above, for example:
   ```yaml
   - name: Apply certificate thumbprint to tauri.conf.json
     if: matrix.platform == 'windows-latest'
     shell: pwsh
     run: |
       (Get-Content src-tauri/tauri.conf.json) -replace '\$\{CERT_THUMBPRINT\}', $env:CERT_THUMBPRINT | Set-Content src-tauri/tauri.conf.json
   ```

### Adding the secrets on GitHub

1. Open the repository on GitHub and go to the **Settings** tab.
2. In the left sidebar, click **Secrets and variables → Actions**.
3. Click **New repository secret**.
4. Enter the **Name** exactly as shown in the table above (e.g. `APPLE_CERTIFICATE`), paste the **Value**, and click **Add secret**.
5. Repeat for every secret needed. Secrets are encrypted at rest, are never displayed again in the UI after creation, and can only be read by GitHub Actions workflows running in this repository — they stay private even though the repository and its releases are public.
6. The next push of a `v*` tag picks up the configured secrets automatically, since [`release.yml`](.github/workflows/release.yml) already references them through `secrets.*`.

### Using the same secrets for a local signed build

To produce a signed build on a local Mac with `npm run tauri build` instead of through CI, export the same values as environment variables in the shell before building. Do not write them into any file tracked by git:

```bash
export APPLE_CERTIFICATE="$(base64 -i cert.p12)"
export APPLE_CERTIFICATE_PASSWORD="your-export-password"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Jane Doe (ABCDE12345)"
export APPLE_ID="you@example.com"
export APPLE_PASSWORD="your-app-specific-password"
export APPLE_TEAM_ID="ABCDE12345"

npm run tauri build
```

To avoid retyping these every session, save them as `export VAR=value` lines in a local `.env` file — it's already excluded by `.gitignore`, so it will never be committed — and load it with `source .env` before building.

## Project layout

- `src-tauri/src/lib.rs` — Rust commands (`scan_folder`, `parent_dir`, `take_initial_path`), single-instance + "open with" handling, and plugin setup (dialog, fs, opener).
- `src/lib/gallery.svelte.ts` — reactive store holding the current folder, items, filters, sort, and selection.
- `src/lib/Toolbar.svelte` — open folder/file, search, kind filter, sort controls, view toggle.
- `src/lib/Viewer.svelte` — full-size image/video viewer with arrow navigation and neighbor preloading.
- `src/lib/Filmstrip.svelte` — horizontal thumbnail strip shown under the viewer.
- `src/lib/Grid.svelte` — virtualized thumbnail grid.

## Recommended IDE setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).
