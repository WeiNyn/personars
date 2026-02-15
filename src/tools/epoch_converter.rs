use super::Tool;
use chrono::{Local, TimeZone as _, Utc};
use eframe::egui;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct EpochConverter {
    timestamp_input: String,
    date_output: String,
    // For reverse conversion (optional for now, let's stick to timestamp -> date first)
    // or we can make it bidirectional
}

impl Tool for EpochConverter {
    fn name(&self) -> &'static str {
        "Epoch Converter"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::CLOCK
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(250.0)
            .default_height(300.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // 1. Current Time Section
                    ui.group(|ui| {
                        ui.set_width(ui.available_width());
                        ui.heading("Current Time");
                        let now = Local::now();
                        let utc = Utc::now();

                        ui.horizontal(|ui| {
                            ui.label("Unix (s):");
                            ui.code(now.timestamp().to_string());
                            if ui.button("Copy").clicked() {
                                ui.output_mut(|o| {
                                    o.commands.push(egui::OutputCommand::CopyText(
                                        now.timestamp().to_string(),
                                    ));
                                });
                            }
                        });
                        ui.horizontal(|ui| {
                            ui.label("Unix (ms):");
                            ui.code(now.timestamp_millis().to_string());
                        });

                        ui.label(format!("UTC:   {}", utc.format("%Y-%m-%d %H:%M:%S")));
                        ui.label(format!("Local: {}", now.format("%Y-%m-%d %H:%M:%S")));
                    });

                    ui.separator();

                    // 2. Converter Section
                    ui.heading("Convert");

                    ui.horizontal(|ui| {
                        ui.label("Timestamp:");
                        let response = ui.text_edit_singleline(&mut self.timestamp_input);

                        if response.changed() {
                            self.convert();
                        }
                    });

                    if !self.date_output.is_empty() {
                        ui.group(|ui| {
                            ui.label(egui::RichText::new(&self.date_output).monospace());
                        });
                    }

                    if ui.button("Clear").clicked() {
                        self.timestamp_input.clear();
                        self.date_output.clear();
                    }
                });
            });

        // Request repaint to update the clock every second suitable for a "clock" tool
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
    }
}

impl EpochConverter {
    fn convert(&mut self) {
        let trimmed = self.timestamp_input.trim();
        if trimmed.is_empty() {
            self.date_output.clear();
            return;
        }

        if let Ok(ts) = trimmed.parse::<i64>() {
            // Guess if it's seconds or milliseconds
            // Current timestamp in seconds is ~1.7 billion (10 digits)
            // Milliseconds is ~1.7 trillion (13 digits)

            let (seconds, nanos) = if ts.abs() > 100_000_000_000 {
                // Likely milliseconds
                (ts / 1000, (ts % 1000) as u32 * 1_000_000)
            } else {
                // Likely seconds
                (ts, 0)
            };

            if let Some(dt) = Utc.timestamp_opt(seconds, nanos).single() {
                let local = dt.with_timezone(&Local);
                self.date_output = format!(
                    "UTC:   {}\nLocal: {}",
                    dt.format("%Y-%m-%d %H:%M:%S"),
                    local.format("%Y-%m-%d %H:%M:%S")
                );
            } else {
                self.date_output = "Invalid timestamp".to_owned();
            }
        } else {
            self.date_output = "Invalid number".to_owned();
        }
    }
}
