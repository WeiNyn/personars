use super::Tool;
use eframe::egui;
use uuid::Uuid;

#[derive(PartialEq, Clone, Copy)]
enum UuidVersion {
    V4,
    V7,
}

pub struct UuidGenerator {
    version: UuidVersion,
    count: usize,
    uppercase: bool,
    output: String,
}

impl Default for UuidGenerator {
    fn default() -> Self {
        Self {
            version: UuidVersion::V4,
            count: 1,
            uppercase: false,
            output: String::new(),
        }
    }
}

impl Tool for UuidGenerator {
    fn name(&self) -> &'static str {
        "UUID Generator"
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(self.name())
            .open(open)
            .default_width(400.0)
            .default_height(400.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("UUID Generator");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Clear").clicked() {
                                self.output.clear();
                            }
                        });
                    });
                    ui.separator();

                    // Configuration
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label("Version:");
                            egui::ComboBox::from_id_salt("uuid_version")
                                .selected_text(match self.version {
                                    UuidVersion::V4 => "v4 (Random)",
                                    UuidVersion::V7 => "v7 (Time-ordered)",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.version,
                                        UuidVersion::V4,
                                        "v4 (Random)",
                                    );
                                    ui.selectable_value(
                                        &mut self.version,
                                        UuidVersion::V7,
                                        "v7 (Time-ordered)",
                                    );
                                });
                        });

                        ui.horizontal(|ui| {
                            ui.label("Count:");
                            ui.add(egui::Slider::new(&mut self.count, 1..=100));
                        });

                        ui.checkbox(&mut self.uppercase, "Uppercase");

                        ui.add_space(5.0);

                        if ui.button("Generate").clicked() {
                            self.generate();
                        }
                    });

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
                            .desired_rows(10)
                            .desired_width(f32::INFINITY),
                    );
                });
            });
    }
}

impl UuidGenerator {
    fn generate(&mut self) {
        let mut results = Vec::with_capacity(self.count);

        for _ in 0..self.count {
            let uuid = match self.version {
                UuidVersion::V4 => Uuid::new_v4(),
                UuidVersion::V7 => Uuid::now_v7(),
            };

            let mut s = uuid.to_string();
            if self.uppercase {
                s = s.to_uppercase();
            }
            results.push(s);
        }

        self.output = results.join("\n");
    }
}
