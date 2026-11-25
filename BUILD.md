# Build Instructions

This guide covers how to set up the development environment and build Dictus Desktop from source across different platforms.

## Prerequisites

### All Platforms

- [Rust](https://rustup.rs/) (latest stable)
- [Bun](https://bun.sh/) package manager
- [Tauri Prerequisites](https://tauri.app/start/prerequisites/)

### Platform-Specific Requirements

#### macOS

- Xcode Command Line Tools
- Install with: `xcode-select --install`

#### Windows

- Microsoft C++ Build Tools
- Visual Studio 2019/2022 with C++ development tools
- Or Visual Studio Build Tools 2019/2022

#### Linux

- Build essentials
- ALSA development libraries
- libevdev (global shortcuts) and OpenBLAS, both installed by CI too
- Install with:

  ```bash
  # Ubuntu/Debian
  sudo apt update
  sudo apt install build-essential libasound2-dev pkg-config libssl-dev libvulkan-dev vulkan-tools glslc libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libgtk-layer-shell0 libgtk-layer-shell-dev patchelf cmake libevdev-dev libopenblas-dev

  # Fedora/RHEL
  sudo dnf groupinstall "Development Tools"
  sudo dnf install alsa-lib-devel pkgconf openssl-devel vulkan-devel \
    gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel \
    gtk-layer-shell gtk-layer-shell-devel \
    cmake libevdev-devel openblas-devel

  # Arch Linux
  sudo pacman -S base-devel alsa-lib pkgconf openssl vulkan-devel \
    gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg gtk-layer-shell \
    cmake libevdev openblas
  ```

## Setup Instructions

### 1. Clone the Repository

```bash
git clone git@github.com:getdictus/dictus-desktop.git
cd dictus-desktop
```

### 2. Install Dependencies

```bash
bun install
```

### 3. Start Dev Server

```bash
bun tauri dev
```

Dev builds (`bun tauri dev`, `bun tauri build --debug`) run as **Dictus Dev** (`com.dictus.desktop.dev`), separate from an installed Dictus. See [Dev identity](#dev-identity).

On Linux, `bun run tauri dev` currently panics at startup when built with cargo 1.100 or newer (tray icon not found, see [#111](https://github.com/getdictus/dictus-desktop/issues/111)). Run a debug build from its `.deb` instead:

```bash
bun run tauri build --debug --bundles deb
# run it without installing
dpkg-deb -x src-tauri/target/debug/bundle/deb/Dictus\ Dev_*_amd64.deb /tmp/dictus-dev
/tmp/dictus-dev/usr/bin/dictus
# or install it
sudo apt install ./src-tauri/target/debug/bundle/deb/Dictus\ Dev_*_amd64.deb
```

Do not start `src-tauri/target/debug/dictus` directly: its resources (tray icons, sounds, VAD model) are not next to the binary, and it panics in `lib.rs` when it loads the tray icon.

### 4. Build for Production

```bash
bun run tauri build
```

This compiles a release binary and generates platform-specific bundles (deb, rpm, AppImage on Linux; dmg on macOS; msi on Windows).

## Dev identity

`bun tauri …` goes through `scripts/tauri.ts`, which merges `src-tauri/tauri.dev.conf.json` into the config for `tauri dev` and `tauri build --debug`. A release `bun tauri build` is unchanged.

|                                                   | Release                         | Dev / debug                                  |
| ------------------------------------------------- | ------------------------------- | -------------------------------------------- |
| Name                                              | Dictus                          | Dictus Dev                                   |
| Identifier                                        | `com.dictus.desktop`            | `com.dictus.desktop.dev`                     |
| Data (settings, `history.db`, models, recordings) | `<app data>/com.dictus.desktop` | `<app data>/com.dictus.desktop.dev`          |
| Logs                                              | `<log dir>/com.dictus.desktop`  | `<log dir>/com.dictus.desktop.dev`           |
| Single instance                                   | its own lock                    | its own lock: runs next to the installed app |
| Autostart entry                                   | `Dictus`                        | `Dictus Dev`                                 |
| Updater                                           | checks GitHub releases          | disabled                                     |
| Focus at startup / relaunch                       | shows and focuses the window    | never takes focus (see below)                |

`<app data>` is `~/Library/Application Support` on macOS, `~/.local/share` on Linux and `%APPDATA%` on Windows.

The dev identity starts with empty data: download a model on first run. Copy files from the release folder if you need real data, never point the dev build at it.

On macOS, Accessibility and Microphone permissions are keyed on the bundle id, so Dictus Dev asks for them separately from Dictus. A bare `tauri dev` binary inherits the permissions of the terminal that launched it.

Pass `--config <file>` yourself to override the merge.

### No focus stealing

Dictus Dev never takes focus on its own, so a dev build can run while you keep working in another app:

- the main window is created unfocused (`focused: false`);
- when it would normally appear by itself (startup, the permission onboarding, or a relaunch that hands off to the running instance), it is ordered in **behind** other windows on macOS (`orderBack:`). It is not activated, not made key and not raised. On Linux and Windows it is shown unfocused;
- explicit actions still raise it as usual: the tray "Settings" item and clicking the Dock icon.

Set `DICTUS_DEV_NO_FOCUS=0` to get the release behaviour back in a dev build. Release builds (`com.dictus.desktop`) ignore the variable and are unchanged.

On macOS the window can stay invisible when the current Space is a full-screen app. Open it from the tray when you need it.

## Linux Install (from source)

> **Note:** The binary is named `handy` and Tauri bundle filenames use `Handy` — a full rename is planned for V2 (TECH-03).

The raw binary (`src-tauri/target/release/handy`) cannot run standalone — it needs Tauri resource files (tray icons, sounds, VAD model) to be co-located at the expected path.

**Install from the deb bundle** (works on any Linux distro):

```bash
cd /tmp
ar x /path/to/Handy/src-tauri/target/release/bundle/deb/Handy_*_amd64.deb data.tar.gz
tar xzf data.tar.gz
sudo cp usr/bin/handy /usr/bin/
sudo cp -r usr/lib/Handy /usr/lib/
sudo cp -r usr/share/icons/hicolor/* /usr/share/icons/hicolor/
sudo cp usr/share/applications/Handy.desktop /usr/share/applications/
```

After subsequent rebuilds, only the binary needs re-copying:

```bash
sudo cp src-tauri/target/release/handy /usr/bin/
```

Resources only need re-copying if they change upstream (new icons, sounds, etc.).

## Troubleshooting

### AppImage build fails on Arch / rolling-release distros

`linuxdeploy` bundles its own `strip` binary which is too old to process system libraries built with newer toolchains on rolling-release distros (Arch, CachyOS, Manjaro, EndeavourOS).

The error from Tauri:

```
Bundling Handy_*_amd64.AppImage
failed to bundle project `failed to run linuxdeploy`
```

Tauri swallows the real linuxdeploy error. To see it, run linuxdeploy manually:

```bash
cd src-tauri/target/release/bundle/appimage
~/.cache/tauri/linuxdeploy-x86_64.AppImage --appimage-extract-and-run \
  --appdir Handy.AppDir --plugin gtk --output appimage
```

**Workaround:** The binary, deb, and rpm bundles all build fine — only the AppImage step fails. To skip it:

```bash
bun run tauri build -- --bundles deb
```

Then install using the deb extraction method above.
