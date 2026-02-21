use gtk4::prelude::*;
use gtk4::{Application, Window, Box as GtkBox, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use std::sync::{Arc, Mutex};
use crate::db::Database;

pub fn show_stats_window(app: &Application, db: &Arc<Mutex<Database>>) {
    let window = Window::builder()
        .application(app)
        .title("VoiceClip Stats")
        .default_width(500)
        .default_height(400)
        .build();

    let provider = gtk4::CssProvider::new();
    provider.load_from_data(
        ".stats-header { padding: 16px; }
         .stats-metric { font-size: 16px; font-weight: bold; padding: 8px 16px; }
         .stats-metric-value { font-size: 24px; }
         .session-row { padding: 8px 16px; }
         .session-error { color: #dc143c; }
         .section-title { font-size: 14px; font-weight: bold; padding: 8px 16px; }"
    );
    gtk4::style_context_add_provider_for_display(
        &gtk4::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let main_box = GtkBox::new(Orientation::Vertical, 0);

    // Summary section
    let summary_box = GtkBox::builder()
        .orientation(Orientation::Horizontal)
        .homogeneous(true)
        .css_classes(["stats-header"])
        .build();

    let db_lock = db.lock().unwrap();

    let (total_label, minutes_label, avg_label) = match db_lock.get_stats_summary() {
        Ok(stats) => {
            let minutes = stats.total_seconds as f64 / 60.0;
            (
                format!("Total Recordings\n{}", stats.total_recordings),
                format!("Total Minutes\n{:.1}", minutes),
                format!("Avg Words/Session\n{:.1}", stats.avg_words),
            )
        }
        Err(_) => (
            "Total Recordings\n0".to_string(),
            "Total Minutes\n0.0".to_string(),
            "Avg Words/Session\n0.0".to_string(),
        ),
    };

    for text in [&total_label, &minutes_label, &avg_label] {
        let label = Label::builder()
            .label(text)
            .css_classes(["stats-metric"])
            .halign(gtk4::Align::Center)
            .justify(gtk4::Justification::Center)
            .build();
        summary_box.append(&label);
    }

    main_box.append(&summary_box);

    // Section title
    let history_title = Label::builder()
        .label("Recent Sessions")
        .css_classes(["section-title"])
        .halign(gtk4::Align::Start)
        .build();
    main_box.append(&history_title);

    // History section
    let list_box = ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);

    match db_lock.get_recent_sessions(50) {
        Ok(sessions) => {
            for session in sessions {
                let row = ListBoxRow::new();
                let row_box = GtkBox::new(Orientation::Vertical, 2);
                row_box.add_css_class("session-row");

                let datetime = format_timestamp(session.started_at);
                let duration = format!("{}s", session.duration_secs);
                let header_text = format!(
                    "{} | {} | {} words | {}ms latency",
                    datetime, duration, session.word_count, session.latency_ms
                );

                let header_label = Label::builder()
                    .label(&header_text)
                    .halign(gtk4::Align::Start)
                    .build();

                if session.error.is_some() {
                    header_label.add_css_class("session-error");
                }

                row_box.append(&header_label);

                let body = if let Some(ref err) = session.error {
                    format!("Error: {}", truncate_str(err, 80))
                } else {
                    truncate_str(&session.transcription, 80)
                };

                let body_label = Label::builder()
                    .label(&body)
                    .halign(gtk4::Align::Start)
                    .wrap(true)
                    .build();

                if session.error.is_some() {
                    body_label.add_css_class("session-error");
                }

                row_box.append(&body_label);
                row.set_child(Some(&row_box));
                list_box.append(&row);
            }
        }
        Err(e) => {
            let err_label = Label::new(Some(&format!("Failed to load sessions: {}", e)));
            list_box.append(&err_label);
        }
    }

    drop(db_lock);

    let scrolled = ScrolledWindow::builder()
        .vexpand(true)
        .child(&list_box)
        .build();
    main_box.append(&scrolled);

    window.set_child(Some(&main_box));
    window.present();
}

fn truncate_str(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let truncated: String = s.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

fn format_timestamp(ts: i64) -> String {
    let secs = ts % 60;
    let mins = (ts / 60) % 60;
    let hours = (ts / 3600) % 24;
    let days = ts / 86400;
    // Simple date calc from unix epoch
    let (year, month, day) = days_to_date(days);
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hours, mins, secs)
}

fn days_to_date(days_since_epoch: i64) -> (i64, i64, i64) {
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days_since_epoch + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m as i64, d as i64)
}
