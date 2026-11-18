use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use adw::prelude::*;
use gtk4::gdk::Display;
use gtk4::{Button, CssProvider, glib};
use libadwaita as adw;

use flipper_rpc::fs::FsRead;
use flipper_rpc::proto::storage::DeleteRequest;
use flipper_rpc::rpc::{req::Request, res::Response};
use flipper_rpc::transport::Transport;
use flipper_rpc::transport::serial::{list_flipper_ports, rpc::SerialRpcTransport};

const APP_ID: &str = "com.cardoor.system";
const SCAN_FILE_PATH: &str = "/ext/apps_data/card_sensor/scan.txt";

enum WorkerEvent {
    Status(String),
    CardScanned { protocol: String, uid: String },
}

enum WorkerCommand {
    StartApp(String),
}

fn main() {
    let app = adw::Application::builder()
        .application_id(APP_ID)
        .build();

    app.connect_activate(main_window);
    app.run();
}

fn main_window(app: &adw::Application) {
    // Load theme.css once GTK has initialized the display
    let provider = CssProvider::new();
    provider.load_from_path("src/theme.css");
    if let Some(display) = Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let (event_tx, event_rx) = mpsc::channel::<WorkerEvent>();
    spawn_rpc_worker(event_tx);

    // Header bar with remote app launcher button
    let header_bar = adw::HeaderBar::new();

    // Connection status group
    let connection_group = adw::PreferencesGroup::builder()
        .title("Flipper Zero Connection")
        .build();

    let status_row = adw::ActionRow::builder()
        .title("RPC Session")
        .subtitle("Searching for Flipper Zero over USB...")
        .build();
    connection_group.add(&status_row);

    // Captured card readout group
    let card_group = adw::PreferencesGroup::builder()
        .title("Captured RFID Card")
        .description("Reads incoming scans from /ext/apps_data/card_sensor/scan.txt")
        .build();

    let state_row = adw::ActionRow::builder()
        .title("Capture State")
        .subtitle("Waiting for first card scan...")
        .build();

    let protocol_row = adw::ActionRow::builder()
        .title("Protocol")
        .subtitle("—")
        .build();

    let uid_row = adw::ActionRow::builder()
        .title("UID")
        .subtitle("—")
        .build();

    let auto_reset_row = adw::SwitchRow::builder()
        .title("Auto-Reset After Capture")
        .subtitle("Overwrite readout on every new scan instead of locking on the first card")
        .active(false)
        .build();

    card_group.add(&state_row);
    card_group.add(&protocol_row);
    card_group.add(&uid_row);
    card_group.add(&auto_reset_row);

    // Manual reset button for single-capture mode
    let reset_btn = Button::builder()
        .label("Reset / Capture Next Card")
        .css_classes(["suggested-action", "pill"])
        .halign(gtk4::Align::Center)
        .margin_top(12)
        .sensitive(false)
        .build();

    let content_box = gtk4::Box::builder()
        .orientation(gtk4::Orientation::Vertical)
        .spacing(18)
        .build();
    content_box.append(&connection_group);
    content_box.append(&card_group);
    content_box.append(&reset_btn);

    let clamp = adw::Clamp::builder()
        .maximum_size(560)
        .margin_top(24)
        .margin_bottom(24)
        .margin_start(16)
        .margin_end(16)
        .child(&content_box)
        .build();

    let toolbar_view = adw::ToolbarView::new();
    toolbar_view.add_top_bar(&header_bar);
    toolbar_view.set_content(Some(&clamp));

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Card Door")
        .default_width(600)
        .default_height(520)
        .content(&toolbar_view)
        .build();

    // Shared UI capture state
    let locked = Rc::new(Cell::new(false));
    let auto_reset = Rc::new(Cell::new(false));

    // Reset button clears current card and unlocks capture
    {
        let locked = Rc::clone(&locked);
        let state_row = state_row.clone();
        let protocol_row = protocol_row.clone();
        let uid_row = uid_row.clone();
        let reset_btn = reset_btn.clone();
        let reset_btn_clone = reset_btn.clone();

        reset_btn.connect_clicked(move |_| {
            locked.set(false);
            protocol_row.set_subtitle("—");
            uid_row.set_subtitle("—");
            state_row.set_subtitle("Waiting for card scan...");
            reset_btn_clone.set_sensitive(false);
        });
    }

    // Auto-reset switch handler
    {
        let locked = Rc::clone(&locked);
        let auto_reset = Rc::clone(&auto_reset);
        let state_row = state_row.clone();
        let reset_btn = reset_btn.clone();

        auto_reset_row.connect_active_notify(move |row| {
            let is_auto = row.is_active();
            auto_reset.set(is_auto);

            if is_auto {
                locked.set(false);
                reset_btn.set_sensitive(false);
                state_row.set_subtitle("Auto-Reset Active — Listening for cards...");
            } else if !locked.get() {
                state_row.set_subtitle("Waiting for first card scan...");
            }
        });
    }

    // Poll background RPC thread events on the GTK main loop
    glib::timeout_add_local(Duration::from_millis(100), move || {
        while let Ok(event) = event_rx.try_recv() {
            match event {
                WorkerEvent::Status(msg) => {
                    status_row.set_subtitle(&msg);
                }
                WorkerEvent::CardScanned { protocol, uid } => {
                    if !locked.get() || auto_reset.get() {
                        protocol_row.set_subtitle(&protocol);
                        uid_row.set_subtitle(&uid);

                        if auto_reset.get() {
                            state_row.set_subtitle("Auto-Reset Active — Ready for next card");
                            reset_btn.set_sensitive(false);
                        } else {
                            locked.set(true);
                            state_row.set_subtitle("Card Captured (Locked)");
                            reset_btn.set_sensitive(true);
                        }
                    }
                }
            }
        }
        glib::ControlFlow::Continue
    });

    window.present();
}

