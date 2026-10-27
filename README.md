# Hangar

A small native Windows app for Steam. It switches the signed-in Steam account without Steam's account picker or a password prompt, and shows every game on each account, installed and not installed, with a local favorites list. Built with Tauri 2 (Rust + the system WebView2), styled after Apple's Human Interface Guidelines and the TV app.

## What it does

- **Silent account switching.** Closes Steam, marks the chosen remembered account for auto-login (`loginusers.vdf` + `HKCU\Software\Valve\Steam\AutoLoginUser`) and restarts Steam with `-silent`, so it goes straight to the tray. Progress is shown while Steam signs in, and the app confirms the switch by watching Steam's `ActiveProcess\ActiveUser`.
- **Play as…** A game owned by another account on this PC gets a *Play as <name>* button: one click switches accounts and launches it.
- **Your whole library.** Games are grouped into *Installed* and *Not Installed*, in a grid of Steam's own cover art or a list with playtime, last played and size. Filter, sort, and switch between one account and all accounts.
- **Favorites** stored locally (`%APPDATA%\dev.patrick.hangar\settings.json`), shared by every account.
- **Real data, offline first.**
  - Game names, types, developers, genres and review scores come from Steam's own `appcache\appinfo.vdf`.
  - Playtime and last played come from each account's `userdata\<id>\config\localconfig.vdf`.
  - Installs come from `appmanifest_*.acf` in every library folder.
  - Artwork (covers, hero art, logos) is served from Steam's `appcache\librarycache` through a local `art://` protocol, with the Steam CDN as a fallback. Avatars come from `config\avatarcache`.
  - Store descriptions and screenshots load from the Steam store.
- **Optional Steam Web API key** (Settings) adds games an account owns but has never opened on this PC.

Hangar never reads or stores passwords. An account switches silently once it has signed in on this PC with *Remember me* checked; Steam keeps that sign-in token itself. Use *Add Account* to open Steam's sign-in window for a new one.

## Run

Requires Rust (stable, MSVC) and the WebView2 runtime, which ships with Windows 11.

```bash
cargo run --manifest-path src-tauri/Cargo.toml
```

Release build (single exe at `src-tauri/target/release/hangar.exe`):

```bash
cargo build --release --manifest-path src-tauri/Cargo.toml
```

Optional installer (`.exe` setup via NSIS) with the Tauri CLI:

```bash
cargo install tauri-cli --version "^2" --locked
cargo tauri build
```

## CI

`.github/workflows/build.yml` runs the tests, builds the portable exe and the NSIS setup on Windows, and publishes them as a GitHub Release:

- Every push to `main` (or a manual run) publishes `v<version>-build.<run number>`, for example `v1.0.0-build.2`.
- Pushing a tag such as `v1.1.0` publishes a release under that tag.
- Pull requests only build. The files are attached to the run as the `hangar-windows` artifact.

## Layout

```
ui/                 HTML/CSS/JS front end (no build step, no framework)
src-tauri/src/
  main.rs           commands exposed to the UI and the art:// protocol
  steam.rs          Steam paths, accounts, library, switching
  vdf.rs            Valve KeyValues: text files and the binary appinfo.vdf
  store.rs          Steam store details and the optional Web API
  settings.rs       settings.json, including favorites
```

Not affiliated with Valve. Steam is a trademark of Valve Corporation.
