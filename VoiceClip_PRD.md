# VoiceClip PRD
**Version:** 1.1 — Refined for Implementation  
**Date:** February 2026

---

## Overview

VoiceClip is a lightweight, always-on voice-to-clipboard Linux desktop app targeting Wayland. The user activates a floating badge (or a global hotkey), speaks, and the transcription is immediately placed in the system clipboard. All processing is fully local using whisper.cpp — no audio leaves the machine.

---

## Goals

- Provide WisperFlow-style dictation entirely offline on Linux/Wayland
- Ship a draggable, always-on-top floating badge as the primary UX
- Persist usage statistics in a local SQLite database with a native stats window
- Use whisper.cpp (`whisper-cli` via subprocess) for transcription

## Non-Goals (v1.0)

- Cloud or hybrid transcription
- Windows or macOS support
- Real-time word-by-word streaming (full-utterance only)
- Text editing or correction UI inside the app
- Speaker diarization

---

## Language & Stack

**Rust** is the chosen language. It provides excellent memory safety, GTK4 bindings (`gtk-rs`), and a robust ecosystem for audio (`cpal`), database (`rusqlite`), and subprocess execution.

---

## Features

### 1. Floating Badge
- Small circular widget, 48×48 px default, always-on-top
- Draggable anywhere on screen; position persisted between sessions
- Single click → toggle recording (start/stop)
- Right-click context menu: Open Stats Window, Settings, Quit
- Semi-transparent when idle (opacity configurable, default 80%)
- Visual states:
  - **Idle** — grey, static
  - **Recording** — red, pulsing ring
  - **Processing** — blue, spinning arc
  - **Success** — green flash, then back to idle
  - **Error** — orange, shake animation

**Wayland layer surface:** Use `gtk4-layer-shell` to render the badge as a Wayland layer surface (`wlr-layer-shell` protocol). Fallback to a regular always-on-top window on non-supported compositors.

---

### 2. Recording & Transcription Pipeline
**Audio capture:**
- `cpal` (Rust crate) for default input device capture
- 16 kHz mono format, saved temporarily to a `.wav` file
- Toggle mode: click to start, click to stop. Max recording duration: 120s.

**Transcription:**
- Shell out to the system `whisper-cli` or a downloaded binary
- Plain text output parsing from standard output

**Clipboard delivery:**
- Write transcribed text via `wl-copy` (subprocess)
- Desktop notification on success/failure via `notify-send`

---

### 3. Global Hotkey
- Default: `Super+Shift+V`
- Implementation: `evdev` crate reading from `/dev/input` for Wayland-agnostic global shortcuts (user must be in `input` group).

---

### 4. Stats Window & Database (SQLite)
A GTK4 window with database-backed metrics.
- Uses `rusqlite` for database operations.
- Data logged: Session duration, transcription text, whisper model used, latency.
- UI built with GTK4 list boxes and basic layouts; charts implemented via the `plotters` crate drawn to a GTK canvas (or simple CSS-styled progress bars for minimal v1.0).

---

### 5. Settings & Model Manager
- Config saved via `serde` to `~/.config/voiceclip/config.toml`
- Models stored in `~/.local/share/voiceclip/models/`
- App defaults to fetching `base.en` via HTTP standard library bindings (`reqwest`) on first run if missing.

---

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                    VoiceClip (Rust)                 │
│                                                     │
│  ┌──────────┐   ┌────────────┐   ┌───────────────┐ │
│  │  Badge   │   │  Hotkey    │   │  Stats Window │ │
│  │  (GTK4)  │   │  (evdev)   │   │  (GTK4)       │ │
│  └────┬─────┘   └─────┬──────┘   └──────┬────────┘ │
│       │               │                 │          │
│       └───────────────┼─────────────────┘          │
│                       │                            │
│               ┌───────▼────────┐                   │
│               │ Tokio Async L. │                   │
│               └───────┬────────┘                   │
│                       │                            │
│        ┌──────────────┼──────────────┐             │
│        │              │              │             │
│  ┌─────▼────┐  ┌──────▼──────┐  ┌───▼──────────┐ │
│  │   cpal   │  │ whisper-cli │  │ rusqlite DB  │ │
│  │ (Audio)  │  │ (Process)   │  │              │ │
│  └──────────┘  └──────┬──────┘  └──────────────┘ │
│                       │                            │
│                 ┌─────▼─────┐                     │
│                 │  wl-copy  │                     │
│                 └───────────┘                     │
└─────────────────────────────────────────────────────┘
```

## Resolved Open Questions from Draft
1. **Whisper execution:** Use subprocess (`whisper-cli`) for v1.0 simplicity and robustness, rather than FFI bindings which require complex build environments.
2. **Audio backend:** Use `cpal` to abstract over ALSA/Pulse/PipeWire seamlessly.
3. **Charting:** Use the `plotters` crate if rich charts are needed, otherwise GTK native widgets (progress bars) for early iterations.
4. **License:** MIT License.
