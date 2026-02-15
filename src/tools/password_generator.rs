use super::Tool;
use eframe::egui;
use rand::RngExt as _;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct PasswordGenerator {
    length: usize,
    use_upper: bool,
    use_lower: bool,
    use_numbers: bool,
    use_symbols: bool,
    output: String,
}

impl Default for PasswordGenerator {
    fn default() -> Self {
        Self {
            length: 16,
            use_upper: true,
            use_lower: true,
            use_numbers: true,
            use_symbols: true,
            output: String::new(),
        }
    }
}

impl Tool for PasswordGenerator {
    fn name(&self) -> &'static str {
        "Password Generator"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::LOCK_KEY
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(400.0)
            .default_height(300.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
    }

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_content(ui);
        });
    }
}

impl PasswordGenerator {
    fn render_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Password Generator");
            });
            ui.separator();

            ui.group(|ui| {
                ui.label("Configuration");

                ui.horizontal(|ui| {
                    ui.label("Length:");
                    ui.add(egui::Slider::new(&mut self.length, 4..=128));
                });

                ui.checkbox(&mut self.use_upper, "Uppercase (A-Z)");
                ui.checkbox(&mut self.use_lower, "Lowercase (a-z)");
                ui.checkbox(&mut self.use_numbers, "Numbers (0-9)");
                ui.checkbox(&mut self.use_symbols, "Symbols (!@#$...)");

                ui.add_space(5.0);

                let can_generate =
                    self.use_upper || self.use_lower || self.use_numbers || self.use_symbols;

                if ui
                    .add_enabled(can_generate, egui::Button::new("Generate"))
                    .clicked()
                {
                    self.generate();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(5.0);

            ui.horizontal(|ui| {
                ui.label("Result:");
                if ui.button("Copy").clicked() && !self.output.is_empty() {
                    ui.output_mut(|o| {
                        o.commands
                            .push(egui::OutputCommand::CopyText(self.output.clone()));
                    });
                }
            });

            ui.add(
                egui::TextEdit::singleline(&mut self.output)
                    .hint_text("Password will appear here...")
                    .desired_width(f32::INFINITY),
            );
        });
    }
}

impl PasswordGenerator {
    fn generate(&mut self) {
        const UPPER: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        const LOWER: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
        const NUMBERS: &[u8] = b"0123456789";
        const SYMBOLS: &[u8] = b"!@#$%^&*()-_=+[]{}|;:,.<>?";

        let mut charset = Vec::new();
        if self.use_upper {
            charset.extend_from_slice(UPPER);
        }
        if self.use_lower {
            charset.extend_from_slice(LOWER);
        }
        if self.use_numbers {
            charset.extend_from_slice(NUMBERS);
        }
        if self.use_symbols {
            charset.extend_from_slice(SYMBOLS);
        }

        if charset.is_empty() {
            return;
        }

        let mut rng = rand::rng();
        let password: String = (0..self.length)
            .map(|_| {
                let idx = rng.random_range(0..charset.len());
                *charset.get(idx).expect("charset is empty") as char
            })
            .collect();

        self.output = password;
    }
}
