use std::io::{Read, Cursor};
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;
use rodio::{Decoder, OutputStream, Sink};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    stream_url: String
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            stream_url: "Paste a URL to stream here".to_owned(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("vinyl - rusty radio");

            ui.text_edit_singleline(&mut self.stream_url);

            ui.separator();

            if ui.button("Play").clicked() {
                let stream_url = self.stream_url.clone();

                thread::spawn(move || {
                    generate_stream(stream_url);
                });
            }

            ui.separator();

            ui.label(format!("Current stream URL: {}", self.stream_url));

            ui.separator();

            ui.add(
              egui::Image::new(egui::include_image!("../assets/icon-vinyl.png"))
                .corner_radius(5)
            );
        });
    }
}

fn generate_stream(stream_url: String) -> () {
    println!("Starting buffered stream from: {}", stream_url);

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to build client");

    let mut response = match client.get(&stream_url).send() {
        Ok(resp) => {
            println!("Response received. Status: {}", resp.status());
            resp
        }
        Err(e) => {
            eprintln!("Failed to fetch stream: {}", e);
            return;
        }
    };

    let mut buffer = Vec::new();
    let mut temp = [0; 8192]; // 8 KB chunks

    println!("Buffering audio...");

    // Read first few chunks to start playback early
    for _ in 0..10 {
        match response.read(&mut temp) {
            Ok(0) => break, // EOF
            Ok(n) => buffer.extend_from_slice(&temp[..n]),
            Err(e) => {
                eprintln!("Error reading stream: {}", e);
                return;
            }
        }
    }

    println!("Buffered {} bytes. Starting playback...", buffer.len());

    let cursor = Cursor::new(buffer);
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    match Decoder::new(cursor) {
        Ok(source) => {
            sink.append(source);
            println!("Playback started.");

            thread::spawn(move || {
                while let Ok(n) = response.read(&mut temp) {
                    if n == 0 {
                        break;
                    }
                }
                println!("Finished buffering.");
            });

            sink.sleep_until_end();
            generate_stream(stream_url.clone());
            println!("Playback finished.");
        }
        Err(e) => {
            eprintln!("Failed to decode stream: {}", e);
        }
    }
}