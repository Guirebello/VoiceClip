# VoiceClip

A lightweight, always-on voice-to-clipboard desktop app. Press a hotkey, speak, press again — your speech is transcribed locally via [Whisper](https://github.com/ggerganov/whisper.cpp) and placed directly in your clipboard. No audio ever leaves your machine.

## Features

- **Global hotkey** (`Win+Alt+V` / `Super+Alt+V`) to toggle recording from anywhere
- **Local transcription** — powered by whisper.cpp, everything stays on your device
- **Floating badge** — a small draggable indicator shows recording/processing/success state
- **Configurable hotkey** — change or disable via right-click badge > Settings
- **Graceful fallback** — if the hotkey is taken by another app, VoiceClip continues without it (badge clicks still work)
- **Append mode** — optionally append new transcriptions to existing clipboard text
- **Session stats** — track recording history, word counts, and latency
- **Cross-platform** — Linux (X11/Wayland) and Windows

## Quick Start

### Download

Grab the latest release from the [Releases](../../releases) page:

- **Linux**: `VoiceClip-linux-x86_64.tar.gz` — extract and run `./VoiceClip`
- **Windows**: `VoiceClip-windows-x86_64.zip` — extract and run `VoiceClip.exe`

Both packages include `whisper-cli` and the `base.en` model, so they work out of the box.

### Build from Source

```bash
git clone https://github.com/Guirebello/VoiceClip.git
cd VoiceClip
cargo build --release
```

See the [User Guide](GUIDE.md) for full prerequisites and platform-specific setup.

## Usage

1. Launch VoiceClip — a small grey badge appears on screen
2. Press **Super+Alt+V** (or click the badge) to start recording
3. Speak into your microphone
4. Press **Super+Alt+V** again (or click the badge) to stop
5. Your transcription is copied to the clipboard — paste it anywhere with `Ctrl+V`

### Badge Colors

| Color | Meaning |
|-------|---------|
| Grey | Idle — ready to record |
| Orange-red | Recording |
| Blue | Processing (transcribing) |
| Green | Success — text copied |
| Red | Error |

### Right-Click Menu

- **Stats** — view recording history and metrics
- **Settings** — change the global hotkey
- **Quit** — exit VoiceClip

## Configuration

Config file location:

| Platform | Path |
|----------|------|
| Linux | `~/.config/voiceclip/config.toml` |
| Windows | `%APPDATA%\voiceclip\config\config.toml` |

```toml
model_name = "base.en"
hotkey = "Super+Alt+V"
badge_opacity = 0.8
max_recording_duration = 120
append_mode = false
```

Set `hotkey = "None"` to disable the global hotkey (badge clicks still work).

See the [User Guide](GUIDE.md) for full configuration details, model setup, and troubleshooting.

## Documentation

- **[User Guide](GUIDE.md)** — prerequisites, installation, configuration, usage, and troubleshooting

## License

MIT