fn spawn_rpc_worker(event_tx: mpsc::Sender<WorkerEvent>) {
    thread::spawn(move || {
        loop {
            let device = match list_flipper_ports() {
                Ok(devices) => match devices.into_iter().next() {
                    Some(dev) => dev,
                    None => {
                        let _ = event_tx.send(WorkerEvent::Status(
                            "Waiting for Flipper Zero over USB...".into(),
                        ));
                        thread::sleep(Duration::from_secs(1));
                        continue;
                    }
                },
                Err(err) => {
                    let _ = event_tx.send(WorkerEvent::Status(format!(
                        "Serial port scan error: {err}"
                    )));
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };

            let _ = event_tx.send(WorkerEvent::Status(format!(
                "Connecting to {} ({})...",
                device.device_name, device.port_name
            )));

            let mut client = match SerialRpcTransport::new(device.port_name) {
                Ok(c) => c,
                Err(err) => {
                    let _ = event_tx.send(WorkerEvent::Status(format!(
                        "RPC handshake failed ({err:?}), retrying..."
                    )));
                    thread::sleep(Duration::from_secs(1));
                    continue;
                }
            };

            let _ = event_tx.send(WorkerEvent::Status(format!(
                "Connected to {}",
                device.device_name
            )));

            // Active session polling loop
            loop {
                match client.fs_read_to_string(SCAN_FILE_PATH) {
                    Ok(raw) => {
                        let _ = client.send_and_receive(Request::StorageDelete(DeleteRequest {
                            path: SCAN_FILE_PATH.to_string(),
                            recursive: false,
                        }));

                        let trimmed = raw.trim();
                        if !trimmed.is_empty() {
                            let (protocol, uid) = match trimmed.split_once('|') {
                                Some((p, u)) => (p.trim().to_string(), u.trim().to_string()),
                                None => ("Unknown".to_string(), trimmed.to_string()),
                            };
                            let _ = event_tx.send(WorkerEvent::CardScanned { protocol, uid });
                        }
                    }
                    Err(_) => {
                        match client.send_and_receive(Request::Ping(vec![1, 2, 3, 4])) {
                            Ok(Response::Ping(_)) => {}
                            _ => {
                                let _ = event_tx.send(WorkerEvent::Status(
                                    "Flipper Zero disconnected. Reconnecting...".into(),
                                ));
                                break;
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(200));
            }
        }
    });
}