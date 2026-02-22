use tokio::sync::mpsc::Sender;
use crate::AppEvent;

#[cfg(target_os = "linux")]
pub fn start_hotkey_listener(tx: Sender<AppEvent>, configured_hotkey: &str) {
    use evdev::{EventType, KeyCode};
    use std::collections::HashSet;

    let hotkey_trimmed = configured_hotkey.trim();
    if hotkey_trimmed.is_empty() || hotkey_trimmed.eq_ignore_ascii_case("none") {
        eprintln!("Hotkey disabled (set to '{}'), badge click still works.", configured_hotkey);
        return;
    }

    let parsed = match parse_hotkey_evdev(hotkey_trimmed) {
        Some(p) => p,
        None => {
            eprintln!("Warning: could not parse hotkey '{}'. Hotkey disabled, badge click still works.", configured_hotkey);
            return;
        }
    };

    let devices = evdev::enumerate().filter(|(_, d)| d.supported_keys().is_some());

    for (_path, mut device) in devices {
        let tx_clone = tx.clone();
        let parsed = parsed.clone();

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

                                let all_modifiers_pressed = parsed.modifiers.iter().all(|mod_pair| {
                                    pressed_keys.contains(&mod_pair.0) || pressed_keys.contains(&mod_pair.1)
                                });

                                let trigger_pressed = pressed_keys.contains(&parsed.trigger);

                                if all_modifiers_pressed && trigger_pressed && event.value() == 1 && key == parsed.trigger {
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

#[cfg(target_os = "linux")]
#[derive(Clone)]
struct ParsedEvdevHotkey {
    /// Each modifier is a pair of (left, right) keycodes — either satisfies
    modifiers: Vec<(evdev::KeyCode, evdev::KeyCode)>,
    trigger: evdev::KeyCode,
}

#[cfg(target_os = "linux")]
fn parse_hotkey_evdev(s: &str) -> Option<ParsedEvdevHotkey> {
    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let trigger_str = parts.last()?;
    let modifier_strs = &parts[..parts.len() - 1];

    let trigger = key_name_to_evdev(trigger_str)?;

    let mut modifiers = Vec::new();
    for m in modifier_strs {
        let pair = modifier_name_to_evdev_pair(m)?;
        modifiers.push(pair);
    }

    Some(ParsedEvdevHotkey { modifiers, trigger })
}

#[cfg(target_os = "linux")]
fn modifier_name_to_evdev_pair(name: &str) -> Option<(evdev::KeyCode, evdev::KeyCode)> {
    use evdev::KeyCode;
    match name.to_ascii_lowercase().as_str() {
        "super" | "win" | "meta" => Some((KeyCode::KEY_LEFTMETA, KeyCode::KEY_RIGHTMETA)),
        "shift" => Some((KeyCode::KEY_LEFTSHIFT, KeyCode::KEY_RIGHTSHIFT)),
        "alt" => Some((KeyCode::KEY_LEFTALT, KeyCode::KEY_RIGHTALT)),
        "ctrl" | "control" => Some((KeyCode::KEY_LEFTCTRL, KeyCode::KEY_RIGHTCTRL)),
        _ => None,
    }
}

#[cfg(target_os = "linux")]
fn key_name_to_evdev(name: &str) -> Option<evdev::KeyCode> {
    use evdev::KeyCode;
    let upper = name.to_ascii_uppercase();
    match upper.as_str() {
        "A" => Some(KeyCode::KEY_A),
        "B" => Some(KeyCode::KEY_B),
        "C" => Some(KeyCode::KEY_C),
        "D" => Some(KeyCode::KEY_D),
        "E" => Some(KeyCode::KEY_E),
        "F" => Some(KeyCode::KEY_F),
        "G" => Some(KeyCode::KEY_G),
        "H" => Some(KeyCode::KEY_H),
        "I" => Some(KeyCode::KEY_I),
        "J" => Some(KeyCode::KEY_J),
        "K" => Some(KeyCode::KEY_K),
        "L" => Some(KeyCode::KEY_L),
        "M" => Some(KeyCode::KEY_M),
        "N" => Some(KeyCode::KEY_N),
        "O" => Some(KeyCode::KEY_O),
        "P" => Some(KeyCode::KEY_P),
        "Q" => Some(KeyCode::KEY_Q),
        "R" => Some(KeyCode::KEY_R),
        "S" => Some(KeyCode::KEY_S),
        "T" => Some(KeyCode::KEY_T),
        "U" => Some(KeyCode::KEY_U),
        "V" => Some(KeyCode::KEY_V),
        "W" => Some(KeyCode::KEY_W),
        "X" => Some(KeyCode::KEY_X),
        "Y" => Some(KeyCode::KEY_Y),
        "Z" => Some(KeyCode::KEY_Z),
        "0" => Some(KeyCode::KEY_0),
        "1" => Some(KeyCode::KEY_1),
        "2" => Some(KeyCode::KEY_2),
        "3" => Some(KeyCode::KEY_3),
        "4" => Some(KeyCode::KEY_4),
        "5" => Some(KeyCode::KEY_5),
        "6" => Some(KeyCode::KEY_6),
        "7" => Some(KeyCode::KEY_7),
        "8" => Some(KeyCode::KEY_8),
        "9" => Some(KeyCode::KEY_9),
        "F1" => Some(KeyCode::KEY_F1),
        "F2" => Some(KeyCode::KEY_F2),
        "F3" => Some(KeyCode::KEY_F3),
        "F4" => Some(KeyCode::KEY_F4),
        "F5" => Some(KeyCode::KEY_F5),
        "F6" => Some(KeyCode::KEY_F6),
        "F7" => Some(KeyCode::KEY_F7),
        "F8" => Some(KeyCode::KEY_F8),
        "F9" => Some(KeyCode::KEY_F9),
        "F10" => Some(KeyCode::KEY_F10),
        "F11" => Some(KeyCode::KEY_F11),
        "F12" => Some(KeyCode::KEY_F12),
        _ => None,
    }
}

#[cfg(target_os = "windows")]
pub fn start_hotkey_listener(tx: Sender<AppEvent>, configured_hotkey: &str) {
    use global_hotkey::{GlobalHotKeyManager, GlobalHotKeyEvent, hotkey::{HotKey, Modifiers, Code}};

    let hotkey_trimmed = configured_hotkey.trim();
    if hotkey_trimmed.is_empty() || hotkey_trimmed.eq_ignore_ascii_case("none") {
        eprintln!("Hotkey disabled (set to '{}'), badge click still works.", configured_hotkey);
        return;
    }

    let hotkey = match parse_hotkey(hotkey_trimmed) {
        Some(hk) => hk,
        None => {
            eprintln!("Warning: could not parse hotkey '{}'. Hotkey disabled, badge click still works.", configured_hotkey);
            return;
        }
    };

    let manager = match GlobalHotKeyManager::new() {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Warning: failed to create hotkey manager: {}. Hotkey disabled, badge click still works.", e);
            return;
        }
    };

    if let Err(e) = manager.register(hotkey) {
        eprintln!("Warning: failed to register hotkey '{}': {}. Hotkey disabled, badge click still works.", configured_hotkey, e);
        return;
    }

    // GlobalHotKeyManager contains a raw pointer that isn't Send,
    // but it is safe to move to another thread as long as we keep it alive.
    struct SendManager(GlobalHotKeyManager);
    unsafe impl Send for SendManager {}

    let manager = SendManager(manager);

    std::thread::spawn(move || {
        let _manager = manager; // prevent drop
        loop {
            if let Ok(_event) = GlobalHotKeyEvent::receiver().recv() {
                let _ = tx.blocking_send(AppEvent::ToggleRecording);
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn parse_hotkey(s: &str) -> Option<global_hotkey::hotkey::HotKey> {
    use global_hotkey::hotkey::{HotKey, Modifiers, Code};

    let parts: Vec<&str> = s.split('+').map(|p| p.trim()).collect();
    if parts.is_empty() {
        return None;
    }

    let trigger_str = parts.last()?;
    let modifier_strs = &parts[..parts.len() - 1];

    let mut modifiers = Modifiers::empty();
    for m in modifier_strs {
        match m.to_ascii_lowercase().as_str() {
            "super" | "win" | "meta" => modifiers |= Modifiers::SUPER,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" => modifiers |= Modifiers::ALT,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            _ => return None,
        }
    }

    let code = key_name_to_code(trigger_str)?;

    let mods = if modifiers.is_empty() { None } else { Some(modifiers) };
    Some(HotKey::new(mods, code))
}

#[cfg(target_os = "windows")]
fn key_name_to_code(name: &str) -> Option<global_hotkey::hotkey::Code> {
    use global_hotkey::hotkey::Code;
    let upper = name.to_ascii_uppercase();
    match upper.as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" | "DIGIT0" => Some(Code::Digit0),
        "1" | "DIGIT1" => Some(Code::Digit1),
        "2" | "DIGIT2" => Some(Code::Digit2),
        "3" | "DIGIT3" => Some(Code::Digit3),
        "4" | "DIGIT4" => Some(Code::Digit4),
        "5" | "DIGIT5" => Some(Code::Digit5),
        "6" | "DIGIT6" => Some(Code::Digit6),
        "7" | "DIGIT7" => Some(Code::Digit7),
        "8" | "DIGIT8" => Some(Code::Digit8),
        "9" | "DIGIT9" => Some(Code::Digit9),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
}
