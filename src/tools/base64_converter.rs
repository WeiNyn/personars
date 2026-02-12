use super::Tool;
use base64::{Engine as _, engine::general_purpose};
use eframe::egui;

#[derive(Default)]
pub struct Base64Converter {
    input: String,
    output: String,
    url_safe: bool,
    error: Option<String>,
}

impl Tool for Base64Converter {
    fn name(&self) -> &'static str {
        "Base64 Converter"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::CODE
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(600.0)
            .default_height(400.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Header
                    ui.horizontal(|ui| {
                        ui.label("Base64 Converter");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear All").clicked() {
                                self.input.clear();
                                self.output.clear();
                                self.error = None;
                            }
                        });
                    });

                    ui.separator();

                    // Input
                    ui.label("Input:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.input)
                            .hint_text("Paste text here...")
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );

                    ui.add_space(5.0);

                    // Controls
                    ui.horizontal(|ui| {
                        if ui.button("Encode").clicked() {
                            self.encode();
                        }
                        if ui.button("Decode").clicked() {
                            self.decode();
                        }
                        ui.checkbox(&mut self.url_safe, "URL Safe");
                    });

                    if let Some(err) = &self.error {
                        ui.colored_label(egui::Color32::RED, err);
                    }

                    ui.add_space(5.0);

                    // Output
                    ui.horizontal(|ui| {
                        ui.label("Output:");
                        if ui.button("Copy").clicked() && !self.output.is_empty() {
                            ui.output_mut(|o| {
                                o.commands
                                    .push(egui::OutputCommand::CopyText(self.output.clone()));
                            });
                        }
                    });

                    ui.add(
                        egui::TextEdit::multiline(&mut self.output)
                            .interactive(false)
                            .desired_rows(5)
                            .desired_width(f32::INFINITY),
                    );
                });
            });
    }
}

impl Base64Converter {
    fn encode(&mut self) {
        let engine = if self.url_safe {
            general_purpose::URL_SAFE
        } else {
            general_purpose::STANDARD
        };

        self.output = engine.encode(&self.input);
        self.error = None;
    }

    fn decode(&mut self) {
        let trimmed = self.input.trim();
        let engine = if self.url_safe {
            general_purpose::URL_SAFE
        } else {
            general_purpose::STANDARD
        };

        match engine.decode(trimmed) {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => {
                    self.output = s;
                    self.error = None;
                }
                Err(_) => {
                    self.error = Some("Decoded bytes are not valid UTF-8".to_owned());
                }
            },
            Err(e) => {
                self.error = Some(format!("Decode Error: {e}"));
            }
        }
    }
}
