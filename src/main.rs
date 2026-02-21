mod config;
mod audio;
mod whisper;
mod delivery;
mod db;
mod hotkey;
mod ui;

use config::Config;
use anyhow::Result;
use gtk4::prelude::*;
use gtk4::Application;
use gtk4::glib;

#[derive(Debug)]
pub enum AppEvent {
    ToggleRecording,
    ShowStats,
    ShowSettings,
    QuitApp,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("VoiceClip initializing...");
    
    let config = Config::load()?;
    println!("Loaded configuration: {:#?}", config);

    let db_path = Config::get_db_path()?;
    let db = db::Database::new(&db_path)?;
    println!("Database initialized at {:?}", db_path);

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AppEvent>(32);

    // App Initialization
    let app = Application::builder()
        .application_id("com.github.voiceclip")
        .build();

    let tx_clone = tx.clone();
    
    // Start global hotkey listener
    let tx_hotkey = tx.clone();
    hotkey::start_hotkey_listener(tx_hotkey, &config.hotkey);

    let db = std::sync::Arc::new(std::sync::Mutex::new(db));

    let (ui_tx, ui_rx) = tokio::sync::mpsc::unbounded_channel::<ui::badge::BadgeState>();
    let ui_rx_opt = std::rc::Rc::new(std::cell::RefCell::new(Some(ui_rx)));

    app.connect_activate(move |app| {
        if let Some(rx) = ui_rx_opt.borrow_mut().take() {
            ui::badge::build_ui(app, tx_clone.clone(), rx);
        }
    });

    enum AudioCommand {
        Start,
        StopAndSave(String, tokio::sync::oneshot::Sender<anyhow::Result<()>>),
    }

    let (audio_tx, audio_rx) = std::sync::mpsc::channel::<AudioCommand>();

    std::thread::spawn(move || {
        let mut recorder: Option<audio::AudioRecorder> = None;
        while let Ok(cmd) = audio_rx.recv() {
            match cmd {
                AudioCommand::Start => {
                    if recorder.is_none() {
                        match audio::AudioRecorder::start_recording() {
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
            }
        }
    });

    // Create central background processor task
    tokio::spawn(async move {
        let mut is_recording = false;
        let mut recording_start_time = 0;

        while let Some(event) = rx.recv().await {
            println!("Received AppEvent: {:?}", event);
            match event {
                AppEvent::ToggleRecording => {
                    if !is_recording {
                        let _ = audio_tx.send(AudioCommand::Start);
                        is_recording = true;
                        recording_start_time = db::current_timestamp();
                        let _ = ui_tx.send(ui::badge::BadgeState::Recording);
                        println!("Recording started");
                    } else {
                        is_recording = false;
                        let _ = ui_tx.send(ui::badge::BadgeState::Processing);
                        
                        let save_path = std::env::temp_dir().join("voiceclip_audio.wav");
                        let save_path_str = save_path.to_string_lossy().to_string();
                        
                        let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
                        let _ = audio_tx.send(AudioCommand::StopAndSave(save_path_str.clone(), reply_tx));

                        let stop_res = reply_rx.await.unwrap_or_else(|_| Err(anyhow::anyhow!("Audio thread died")));

                        match stop_res {
                            Ok(()) => {
                                let start_time = std::time::Instant::now();
                                let model_dir = config::Config::get_models_dir().unwrap_or_else(|_| std::env::temp_dir());
                                let model_path = model_dir.join(&config.model_name);
                                let model_path_clone = model_path.clone();
                                
                                match whisper::transcribe(&save_path, &model_path).await {
                                    Ok(text) => {
                                        let latency_ms = start_time.elapsed().as_millis() as u32;
                                        println!("Transcription output: {}", text);

                                        {
                                            let text_clone = text.clone();
                                            let append = config.append_mode;
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
                                        
                                        if let Ok(db_lock) = db.lock() {
                                            let _ = db_lock.log_session(session);
                                        }
                                        
                                        let _ = ui_tx.send(ui::badge::BadgeState::Success);

                                        let ui_tx_clone = ui_tx.clone();
                                        tokio::spawn(async move {
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                            let _ = ui_tx_clone.send(ui::badge::BadgeState::Idle);
                                        });
                                    }
                                    Err(e) => {
                                        eprintln!("Transcription failed: {:?}", e);
                                        let _ = ui_tx.send(ui::badge::BadgeState::Error);
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
                                        if let Ok(db_lock) = db.lock() {
                                            let _ = db_lock.log_session(session);
                                        }

                                        let ui_tx_clone = ui_tx.clone();
                                        tokio::spawn(async move {
                                            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                            let _ = ui_tx_clone.send(ui::badge::BadgeState::Idle);
                                        });
                                    }
                                }
                            }
                            Err(_) => {
                                eprintln!("Failed to save recording.");
                                let _ = ui_tx.send(ui::badge::BadgeState::Error);
                                let ui_tx_clone = ui_tx.clone();
                                tokio::spawn(async move {
                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                    let _ = ui_tx_clone.send(ui::badge::BadgeState::Idle);
                                });
                            }
                        }
                    }
                }
                AppEvent::ShowStats => {
                    let db_clone = db.clone();
                    glib::MainContext::default().invoke(move || {
                        if let Some(app) = gtk4::gio::Application::default() {
                            let app = app.downcast::<Application>().unwrap();
                            ui::stats::show_stats_window(&app, &db_clone);
                        }
                    });
                }
                AppEvent::ShowSettings => {
                    println!("Showing settings...");
                }
                AppEvent::QuitApp => {
                    println!("Quitting application...");
                    std::process::exit(0);
                }
            }
        }
    });

    app.run();

    Ok(())
}
