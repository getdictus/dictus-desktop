<p align="center">
  <img src="https://raw.githubusercontent.com/getdictus/dictus-brand/main/source/appicon-light.svg" alt="Dictus" width="120" height="120" />
</p>

<h1 align="center">Dictus Desktop</h1>

<p align="center">
  <strong>100% offline voice dictation for macOS, Windows & Linux.</strong><br />
  Press a shortcut, speak, and your words appear in any app — entirely on your machine.
</p>

<p align="center">
  <a href="https://github.com/getdictus/dictus-desktop/actions"><img src="https://img.shields.io/github/actions/workflow/status/getdictus/dictus-desktop/build.yml?branch=main&label=build" alt="Build" /></a>
  <a href="https://github.com/getdictus/dictus-desktop/releases/latest"><img src="https://img.shields.io/github/v/release/getdictus/dictus-desktop?include_prereleases&label=release" alt="Latest release" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/getdictus/dictus-desktop" alt="License" /></a>
  <a href="https://github.com/getdictus/dictus-desktop/stargazers"><img src="https://img.shields.io/github/stars/getdictus/dictus-desktop?style=social" alt="Stars" /></a>
</p>

<p align="center">
  <a href="https://getdictus.com">Website</a> ·
  <a href="https://github.com/getdictus/dictus-desktop/releases/latest">Download</a> ·
  <a href="https://github.com/getdictus/dictus-ios">iOS</a> ·
  <a href="https://github.com/getdictus/dictus-android">Android</a> ·
  <a href="https://t.me/getdictus">Community</a>
</p>

---

## Why Dictus Desktop?

- 🔒 **100% offline** — no network calls, no cloud, no telemetry. Your voice never leaves your machine.
- 🆓 **Free & open source** — MIT licensed, no subscription, no account required.
- ⚡ **Fast & accurate** — Whisper (GPU-accelerated) or Parakeet V3 (CPU, ~5× realtime).
- 🖥 **Cross-platform** — macOS (Apple Silicon & Intel), Windows, Linux.
- ⌨️ **Anywhere you type** — global shortcut pastes transcription into the focused app.

## Quick start

