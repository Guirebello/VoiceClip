use tokio::sync::mpsc::Sender;
use crate::AppEvent;

#[cfg(target_os = "linux")]
pub fn start_hotkey_listener(tx: Sender<AppEvent>, _configured_hotkey: &str) {
    use evdev::{EventType, KeyCode};
    use std::collections::HashSet;

    let devices = evdev::enumerate().filter(|(_, d)| d.supported_keys().is_some());

    for (_path, mut device) in devices {
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

#[cfg(target_os = "windows")]
pub fn start_hotkey_listener(tx: Sender<AppEvent>, _configured_hotkey: &str) {
    use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent, hotkey::{HotKey, Modifiers, Code}};

    let hotkey = HotKey::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyV);
    let manager = GlobalHotKeyManager::new().unwrap();
    manager.register(hotkey).unwrap();

    std::thread::spawn(move || {
        let _manager = manager; // prevent drop
        loop {
            if let Ok(_event) = GlobalHotKeyEvent::receiver().recv() {
                let _ = tx.blocking_send(AppEvent::ToggleRecording);
            }
        }
    });
}
