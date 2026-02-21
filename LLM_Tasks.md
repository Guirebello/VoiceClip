# VoiceClip LLM Prompts & Tasks Breakdown

Use these prompts sequentially with your LLM (e.g., Claude 3.5 Sonnet, GPT-4o, or Gemini 1.5 Pro) in a single context window or across multiple sessions, appending the `VoiceClip_PRD.md` to the first prompt to provide context.

## Prerequisites
Before starting, ensure you provide the LLM with the complete refined `VoiceClip_PRD.md`.

---

### Task 1: Project Skeleton, Settings & Model Configuration
**Prompt:**
> Please review the `VoiceClip_PRD.md`. We are going to build this Rust application step-by-step.
> 
> For Task 1, please generate the `Cargo.toml` with the necessary dependencies: `tokio`, `cpal` (audio), `hound` (wav writing), `rusqlite` (database), `gtk4`, `gtk4-layer-shell`, `serde`, `toml`, `directories`, and any standard error handling crates like `anyhow`. 
> 
> Next, implement the core configuration logic in `src/config.rs`. We need a `Config` struct that serializes/deserializes to `~/.config/voiceclip/config.toml` via the `directories` crate. It should include fields for `model_name` (default "base.en"), `hotkey` (default "Super+Shift+V"), `badge_opacity` (0.8), and `max_recording_duration` (120). Finally, provide a basic `main.rs` that loads this config and prints it.

---

### Task 2: Audio Capture & File Saving
**Prompt:**
> For Task 2, let's implement the audio capture pipeline in `src/audio.rs`.
> 
> We need a function `start_recording` that uses `cpal` to connect to the default input device. It should capture audio at 16 kHz, mono format (required by whisper), and buffer it in memory using a crossbeam channel or standard mpsc.
> 
> Also write a `stop_recording_and_save` function that takes the buffered `f32` or `i16` samples and uses the `hound` crate to write them to `/tmp/voiceclip_audio.wav`. Make sure this is wrapped in a neat async-friendly API so we can trigger it from our future GTK/Hotkey event loops. Please show how to test this quickly from `main.rs`.

---

### Task 3: Whisper-CLI Subprocess
**Prompt:**
> For Task 3, let's implement `src/whisper.rs`. 
> 
> Write an async function `transcribe(wav_path: &Path, model_path: &Path) -> Result<String>`. 
> It should spawn the `whisper-cli` executable as a subprocess using `tokio::process::Command`. 
> Pass the required arguments: model (`-m`), input file (`-f`), output text only (`--output-txt`), no timestamps (`--no-timestamps`), and language (`-l auto`). 
> Wait for the process to finish, then read the generated output/text and return it as a cleaned-up `String`. Include robust error handling if the binary is missing or fails.

---

### Task 4: Clipboard & Notifications Delivery
**Prompt:**
> For Task 4, implement the results delivery in `src/delivery.rs`.
> 
> Create a function `copy_to_clipboard(text: &str)` that shells out to `wl-copy`. If the user has "append mode" enabled, it should first read the clipboard (`wl-paste`), append the new text, and write it back.
> 
> Next, create a `notify(title: &str, body: &str, is_error: bool)` function that uses the system `notify-send` command to show a desktop notification when transcription is completely successful or if an error occurs. 

---

### Task 5: Database & Session Logging
**Prompt:**
> For Task 5, let's set up the SQLite tracking inside `src/db.rs`.
> 
> Create a `Database` struct using `rusqlite`. On initialization, it should create a `~/.local/share/voiceclip/voiceclip.db` file and execute an initialization SQL script to create the `sessions` table (as defined in the PRD: `id`, `started_at`, `duration`, `word_count`, `model_used`, `transcription`, `latency`, `error`). 
> 
> Write a method `pub fn log_session(&self, session: SessionRecord) -> Result<()>` to insert a completed transcription event. Connect this mock logic to `main.rs` to show it working end-to-end (Record -> Transcribe -> wl-copy -> DB log).

---

### Task 6: Wayland GTK4 Badge Foundation
**Prompt:**
> For Task 6, it is time for the primary UI. Let's create `src/ui/badge.rs`.
> 
> Set up a standard `gtk4` application in `main.rs` that spawns a small 48x48 borderless window. 
> Use the `gtk4-layer-shell` crate to initialize the window as a Wayland layer surface (`init_for_window`). 
> Set the anchor properties so it floats on top, and make it draggable by listening to pointer drag events to move the layer shell margins or position.
> Include a basic circular CSS styling with an idle state (grey, 80% opacity). Note: just the visual foundation for now, no audio hooks yet.

---

### Task 7: Badge Visual States & Context Menu
**Prompt:**
> For Task 7, expand `src/ui/badge.rs`.
> 
> Add GTK state management for the badge: `Idle`, `Recording` (make it turn red and pulse via CSS animations/timeouts), `Processing` (blue), `Success` (green flash), and `Error` (orange shake).
> Also attach a GTK popover / context menu on right-click that has three options: "Stats", "Settings", and "Quit". 
> Wire up the Left-Click event to emit a signal or message over a channel that will eventually trigger the audio recording toggle.

---

### Task 8: Global Hotkey Daemon
**Prompt:**
> For Task 8, we need the background global shortcut. Create `src/hotkey.rs`.
> 
> We will use the `evdev` crate to read raw input events from `/dev/input/`. Create a background Tokio task that opens the keyboard device, listens for the configured hotkey (e.g., `Super+Shift+V`), and when pressed, sends a message over an `mpsc::channel` to our main application loop to toggle recording. 
> Keep in mind this needs to handle keydown and keyup states securely.

---

### Task 9: Integrating the Application Loop
**Prompt:**
> For Task 9, we need to wire everything together in `src/app.rs` or `main.rs`.
> 
> Set up an `mpsc` channel for global application events (e.g., `Event::ToggleRecording`, `Event::RecordingFinished(audio_path)`, `Event::TranscriptionDone(Result)`).
> Bridge the GTK4 UI, the Evdev hotkey task, the async recording task, and the Whisper transcription task.
> Make sure clicking the badge OR pressing the hotkey starts recording, updates the badge to Red, stops on second click, turns badge Blue, transcibes, copies to clipboard, turns badge Green, and logs to SQLite.

---

### Task 10: Stats Window (GTK4)
**Prompt:**
> For our final Task 10, implement the Stats Window in `src/ui/stats.rs`.
> 
> This is a normal GTK4 Application Window that is spawned when "Stats" is clicked in the badge context menu.
> Read the SQLite `rusqlite` database and calculate: Total recordings, total minutes, average words per session. 
> Display these metrics cleanly in a GTK Grid or Box layout. 
> Add a `gtk::ListView` or `gtk::ListBox` below the summary to show the recent transcription history.

---
**You are now fully set up to pipe this project piece by piece!**
