use gtk4::prelude::*;
use gtk4::{Application, Window, Box as GtkBox, Label, Entry, Button, Orientation};
use crate::config::Config;

pub fn show_settings_window(app: &Application) {
    let window = Window::builder()
        .application(app)
        .title("VoiceClip Settings")
        .default_width(400)
        .default_height(250)
        .build();

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        ".settings-section { font-size: 14px; font-weight: bold; padding: 8px 16px; }
         .settings-help { padding: 4px 16px; opacity: 0.7; }
         .settings-info { padding: 8px 16px; font-style: italic; }
         .settings-status { padding: 8px 16px; }
         .settings-save { margin: 8px 16px; }"
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let main_box = GtkBox::new(Orientation::Vertical, 8);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);

    // Section label
    let section_label = Label::builder()
        .label("Global Hotkey")
        .css_classes(["settings-section"])
        .halign(gtk4::Align::Start)
        .build();
    main_box.append(&section_label);

    // Hotkey entry
    let current_hotkey = Config::load()
        .map(|c| c.hotkey)
        .unwrap_or_else(|_| "Super+Alt+V".to_string());

    let entry = Entry::builder()
        .text(&current_hotkey)
        .margin_start(16)
        .margin_end(16)
        .build();
    main_box.append(&entry);

    // Help text
    let help_label = Label::builder()
        .label("Examples: Super+Alt+V, Ctrl+Shift+R, Super+Shift+1\nSet to \"None\" to disable the hotkey.")
        .css_classes(["settings-help"])
        .halign(gtk4::Align::Start)
        .wrap(true)
        .build();
    main_box.append(&help_label);

    // Info label
    let info_label = Label::builder()
        .label("Changes take effect on restart.")
        .css_classes(["settings-info"])
        .halign(gtk4::Align::Start)
        .build();
    main_box.append(&info_label);

    // Status label (for save confirmation)
    let status_label = Label::builder()
        .label("")
        .css_classes(["settings-status"])
        .halign(gtk4::Align::Start)
        .build();
    main_box.append(&status_label);

    // Save button
    let save_button = Button::builder()
        .label("Save")
        .css_classes(["settings-save"])
        .build();

    let entry_clone = entry.clone();
    let status_clone = status_label.clone();
    save_button.connect_clicked(move |_| {
        let new_hotkey = entry_clone.text().to_string();
        match Config::load() {
            Ok(mut config) => {
                config.hotkey = new_hotkey;
                match config.save() {
                    Ok(()) => {
                        status_clone.set_text("Saved! Restart VoiceClip to apply.");
                    }
                    Err(e) => {
                        status_clone.set_text(&format!("Error saving: {}", e));
                    }
                }
            }
            Err(e) => {
                status_clone.set_text(&format!("Error loading config: {}", e));
            }
        }
    });
    main_box.append(&save_button);

    window.set_child(Some(&main_box));
    window.present();
}
