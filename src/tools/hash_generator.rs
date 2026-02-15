use super::Tool;
use eframe::egui;
use md5::Md5;
use sha1::Sha1;
use sha2::{Digest as _, Sha256, Sha512};

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct HashGenerator {
    input: String,
    md5: String,
    sha1: String,
    sha256: String,
    sha512: String,
}

impl Tool for HashGenerator {
    fn name(&self) -> &'static str {
        "Hash Generator"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::FINGERPRINT
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(600.0)
            .default_height(500.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
    }

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        self.render_content(ui);
    }
}

impl HashGenerator {
    fn render_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Hash Generator");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Clear").clicked() {
                        self.input.clear();
                        self.update_hashes();
                    }
                });
            });
            ui.separator();

            ui.label("Input Text:");
            let response = ui.add(
                egui::TextEdit::multiline(&mut self.input)
                    .hint_text("Type or paste text here to hash...")
                    .desired_rows(4)
                    .desired_width(f32::INFINITY),
            );

            if response.changed() {
                self.update_hashes();
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                Self::render_hash_row(ui, "MD5", &self.md5);
                Self::render_hash_row(ui, "SHA-1", &self.sha1);
                Self::render_hash_row(ui, "SHA-256", &self.sha256);
                Self::render_hash_row(ui, "SHA-512", &self.sha512);
            });
        });
    }
}

impl HashGenerator {
    fn update_hashes(&mut self) {
        if self.input.is_empty() {
            self.md5.clear();
            self.sha1.clear();
            self.sha256.clear();
            self.sha512.clear();
            return;
        }

        let input_bytes = self.input.as_bytes();

        // MD5
        let mut hasher = Md5::new();
        hasher.update(input_bytes);
        self.md5 = hex::encode(hasher.finalize());

        // SHA-1
        let mut hasher = Sha1::new();
        hasher.update(input_bytes);
        self.sha1 = hex::encode(hasher.finalize());

        // SHA-256
        let mut hasher = Sha256::new();
        hasher.update(input_bytes);
        self.sha256 = hex::encode(hasher.finalize());

        // SHA-512
        let mut hasher = Sha512::new();
        hasher.update(input_bytes);
        self.sha512 = hex::encode(hasher.finalize());
    }

    fn render_hash_row(ui: &mut egui::Ui, label: &str, hash: &str) {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(agui_strong_label(label)); // Helper or just strong text
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Copy").clicked() && !hash.is_empty() {
                        ui.output_mut(|o| {
                            o.commands
                                .push(egui::OutputCommand::CopyText(hash.to_owned()));
                        });
                    }
                });
            });
            ui.add(
                egui::TextEdit::multiline(&mut hash.to_owned())
                    .hint_text("Hash will appear here...")
                    .interactive(false)
                    .desired_rows(if label == "SHA-512" { 3 } else { 1 })
                    .desired_width(f32::INFINITY),
            );
        });
        ui.add_space(5.0);
    }
}

fn agui_strong_label(text: &str) -> egui::RichText {
    egui::RichText::new(text).strong()
}
