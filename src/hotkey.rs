use evdev::{Device, EventType, KeyCode};
use std::collections::HashSet;
use tokio::sync::mpsc::Sender;
use crate::AppEvent;

pub fn start_hotkey_listener(tx: Sender<AppEvent>, _configured_hotkey: &str) {
    let devices = evdev::enumerate().filter(|(_, d)| d.supported_keys().is_some());
    
    for (path, mut device) in devices {
        let name = device.name().unwrap_or("Unknown").to_string();
        let tx_clone = tx.clone();
        
        std::thread::spawn(move || {
            let mut pressed_keys = HashSet::new();
            
            loop {
                match device.fetch_events() {
                    Ok(events) => {
                        for event in events {
                            if event.event_type() == EventType::KEY {
                                let key = KeyCode::new(event.code());
                                if event.value() == 1 {
                                    pressed_keys.insert(key);
                                } else if event.value() == 0 {
                                    pressed_keys.remove(&key);
                                }

                                let super_pressed = pressed_keys.contains(&KeyCode::KEY_LEFTMETA) || pressed_keys.contains(&KeyCode::KEY_RIGHTMETA);
                                let shift_pressed = pressed_keys.contains(&KeyCode::KEY_LEFTSHIFT) || pressed_keys.contains(&KeyCode::KEY_RIGHTSHIFT);
                                let v_pressed = pressed_keys.contains(&KeyCode::KEY_V);

                                if super_pressed && shift_pressed && v_pressed && event.value() == 1 && key == KeyCode::KEY_V {
                                    let _ = tx_clone.blocking_send(AppEvent::ToggleRecording);
                                }
                            }
                        }
                    }
                    Err(_) => {
                        break; // Device removed or error
                    }
                }
            }
        });
    }
}
