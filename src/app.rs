/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,
    stream_url: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            stream_url: "https://upload.wikimedia.org/wikipedia/commons/1/10/Haruo_Sat%C5%8D_-_Kokoro-kayo_wa_zaru-bi_ni.ogg".to_owned(),
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

// use rodio::{Decoder, OutputStream, Sink};
// use std::thread;
// use std::io::Cursor;

// use std::fs::File;
use rodio::{Decoder, OutputStream};
// use std::io::BufReader;
use std::io::Cursor;
// use rodio::Source;
use rodio::Sink;
use std::thread;
use std::time::Duration;


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
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("vinyl - rusty radio");

            ui.text_edit_singleline(&mut self.stream_url);

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            if ui.button("Play").clicked() {
                // let file = File::open("examples/Imperial_Rescript_on_the_Termination_of_the_War_(full_broadcast).ogg").unwrap();
                // let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
                // let sink = rodio::Sink::try_new(&stream_handle).unwrap();
                // let source = Decoder::new(BufReader::new(file)).unwrap();
                // sink.append(source);
                // sink.sleep_until_end();

                // Remember to add the "blocking" feature in the Cargo.toml for reqwest
                // let resp = reqwest::blocking::get("https://upload.wikimedia.org/wikipedia/commons/e/ed/Imperial_Rescript_on_the_Termination_of_the_War_%28full_broadcast%29.ogg")
                //     .unwrap();
                // let cursor = Cursor::new(resp.bytes().unwrap()); // Adds Read and Seek to the bytes via Cursor
                // let source = rodio::Decoder::new(cursor).unwrap(); // Decoder requires it's source to impl both Read and Seek
                // let (_stream, stream_handle) = OutputStream::try_default().unwrap();
                // let sink = Sink::try_new(&stream_handle).unwrap();
                // // let source = Decoder::new(cursor).unwrap();
                // sink.append(source);
                // sink.sleep_until_end();


use std::io::{Read, Cursor};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use reqwest::blocking::Client;
use rodio::{Decoder, OutputStream, Sink};

let stream_url = self.stream_url.clone();

thread::spawn(move || {
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

            // Continue buffering in background
            thread::spawn(move || {
                while let Ok(n) = response.read(&mut temp) {
                    if n == 0 {
                        break;
                    }
                    // You could extend the sink here with more audio if rodio supported it
                    // But since Decoder doesn't support streaming, we can't append more
                }
                println!("Finished buffering.");
            });

            sink.sleep_until_end();
            println!("Playback finished.");
        }
        Err(e) => {
            eprintln!("Failed to decode stream: {}", e);
        }
    }
});



            }

            ui.separator();

            ui.label(format!("Current stream URL: {}", self.stream_url));

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/emilk/eframe_template/blob/main/",
                "Source code."
            ));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
