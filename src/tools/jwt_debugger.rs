use super::Tool;
use base64::{Engine as _, engine::general_purpose};
use eframe::egui;

#[derive(Default)]
pub struct JwtDebugger {
    input_token: String,
    header_json: String,
    payload_json: String,
    signature: String,
    error: Option<String>,
}

impl Tool for JwtDebugger {
    fn name(&self) -> &'static str {
        "JWT Debugger"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::SHIELD_CHECK
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(600.0)
            .default_height(500.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("JWT Debugger");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear All").clicked() {
                                self.clear();
                            }
                        });
                    });
                    ui.separator();

                    ui.label("Encoded Token:");
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.input_token)
                            .hint_text("Paste JWT here...")
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );

                    if response.changed() {
                        self.decode();
                    }

                    if let Some(err) = &self.error {
                        ui.colored_label(egui::Color32::RED, err);
                    }

                    ui.separator();

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.heading("Header");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.header_json)
                                .code_editor()
                                .interactive(false)
                                .desired_rows(4)
                                .desired_width(f32::INFINITY),
                        );

                        ui.add_space(5.0);
                        ui.heading("Payload");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.payload_json)
                                .code_editor()
                                .interactive(false)
                                .desired_rows(10)
                                .desired_width(f32::INFINITY),
                        );

                        ui.add_space(5.0);
                        ui.heading("Signature");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.signature)
                                .code_editor()
                                .interactive(false)
                                .desired_rows(2)
                                .desired_width(f32::INFINITY),
                        );
                    });
                });
            });
    }
}

impl JwtDebugger {
    fn clear(&mut self) {
        self.input_token.clear();
        self.header_json.clear();
        self.payload_json.clear();
        self.signature.clear();
        self.error = None;
    }

    fn decode(&mut self) {
        if self.input_token.trim().is_empty() {
            self.header_json.clear();
            self.payload_json.clear();
            self.signature.clear();
            self.error = None;
            return;
        }

        let parts: Vec<&str> = self.input_token.trim().split('.').collect();
        if let [header, payload, signature] = parts.as_slice() {
            self.error = None;
            self.header_json = Self::decode_part(header);
            self.payload_json = Self::decode_part(payload);
            self.signature = (*signature).to_owned();
        } else {
            self.error =
                Some("Invalid JWT format (must have 3 parts separated by dots)".to_owned());
        }
    }

    fn decode_part(part: &str) -> String {
        // JWT uses Base64Url (no padding usually, or sometimes padding)
        // We try strictly first, then loose
        let engine = general_purpose::URL_SAFE_NO_PAD;

        match engine.decode(part) {
            Ok(bytes) => {
                if let Ok(json_str) = String::from_utf8(bytes) {
                    // Try to pretty print if it is valid JSON
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        serde_json::to_string_pretty(&val).unwrap_or(json_str)
                    } else {
                        json_str
                    }
                } else {
                    "Error: Invalid UTF-8".to_owned()
                }
            }
            Err(e) => format!("Error decoding Base64: {e}"),
        }
    }
}
