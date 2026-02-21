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

- **Rust toolchain** (stable) — install via [rustup.rs](https://rustup.rs)
- **whisper-cli** — the CLI binary from the [whisper.cpp](https://github.com/ggerganov/whisper.cpp) project, available in your system `PATH`
- **A Whisper model file** — e.g. `ggml-base.en.bin`, placed in the models directory (see [Configuration](#configuration))

### Linux

```bash
# GTK4 and layer-shell development libraries
sudo apt install libgtk-4-dev libgtk4-layer-shell-dev

# ALSA headers for audio capture
sudo apt install libasound2-dev

# Add your user to the input group (required for global hotkey detection)
sudo usermod -aG input $USER
# Log out and back in for the group change to take effect
```

### Windows

- **MSYS2** with the GTK4 package, or **vcpkg** with GTK4 support:
  ```
  # MSYS2 (recommended)
  pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-pkg-config
  ```
- **Visual Studio Build Tools** or the MSYS2 MinGW toolchain for compiling native dependencies
- Ensure `whisper-cli.exe` is in your `PATH`

---

## Installation

```bash
git clone https://github.com/your-username/VoiceClip.git
cd VoiceClip
cargo build --release
```

The binary will be at:
- **Linux:** `target/release/VoiceClip`
- **Windows:** `target\release\VoiceClip.exe`

### Download a Whisper model

Download a GGML model from the whisper.cpp project and place it in the models directory:

| Platform | Models directory |
|----------|-----------------|
| Linux    | `~/.local/share/voiceclip/models/` |
| Windows  | `%APPDATA%\voiceclip\data\models\` |

For example, to use the `base.en` model on Linux:

```bash
mkdir -p ~/.local/share/voiceclip/models
cp ggml-base.en.bin ~/.local/share/voiceclip/models/base.en
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
hotkey = "Super+Shift+V"
badge_opacity = 0.8
max_recording_duration = 120
append_mode = false
```

### Options

| Key | Type | Description |
|-----|------|-------------|
| `model_name` | string | Name of the Whisper model file in the models directory |
| `hotkey` | string | Global hotkey combo (currently hardcoded to `Super+Shift+V`) |
| `badge_opacity` | float | Opacity of the floating badge (0.0–1.0) |
| `max_recording_duration` | integer | Maximum recording length in seconds |
| `append_mode` | bool | If `true`, new transcriptions are appended to existing clipboard text instead of replacing it |

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

```bash
# Linux
./target/release/VoiceClip

# Windows
.\target\release\VoiceClip.exe
```

A small circular badge appears in the bottom-right corner of your screen. This badge is your primary visual indicator.

### Recording workflow

1. **Press `Super+Shift+V`** (or left-click the badge) to start recording.
   The badge turns **orange-red**.

2. **Speak** into your microphone.

3. **Press `Super+Shift+V` again** (or left-click the badge) to stop.
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

**Badge not appearing (Wayland)**
- VoiceClip uses `gtk4-layer-shell` for Wayland compositors. Make sure your compositor supports the `wlr-layer-shell` protocol (Sway, Hyprland, and most wlroots-based compositors do).
- On GNOME/Mutter, layer-shell support may be limited — the badge may appear as a regular window.

**"Failed to spawn whisper-cli"**
- Ensure `whisper-cli` is installed and in your `PATH`: `which whisper-cli`
- If installed elsewhere, you can symlink it: `sudo ln -s /path/to/whisper-cli /usr/local/bin/whisper-cli`

**No audio / wrong microphone**
- VoiceClip uses the system default input device. Change your default microphone in your system audio settings (e.g. `pavucontrol` on PulseAudio/PipeWire).

### Windows

**Hotkey not working**
- Some applications or the OS may already have `Win+Shift+V` registered. Close conflicting apps and restart VoiceClip.
- Run VoiceClip as Administrator if the hotkey still doesn't respond.

**GTK4 errors on startup**
- Ensure GTK4 runtime DLLs are in your `PATH`. If using MSYS2, add `C:\msys64\mingw64\bin` to your system `PATH`.

**Clipboard permission issues**
- Windows may prompt for clipboard access on first use. Allow it.

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