1. **Download** the latest build from [Releases](https://github.com/getdictus/dictus-desktop/releases/latest) (macOS `.dmg`, Windows `.msi`, Linux `.AppImage` / `.deb`).
2. **Launch** Dictus Desktop and grant microphone + accessibility permissions.
3. **Configure** your shortcut in Settings (default works out of the box).
4. **Press, speak, release** — your words land in any text field.

> Building from source? See [BUILD.md](BUILD.md).

## How it works

1. Press a configurable shortcut to start/stop recording (or push-to-talk).
2. Speak — Silero VAD filters silence in real time.
3. Whisper or Parakeet transcribes locally — GPU when available, CPU otherwise.
4. Transcribed text is pasted into the focused application.

## Privacy

Dictus runs locally by default. Transcription never leaves your device. Cloud post-processing is OFF by default and opt-in. For the complete list of every outbound network endpoint the app can contact (with how to disable each), see [`docs/PRIVACY.md`](docs/PRIVACY.md).

## How Dictus compares

| Feature          |                                              **Dictus Desktop**                                              |     SuperWhisper     |         Wispr Flow          |      MacWhisper      |
| ---------------- | :----------------------------------------------------------------------------------------------------------: | :------------------: | :-------------------------: | :------------------: |
| Price            |                                                   **Free**                                                   |   Free / $8.49/mo    |        Free / $15/mo        |   Free / $6.99/mo    |
| 100% offline     |                                                      ✅                                                      |          ⚠️          |             ❌              |          ⚠️          |
| Privacy-first    |                                                      ✅                                                      |          ⚠️          |             ❌              |          ⚠️          |
| Open source      |                                                      ✅                                                      |          ❌          |             ❌              |          ❌          |
| Platforms        |                                           macOS · Windows · Linux                                            |  iOS · macOS · Win   | iOS · macOS · Win · Android |     iOS · macOS      |
| Mobile companion | ✅ ([iOS](https://github.com/getdictus/dictus-ios) · [Android](https://github.com/getdictus/dictus-android)) | macOS-only ecosystem |           partial           | macOS-only ecosystem |

## Models

| Model                                  | CPU | GPU | Notes                                                |
| -------------------------------------- | :-: | :-: | ---------------------------------------------------- |
| Whisper Small / Medium / Turbo / Large |  ✓  | ⭐  | GPU recommended for Medium+                          |
| Parakeet V3                            | ⭐  |  —  | CPU-optimized, ~5× realtime, auto language detection |

## Architecture

Dictus Desktop is a Tauri app: a React + TypeScript settings UI on top of a Rust core.

- `whisper-rs` — Whisper inference
- `transcribe-rs` — Parakeet V3 inference
- `cpal` — cross-platform audio capture
- `vad-rs` — Silero Voice Activity Detection
- `rdev` / OS APIs — global shortcuts and paste

## Roadmap

- [x] macOS (Apple Silicon + Intel), Windows x64, Linux x64
- [x] Whisper + Parakeet V3 engines
- [x] CLI flags & signal-based control
- [ ] Smart Mode Pro — local LLM reformulation
- [ ] Custom vocabulary (technical terms, names)
- [ ] Audio-file transcription
- [ ] Searchable local history
- [ ] Sync settings across Dictus iOS / Android / Desktop (offline-first, peer-to-peer)

Open an issue with a [feature request](https://github.com/getdictus/dictus-desktop/issues/new) — we prioritize the most-upvoted ideas.

## Platform notes

<details>
<summary><strong>Linux</strong> — text input tools & global shortcuts</summary>

For reliable text input, install the right tool for your display server:

| Display Server | Tool      | Install                                               |
| -------------- | --------- | ----------------------------------------------------- |
| X11            | `xdotool` | `sudo apt install xdotool`                            |
| Wayland        | `wtype`   | `sudo apt install wtype`                              |
| Both           | `dotool`  | `sudo apt install dotool` (add user to `input` group) |

**Runtime library:** if startup fails with `libgtk-layer-shell.so.0`, install your distro's package (`libgtk-layer-shell0` on Debian/Ubuntu, `gtk-layer-shell` on Fedora/Arch).

**Wayland shortcuts:** must be configured at the desktop-environment level. Use the CLI flags below as the command for your custom binding:

```bash
handy --toggle-transcription    # toggle recording
handy --toggle-post-process     # toggle with post-processing
handy --cancel                  # cancel current operation
```

Sway / i3 example:

```ini
bindsym $mod+o exec handy --toggle-transcription
```

Or via Unix signals (works with any hotkey daemon):

```bash
pkill -USR2 -n handy   # toggle transcription
pkill -USR1 -n handy   # toggle with post-processing
```

</details>

<details>
<summary><strong>macOS</strong> — CLI usage when installed as a bundle</summary>

```bash
/Applications/Dictus.app/Contents/MacOS/handy --toggle-transcription
```

</details>

<details>
<summary><strong>CLI flags</strong></summary>

```bash
handy --toggle-transcription    # Toggle recording
handy --toggle-post-process     # Toggle recording with post-processing
handy --cancel                  # Cancel current operation
handy --start-hidden            # Start without showing the main window
handy --no-tray                 # Start without the system tray icon
handy --debug                   # Enable verbose logging
handy --help                    # Show all flags
```

> The binary is currently named `handy` (a V2 rename is planned).

</details>

## System requirements

**Whisper (GPU recommended for Medium+):**

- macOS — Apple Silicon or Intel
- Windows — Intel / AMD / NVIDIA GPU
- Linux — Intel / AMD / NVIDIA GPU (Ubuntu 22.04 / 24.04 tested)

**Parakeet V3 (CPU-only):**

- Intel Skylake (6th gen) / AMD equivalent or newer

## Known issues

This project is actively developed. See [open issues](https://github.com/getdictus/dictus-desktop/issues) for current state. Major items:

- Whisper crashes on some Windows / Linux configurations (help wanted — debug logs welcome).
- Wayland support is partial — requires `wtype` or `dotool` (see Linux notes).

## Contributing

Contributions are very welcome. Please:

1. Browse [open issues](https://github.com/getdictus/dictus-desktop/issues) — `good first issue` and `help wanted` are good starts.
2. Fork, branch, and submit a PR with a clear description.
3. Test on your target platform before requesting review.
4. Join the conversation on [Telegram](https://t.me/getdictus) or via [hello@getdictus.com](mailto:hello@getdictus.com).

## Support the project

Dictus is free and will stay free. If it saves you time, consider [supporting development](https://getdictus.com/donate) — it directly funds new features and platform support.

## Community

- 🌐 [getdictus.com](https://getdictus.com)
- 💬 [Telegram](https://t.me/getdictus)
- 🐛 [Issues](https://github.com/getdictus/dictus-desktop/issues)
- 📧 [hello@getdictus.com](mailto:hello@getdictus.com)

## License

MIT — see [LICENSE](LICENSE).

## Acknowledgments

Dictus Desktop is a fork of **[Handy](https://github.com/cjpais/Handy)** by [cjpais](https://github.com/cjpais) — huge thanks for the foundation.

- [OpenAI Whisper](https://github.com/openai/whisper) — speech recognition model
- [whisper.cpp](https://github.com/ggerganov/whisper.cpp) & ggml — cross-platform inference
- [Silero VAD](https://github.com/snakers4/silero-vad) — voice activity detection
- [Tauri](https://tauri.app) — Rust-based app framework

---

<p align="center">
  <sub>Made with ❤️ by <a href="https://pivi.solutions">PIVI Solutions</a> · <a href="https://github.com/getdictus">@getdictus</a></sub>
</p>
