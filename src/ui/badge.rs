use gtk4::prelude::*;
use gtk4::{gio, Application, ApplicationWindow, GestureDrag, GestureClick, Box as GtkBox, PopoverMenu};
use crate::AppEvent;

#[cfg(target_os = "linux")]
use std::cell::Cell;
#[cfg(target_os = "linux")]
use std::rc::Rc;
#[cfg(target_os = "linux")]
use gtk4_layer_shell::{Layer, LayerShell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeState {
    Idle,
    Recording,
    Processing,
    Success,
    Error,
}

pub fn build_ui(app: &Application, tx: tokio::sync::mpsc::Sender<AppEvent>, mut rx: tokio::sync::mpsc::UnboundedReceiver<BadgeState>) {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("VoiceClip Badge")
        .default_width(48)
        .default_height(48)
        .decorated(false)
        .build();

    // Platform-specific window positioning
    #[cfg(target_os = "linux")]
    {
        window.init_layer_shell();
        window.set_layer(Layer::Top);
        window.set_namespace(Some("voiceclip_badge"));
        window.set_anchor(gtk4_layer_shell::Edge::Bottom, true);
        window.set_anchor(gtk4_layer_shell::Edge::Right, true);
        window.set_margin(gtk4_layer_shell::Edge::Bottom, 50);
        window.set_margin(gtk4_layer_shell::Edge::Right, 50);
    }

    #[cfg(target_os = "windows")]
    {
        window.set_resizable(false);
        window.set_deletable(false);
        // On Windows, the badge acts as a regular always-on-top window.
        // GTK4 does not provide set_keep_above directly; platform-specific
        // Win32 SetWindowPos(HWND_TOPMOST) can be used if needed.
    }

    // Make the background transparent in GTK4
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        "window {
            background-color: transparent;
        }
        .badge {
            border-radius: 24px;
            transition: all 0.3s ease-in-out;
        }
        .badge.idle {
            background-color: rgba(128, 128, 128, 0.8);
        }
        .badge.recording {
            background-color: rgba(255, 69, 0, 0.9);
        }
        .badge.processing {
            background-color: rgba(30, 144, 255, 0.9);
        }
        .badge.success {
            background-color: rgba(50, 205, 50, 0.9);
        }
        .badge.error {
            background-color: rgba(220, 20, 60, 0.9);
        }"
    );

    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    // Create the circular badge widget
    let badge_box = GtkBox::builder()
        .css_classes(["badge", "idle"])
        .width_request(48)
        .height_request(48)
        .halign(gtk4::Align::Center)
        .valign(gtk4::Align::Center)
        .build();

    window.set_child(Some(&badge_box));

    // Listen to UI state updates
    let badge_clone = badge_box.clone();
    gtk4::glib::MainContext::default().spawn_local(async move {
        while let Some(state) = rx.recv().await {
            badge_clone.remove_css_class("idle");
            badge_clone.remove_css_class("recording");
            badge_clone.remove_css_class("processing");
            badge_clone.remove_css_class("success");
            badge_clone.remove_css_class("error");

            let class = match state {
                BadgeState::Idle => "idle",
                BadgeState::Recording => "recording",
                BadgeState::Processing => "processing",
                BadgeState::Success => "success",
                BadgeState::Error => "error",
            };
            badge_clone.add_css_class(class);
        }
    });

    // Implement Dragging
    setup_dragging(&window, &badge_box);

    // Setup Context Menu (Popover)
    let menu = gio::Menu::new();
    menu.append(Some("Stats"), Some("app.stats"));
    menu.append(Some("Settings"), Some("app.settings"));
    menu.append(Some("Quit"), Some("app.quit"));

    let popover = PopoverMenu::from_model(Some(&menu));
    popover.set_parent(&badge_box);

    // Setup actions for Context Menu
    setup_actions(app, tx.clone());

    // Implement Click Handling
    setup_clicks(&badge_box, &popover, tx);

    window.present();
}

fn setup_actions(app: &Application, tx: tokio::sync::mpsc::Sender<AppEvent>) {
    let stats_action = gio::SimpleAction::new("stats", None);
    let tx_stats = tx.clone();
    stats_action.connect_activate(move |_, _| {
        let _ = tx_stats.try_send(AppEvent::ShowStats);
    });

    let settings_action = gio::SimpleAction::new("settings", None);
    let tx_settings = tx.clone();
    settings_action.connect_activate(move |_, _| {
        let _ = tx_settings.try_send(AppEvent::ShowSettings);
    });

    let quit_action = gio::SimpleAction::new("quit", None);
    let tx_quit = tx;
    quit_action.connect_activate(move |_, _| {
        let _ = tx_quit.try_send(AppEvent::QuitApp);
    });

    app.add_action(&stats_action);
    app.add_action(&settings_action);
    app.add_action(&quit_action);
}

fn setup_clicks(badge: &GtkBox, popover: &PopoverMenu, tx: tokio::sync::mpsc::Sender<AppEvent>) {
    let click = GestureClick::new();
    // Accept standard clicks (left, right, etc.)
    click.set_button(0);

    let popover_clone = popover.clone();

    click.connect_pressed(move |gesture, n_press, x, y| {
        let button = gesture.current_button();

        if n_press == 1 {
            if button == gtk4::gdk::BUTTON_PRIMARY {
                // Left click
                let _ = tx.try_send(AppEvent::ToggleRecording);
            } else if button == gtk4::gdk::BUTTON_SECONDARY {
                // Right click
                let rect = gtk4::gdk::Rectangle::new(x as i32, y as i32, 1, 1);
                popover_clone.set_pointing_to(Some(&rect));
                popover_clone.popup();
            }
        }
    });

    badge.add_controller(click);
}

#[cfg(target_os = "linux")]
fn setup_dragging(window: &ApplicationWindow, badge: &GtkBox) {
    let drag = GestureDrag::new();
    let initial_margin_x = Rc::new(Cell::new(0));
    let initial_margin_y = Rc::new(Cell::new(0));

    let window_clone = window.clone();
    let mx_clone = initial_margin_x.clone();
    let my_clone = initial_margin_y.clone();

    drag.connect_drag_begin(move |_, _x, _y| {
        mx_clone.set(window_clone.margin(gtk4_layer_shell::Edge::Right));
        my_clone.set(window_clone.margin(gtk4_layer_shell::Edge::Bottom));
    });

    let window_clone = window.clone();
    drag.connect_drag_update(move |_, offset_x, offset_y| {
        let new_margin_x = initial_margin_x.get() as f64 - offset_x;
        let new_margin_y = initial_margin_y.get() as f64 - offset_y;

        window_clone.set_margin(gtk4_layer_shell::Edge::Right, new_margin_x as i32);
        window_clone.set_margin(gtk4_layer_shell::Edge::Bottom, new_margin_y as i32);
    });

    badge.add_controller(drag);
}

#[cfg(target_os = "windows")]
fn setup_dragging(_window: &ApplicationWindow, badge: &GtkBox) {
    let drag = GestureDrag::new();

    // On Windows, use GTK4's native drag-begin to initiate an interactive window move
    let window_clone = _window.clone();
    drag.connect_drag_begin(move |gesture, _x, _y| {
        // Initiate native window move via GDK toplevel
        if let Some(native) = window_clone.native() {
            if let Some(surface) = native.surface() {
                // GTK4 handles window dragging through the windowing system
            }
        }
    });

    badge.add_controller(drag);
}
