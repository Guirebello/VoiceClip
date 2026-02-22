# VoiceClip User Guide

VoiceClip is a lightweight, always-on voice-to-clipboard desktop application. Press a hotkey, speak, press again — your speech is transcribed locally via Whisper and placed directly in your clipboard. No audio ever leaves your machine.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Usage](#usage)
5. [Stats Window](#stats-window)
6. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Both Platforms

- **Node.js** (v18+) — install from [nodejs.org](https://nodejs.org) or via a version manager like nvm
- **Rust toolchain** (stable) — install via [rustup.rs](https://rustup.rs)
- **whisper-cli** — the CLI binary from the [whisper.cpp](https://github.com/ggerganov/whisper.cpp) project, available in your system `PATH`
- **A Whisper model file** — e.g. `ggml-base.en.bin`, placed in the models directory (see [Configuration](#configuration))

### Linux

```bash
# WebKitGTK and Tauri system dependencies
sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev librsvg2-dev

# ALSA headers for audio capture
sudo apt install libasound2-dev

# Add your user to the input group (required for global hotkey detection)
sudo usermod -aG input $USER
# Log out and back in for the group change to take effect
```

### Windows

- **Rust toolchain** — install via [rustup.rs](https://rustup.rs) with the default MSVC target (`x86_64-pc-windows-msvc`)
- **Node.js** — install from [nodejs.org](https://nodejs.org)
- **WebView2** — pre-installed on Windows 10 (version 1803+) and Windows 11. If missing, download from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- **whisper-cli** — build or download `whisper-cli.exe` from the [whisper.cpp](https://github.com/ggerganov/whisper.cpp) project and place it in your system `PATH`

> **Note:** Unlike the previous GTK4 build, no MSYS2 or MinGW toolchain is needed. Tauri uses the native Windows WebView2 runtime.

---

## Installation

### Linux

```bash
git clone https://github.com/Guirebello/VoiceClip.git
cd VoiceClip
npm install
npm run tauri build
```

The binary will be at `src-tauri/target/release/VoiceClip`.

For development with hot-reload:
```bash
npm run tauri dev
```

### Windows

```powershell
git clone https://github.com/Guirebello/VoiceClip.git
cd VoiceClip
npm install
npm run tauri build
```

The binary will be at `src-tauri\target\release\VoiceClip.exe`.

### Download a Whisper model

Download a GGML model from the whisper.cpp project and place it in the models directory:

| Platform | Models directory |
|----------|-----------------|
| Linux    | `~/.local/share/voiceclip/models/` |
| Windows  | `%APPDATA%\voiceclip\data\models\` |

For example, to use the `base.en` model:

**Linux:**
```bash
mkdir -p ~/.local/share/voiceclip/models
cp ggml-base.en.bin ~/.local/share/voiceclip/models/base.en
```

**Windows (PowerShell):**
```powershell
mkdir "$env:APPDATA\voiceclip\data\models" -Force
copy ggml-base.en.bin "$env:APPDATA\voiceclip\data\models\base.en"
```

The filename must match the `model_name` value in your config (default: `base.en`).

---

## Configuration

On first run, VoiceClip creates a default configuration file:

| Platform | Config path |
|----------|-------------|
| Linux    | `~/.config/voiceclip/config.toml` |
| Windows  | `%APPDATA%\voiceclip\config\config.toml` |

### Default config

```toml
model_name = "base.en"
hotkey = "Super+Alt+V"
badge_opacity = 0.8
max_recording_duration = 120
append_mode = false
always_on_top = true
```

### Options

| Key | Type | Description |
|-----|------|-------------|
| `model_name` | string | Name of the Whisper model file in the models directory |
| `hotkey` | string | Global hotkey combo (default `Super+Alt+V`). Set to `None` to disable. Can be changed via right-click → Settings. |
| `badge_opacity` | float | Opacity of the floating badge (0.0–1.0) |
| `max_recording_duration` | integer | Maximum recording length in seconds |
| `append_mode` | bool | If `true`, new transcriptions are appended to existing clipboard text instead of replacing it |
| `microphone` | string or null | Name of the input device to use. Omit or set to `null` to use the system default. Selectable via Settings. |
| `always_on_top` | bool | If `true` (default), the badge stays above all other windows |
| `badge_x` | integer or null | Saved X position of the badge window (set automatically when you drag the badge) |
| `badge_y` | integer or null | Saved Y position of the badge window (set automatically when you drag the badge) |

### Key directories

| Directory | Linux | Windows |
|-----------|-------|---------|
| Config | `~/.config/voiceclip/` | `%APPDATA%\voiceclip\config\` |
| Data | `~/.local/share/voiceclip/` | `%APPDATA%\voiceclip\data\` |
| Models | `~/.local/share/voiceclip/models/` | `%APPDATA%\voiceclip\data\models\` |
| Database | `~/.local/share/voiceclip/voiceclip.db` | `%APPDATA%\voiceclip\data\voiceclip.db` |

---

## Usage

### Starting VoiceClip

**Linux:**
```bash
./src-tauri/target/release/VoiceClip
```

**Windows:**
```powershell
.\src-tauri\target\release\VoiceClip.exe
```

A small circular badge appears in the bottom-right corner of your screen. This badge is your primary visual indicator.

### Recording workflow

1. **Press `Super+Alt+V`** (or left-click the badge) to start recording.
   The badge turns **orange-red**.

2. **Speak** into your microphone.

3. **Press `Super+Alt+V` again** (or left-click the badge) to stop.
   The badge turns **blue** while Whisper transcribes your audio.

4. When transcription completes, the text is copied to your clipboard and a desktop notification appears.
   The badge turns **green** for 3 seconds, then returns to grey.

5. **Paste** (`Ctrl+V`) anywhere to use the transcribed text.

### Badge states

| Color | State | Meaning |
|-------|-------|---------|
| Grey | Idle | Ready to record |
| Orange-red | Recording | Microphone is active |
| Blue | Processing | Whisper is transcribing |
| Green | Success | Text copied to clipboard |
| Red | Error | Something went wrong |

### Badge interactions

| Action | Effect |
|--------|--------|
| **Left-click** | Toggle recording on/off |
| **Right-click** | Open context menu (Stats, Settings, Quit) |
| **Drag** | Move the badge around the screen |

### Settings

Right-click the badge and select **Settings** to configure:

- **Hotkey** — change the global hotkey combo or set to `None` to disable
- **Microphone** — select which input device to use for recording
- **Always on Top** — toggle whether the badge stays above all other windows
- **Badge Opacity** — adjust the transparency of the floating badge

Changes are saved to `config.toml` and take effect immediately (hotkey changes require a restart).

### Append mode

When `append_mode = true` in your config, each new transcription is appended to the current clipboard content (separated by a space) rather than replacing it. This is useful for dictating long passages across multiple recordings.

---

## Stats Window

Right-click the badge and select **Stats** to open the statistics window.

### Summary metrics (top)

- **Total Recordings** — number of successful transcription sessions
- **Total Minutes** — cumulative recording time
- **Avg Words/Session** — average word count per transcription

### Session history (scrollable list)

Each row shows:
```
2026-02-21 14:30:05 | 12s | 45 words | 1200ms latency
The quick brown fox jumped over the lazy dog and then proceeded to...
```

- Failed sessions appear in **red** with the error message displayed
- Up to 50 most recent sessions are shown

---

## Troubleshooting

### Linux

**Hotkey not working**
- Make sure your user is in the `input` group: `groups $USER`
- If not, run `sudo usermod -aG input $USER` and log out/in
- Check that `/dev/input/` devices are readable: `ls -la /dev/input/event*`

**Badge not appearing or positioned incorrectly (Wayland)**
- On Wayland, window positioning behavior depends on your compositor. Some compositors may ignore the requested position or place the window differently.
- You may need to add a compositor rule for the VoiceClip window. For example, in Hyprland: `windowrulev2 = float,class:^(VoiceClip)$`
- On GNOME/Mutter, the badge appears as a regular floating window.

**"Failed to spawn whisper-cli"**
- Ensure `whisper-cli` is installed and in your `PATH`: `which whisper-cli`
- If installed elsewhere, you can symlink it: `sudo ln -s /path/to/whisper-cli /usr/local/bin/whisper-cli`

**No audio / wrong microphone**
- Open Settings (right-click badge → Settings) and select your preferred microphone from the dropdown.
- If the dropdown is empty, check that your audio server (PulseAudio/PipeWire) is running and devices are available.

**Build fails with missing WebKitGTK**
- Install the required Tauri dependencies: `sudo apt install libwebkit2gtk-4.1-dev libjavascriptcoregtk-4.1-dev librsvg2-dev`

### Windows

**"Failed to spawn whisper-cli" / whisper-cli not found**
- Ensure `whisper-cli.exe` is in your system PATH. Test by running `whisper-cli --help` in a terminal.

**Hotkey not working**
- If another application has already registered the same hotkey, VoiceClip will print a warning and continue without a hotkey — you can still use badge clicks to record.
- To change the hotkey, right-click the badge → **Settings**, enter a new combo (e.g. `Ctrl+Shift+R`), save, and restart.
- You can also set the hotkey to `None` in Settings to disable it entirely.
- Try running VoiceClip as Administrator if the hotkey still doesn't respond.

**Clipboard permission issues**
- Windows may prompt for clipboard access on first use. Allow it.

**WebView2 missing**
- WebView2 is pre-installed on Windows 10 (1803+) and Windows 11. If VoiceClip fails to start, download the WebView2 runtime from [Microsoft](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

### Both platforms

**"Model file not found"**
- Download a GGML model and place it in the models directory with the exact name matching your `model_name` config value.
- Verify: the full path should be `<models_dir>/<model_name>` (e.g. `~/.local/share/voiceclip/models/base.en`).

**Transcription quality is poor**
- Use a larger model (e.g. `small.en`, `medium.en`) for better accuracy at the cost of slower processing.
- Ensure your microphone input level is adequate and background noise is minimal.
- The audio is recorded at 16 kHz mono, which is optimal for Whisper.

**Database errors**
- The SQLite database is created automatically on first run. If corrupted, delete `voiceclip.db` from the data directory and restart — a new one will be created (session history will be lost).
