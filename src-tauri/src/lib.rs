mod config;
mod audio;
mod whisper;
mod delivery;
mod db;
mod hotkey;

use config::Config;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};

#[derive(Debug)]
pub enum AppEvent {
    ToggleRecording,
}

enum AudioCommand {
    Start(Option<String>),
    StopAndSave(String, tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    GetLevel(tokio::sync::oneshot::Sender<f32>),
}

struct AppState {
    config: Mutex<Config>,
    db: Mutex<db::Database>,
    event_tx: tokio::sync::mpsc::Sender<AppEvent>,
}

#[tauri::command]
async fn toggle_recording(state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.event_tx.send(AppEvent::ToggleRecording).await
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
fn save_config(state: tauri::State<'_, AppState>, new_config: Config) -> Result<(), String> {
    new_config.save().map_err(|e| e.to_string())?;
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    *config = new_config;
    Ok(())
}

#[tauri::command]
fn list_input_devices() -> Result<Vec<String>, String> {
    audio::list_input_devices().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_stats_summary(state: tauri::State<'_, AppState>) -> Result<db::StatsSummary, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_stats_summary().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_recent_sessions(state: tauri::State<'_, AppState>, limit: u32) -> Result<Vec<db::SessionRow>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    db.get_recent_sessions(limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_settings_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("settings") {
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(&app, "settings", tauri::WebviewUrl::App("/settings".into()))
        .title("VoiceClip Settings")
        .inner_size(450.0, 500.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn open_stats_window(app: AppHandle) -> Result<(), String> {
    if let Some(win) = app.get_webview_window("stats") {
        win.set_focus().map_err(|e| e.to_string())?;
        return Ok(());
    }
    tauri::WebviewWindowBuilder::new(&app, "stats", tauri::WebviewUrl::App("/stats".into()))
        .title("VoiceClip Stats")
        .inner_size(550.0, 500.0)
        .resizable(true)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn save_badge_position(state: tauri::State<'_, AppState>, x: i32, y: i32) -> Result<(), String> {
    let mut config = state.config.lock().map_err(|e| e.to_string())?;
    config.badge_x = Some(x);
    config.badge_y = Some(y);
    config.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let config = Config::load().expect("Failed to load config");
            let db_path = Config::get_db_path().expect("Failed to get DB path");
            let database = db::Database::new(&db_path).expect("Failed to init database");

            println!("VoiceClip initializing...");
            println!("Loaded configuration: {:#?}", config);
            println!("Database initialized at {:?}", db_path);

            // Audio worker thread
            let (audio_tx, audio_rx) = std::sync::mpsc::channel::<AudioCommand>();

            std::thread::spawn(move || {
                let mut recorder: Option<audio::AudioRecorder> = None;
                while let Ok(cmd) = audio_rx.recv() {
                    match cmd {
                        AudioCommand::Start(device_name) => {
                            if recorder.is_none() {
                                match audio::AudioRecorder::start_recording(device_name.as_deref()) {
                                    Ok(r) => recorder = Some(r),
                                    Err(e) => eprintln!("Audio recording failed to start: {}", e),
                                }
                            }
                        }
                        AudioCommand::StopAndSave(path, reply) => {
                            if let Some(r) = recorder.take() {
                                let res = r.stop_recording_and_save(&path);
                                let _ = reply.send(res);
                            } else {
                                let _ = reply.send(Err(anyhow::anyhow!("Not recording")));
                            }
                        }
                        AudioCommand::GetLevel(reply) => {
                            if let Some(ref r) = recorder {
                                let _ = reply.send(r.drain_levels());
                            } else {
                                let _ = reply.send(0.0);
                            }
                        }
                    }
                }
            });

            // Event channel for orchestrator
            let (event_tx, mut event_rx) = tokio::sync::mpsc::channel::<AppEvent>(32);

            // Start hotkey listener
            let tx_hotkey = event_tx.clone();
            hotkey::start_hotkey_listener(tx_hotkey, &config.hotkey);

            // Restore badge position
            if let Some(badge_win) = app.get_webview_window("badge") {
                if let (Some(x), Some(y)) = (config.badge_x, config.badge_y) {
                    let _ = badge_win.set_position(tauri::Position::Physical(
                        tauri::PhysicalPosition::new(x, y),
                    ));
                }
                let _ = badge_win.set_always_on_top(config.always_on_top);
            }

            // Store shared state
            let state = AppState {
                config: Mutex::new(config.clone()),
                db: Mutex::new(database),
                event_tx,
            };
            app.manage(state);

            // Spawn orchestrator
            let app_handle = app.handle().clone();
            let config_for_orchestrator = config;
            let audio_tx_orch = audio_tx;

            tokio::spawn(async move {
                let mut is_recording = false;
                let mut recording_start_time: i64 = 0;
                let mut level_poll_task: Option<tokio::task::JoinHandle<()>> = None;

                while let Some(event) = event_rx.recv().await {
                    println!("Received AppEvent: {:?}", event);
                    match event {
                        AppEvent::ToggleRecording => {
                            if !is_recording {
                                // Start recording
                                let mic = {
                                    let state = app_handle.state::<AppState>();
                                    let cfg = state.config.lock().unwrap();
                                    cfg.microphone.clone()
                                };
                                let _ = audio_tx_orch.send(AudioCommand::Start(mic));
                                is_recording = true;
                                recording_start_time = db::current_timestamp();
                                let _ = app_handle.emit("badge-state", "recording");
                                println!("Recording started");

                                // Start audio level polling
                                let handle = app_handle.clone();
                                let atx = audio_tx_orch.clone();
                                level_poll_task = Some(tokio::spawn(async move {
                                    loop {
                                        tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
                                        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                                        if atx.send(AudioCommand::GetLevel(reply_tx)).is_err() {
                                            break;
                                        }
                                        if let Ok(level) = reply_rx.await {
                                            let _ = handle.emit("audio-level", level);
                                        }
                                    }
                                }));
                            } else {
                                // Stop recording
                                is_recording = false;

                                // Stop level polling
                                if let Some(task) = level_poll_task.take() {
                                    task.abort();
                                }

                                let _ = app_handle.emit("badge-state", "processing");

                                let save_path = std::env::temp_dir().join("voiceclip_audio.wav");
                                let save_path_str = save_path.to_string_lossy().to_string();

                                let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                                let _ = audio_tx_orch.send(AudioCommand::StopAndSave(save_path_str, reply_tx));

                                let stop_res = reply_rx.await.unwrap_or_else(|_| Err(anyhow::anyhow!("Audio thread died")));

                                match stop_res {
                                    Ok(()) => {
                                        let start_time = std::time::Instant::now();
                                        let model_dir = Config::get_models_dir().unwrap_or_else(|_| std::env::temp_dir());
                                        let model_name = {
                                            let state = app_handle.state::<AppState>();
                                            let cfg = state.config.lock().unwrap();
                                            cfg.model_name.clone()
                                        };
                                        let model_path = model_dir.join(&model_name);
                                        let model_path_clone = model_path.clone();

                                        match whisper::transcribe(&save_path, &model_path).await {
                                            Ok(text) => {
                                                let latency_ms = start_time.elapsed().as_millis() as u32;
                                                println!("Transcription output: {}", text);

                                                {
                                                    let text_clone = text.clone();
                                                    let append = config_for_orchestrator.append_mode;
                                                    if let Err(e) = tokio::task::spawn_blocking(move || {
                                                        delivery::copy_to_clipboard(&text_clone, append)
                                                    }).await.unwrap_or_else(|e| Err(anyhow::anyhow!(e))) {
                                                        eprintln!("Delivery failed: {:?}", e);
                                                    }
                                                }
                                                {
                                                    let text_clone = text.clone();
                                                    let _ = tokio::task::spawn_blocking(move || {
                                                        delivery::notify("VoiceClip Success", &text_clone, false)
                                                    }).await;
                                                }

                                                let word_count = text.split_whitespace().count() as u32;
                                                let duration_secs = (db::current_timestamp() - recording_start_time) as u32;

                                                let session = db::SessionRecord {
                                                    started_at: recording_start_time,
                                                    duration_secs,
                                                    word_count,
                                                    model_used: model_path_clone.to_string_lossy().to_string(),
                                                    transcription: text,
                                                    latency_ms,
                                                    error: None,
                                                };

                                                {
                                                    let state = app_handle.state::<AppState>();
                                                    let _ = state.db.lock().map(|db| db.log_session(session));
                                                }

                                                let _ = app_handle.emit("badge-state", "success");

                                                let handle = app_handle.clone();
                                                tokio::spawn(async move {
                                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                    let _ = handle.emit("badge-state", "idle");
                                                });
                                            }
                                            Err(e) => {
                                                eprintln!("Transcription failed: {:?}", e);
                                                let _ = app_handle.emit("badge-state", "error");
                                                let _ = tokio::task::spawn_blocking(|| {
                                                    delivery::notify("VoiceClip Error", "Failed to transcribe audio", true)
                                                }).await;

                                                let duration_secs = (db::current_timestamp() - recording_start_time) as u32;
                                                let session = db::SessionRecord {
                                                    started_at: recording_start_time,
                                                    duration_secs,
                                                    word_count: 0,
                                                    model_used: model_path.to_string_lossy().to_string(),
                                                    transcription: String::new(),
                                                    latency_ms: start_time.elapsed().as_millis() as u32,
                                                    error: Some(e.to_string()),
                                                };
                                                {
                                                    let state = app_handle.state::<AppState>();
                                                    let _ = state.db.lock().map(|db| db.log_session(session));
                                                }

                                                let handle = app_handle.clone();
                                                tokio::spawn(async move {
                                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                                    let _ = handle.emit("badge-state", "idle");
                                                });
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        eprintln!("Failed to save recording.");
                                        let _ = app_handle.emit("badge-state", "error");
                                        let handle = app_handle.clone();
                                        tokio::spawn(async move {
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                            let _ = handle.emit("badge-state", "idle");
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            toggle_recording,
            get_config,
            save_config,
            list_input_devices,
            get_stats_summary,
            get_recent_sessions,
            open_settings_window,
            open_stats_window,
            save_badge_position,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
