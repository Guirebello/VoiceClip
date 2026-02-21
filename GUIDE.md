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

VoiceClip on Windows relies on [MSYS2](https://www.msys2.org/), a software distribution that provides a Unix-like build environment and a collection of native Windows libraries. MSYS2 uses a package manager called **pacman** (the same one used by Arch Linux) to install pre-built libraries like GTK4.

**Step 1 — Install MSYS2**

Download and run the installer from [msys2.org](https://www.msys2.org/). Accept the default install location (`C:\msys64`).

**Step 2 — Install GTK4 and build dependencies**

Open the **MINGW64** shell (find "MSYS2 MINGW64" in your Start menu — do *not* use the plain "MSYS2" or "UCRT64" shells). Then run:

```bash
pacman -Syu
pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-pkg-config mingw-w64-x86_64-toolchain
```

The first command updates the package database; the second installs GTK4, pkg-config, and the MinGW GCC toolchain.

**Step 3 — Add MSYS2 to your system PATH**

Add `C:\msys64\mingw64\bin` to your Windows **system** PATH. This is required both for building and for running VoiceClip (GTK4 DLLs live here).

1. Press `Win+R`, type `sysdm.cpl`, press Enter.
2. Go to **Advanced** → **Environment Variables**.
3. Under **System variables**, select `Path` and click **Edit**.
4. Click **New** and add `C:\msys64\mingw64\bin`.
5. Click **OK** to save. Open a **new** terminal for the change to take effect.

**Step 4 — Install Rust with the GNU target**

If you haven't installed Rust yet, install it via [rustup.rs](https://rustup.rs). During setup, choose the `x86_64-pc-windows-gnu` default host triple.

If you already have Rust installed with the default MSVC target, switch to the GNU target:

```powershell
rustup target add x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
```

**Step 5 — Install whisper-cli**

Build or download `whisper-cli.exe` from the [whisper.cpp](https://github.com/ggerganov/whisper.cpp) project and place it somewhere in your system `PATH` (e.g. `C:\msys64\mingw64\bin`).

> **Note:** If you prefer [vcpkg](https://vcpkg.io/) over MSYS2 for managing C/C++ libraries, you can install GTK4 through vcpkg instead. However, MSYS2 is the tested and recommended approach.

---

## Installation

### Linux

```bash
git clone https://github.com/your-username/VoiceClip.git
cd VoiceClip
cargo build --release
```

The binary will be at `target/release/VoiceClip`.

### Windows

> **Important:** Run `cargo build` from a regular terminal (PowerShell or CMD), *not* from the MSYS2 shell. The MSYS2 shell is only needed for installing packages with pacman.

```powershell
git clone https://github.com/your-username/VoiceClip.git
cd VoiceClip
cargo build --release
```

The binary will be at `target\release\VoiceClip.exe`.

If you get a `pkg-config not found` or `GTK4 not found` error, double-check that `C:\msys64\mingw64\bin` is in your system PATH and that you opened a **new** terminal after adding it (see [Prerequisites > Windows](#windows)).

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

**Linux:**
```bash
./target/release/VoiceClip
```

**Windows (PowerShell or CMD — not the MSYS2 shell):**
```powershell
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

**`cargo build` fails with "pkg-config not found" or "GTK4 not found"**
- Make sure you installed the MSYS2 packages from [Prerequisites > Windows](#windows).
- Verify `C:\msys64\mingw64\bin` is in your system PATH (not just user PATH).
- **Open a new terminal** after editing PATH — existing terminals won't see the change.
- Run `pkg-config --modversion gtk4` in your terminal. If it prints a version number (e.g. `4.14.1`), GTK4 is correctly found.

**GTK4 errors or missing DLL on startup**
- This almost always means `C:\msys64\mingw64\bin` is not in your PATH. VoiceClip needs the GTK4 runtime DLLs at launch.
- After adding it to PATH, **close and reopen your terminal** (or restart your IDE) for the change to take effect.

**"Failed to spawn whisper-cli" / whisper-cli not found**
- Ensure `whisper-cli.exe` is in your system PATH. Test by running `whisper-cli --help` in a terminal.
- A convenient location is `C:\msys64\mingw64\bin` since that directory is already in your PATH.

**Hotkey not working**
- Windows Clipboard History uses `Win+V` by default, which can interfere with VoiceClip's `Win+Shift+V`. If you experience conflicts, disable Clipboard History in **Settings > System > Clipboard** or change its shortcut.
- Some other applications may also register `Win+Shift+V`. Close conflicting apps and restart VoiceClip.
- Try running VoiceClip as Administrator if the hotkey still doesn't respond.

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
