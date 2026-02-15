use super::Tool;
use eframe::egui::{self, Color32, ColorImage, TextureHandle};
use qrcode::QrCode;

#[derive(Default)]
pub struct QrCodeGenerator {
    text: String,
    qr_texture: Option<TextureHandle>,
    error: Option<String>,
}

impl Tool for QrCodeGenerator {
    fn name(&self) -> &'static str {
        "QR Code Generator"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::QR_CODE
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(400.0)
            .default_height(500.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("QR Code Generator");
                    ui.separator();

                    ui.label("Content:");
                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.text)
                            .hint_text("Type text or URL here...")
                            .desired_rows(3)
                            .desired_width(f32::INFINITY),
                    );

                    if response.changed() {
                        self.generate(ctx);
                    }

                    if let Some(err) = &self.error {
                        ui.colored_label(Color32::RED, err);
                    }

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    if let Some(texture) = &self.qr_texture {
                        ui.vertical_centered(|ui| {
                            ui.image((texture.id(), texture.size_vec2()));
                        });
                    }
                });
            });
    }
}

impl QrCodeGenerator {
    fn generate(&mut self, ctx: &egui::Context) {
        if self.text.is_empty() {
            self.qr_texture = None;
            self.error = None;
            return;
        }

        match QrCode::new(self.text.as_bytes()) {
            Ok(code) => {
                // Get the colors from the code
                // We'll construct an egui ColorImage manually
                let width = code.width();

                // We scale it up manually to avoid blurriness with linear interpolation if texture filter is not nearest
                // But we set texture filter to Nearest, so 1 pixel per module is fine, let egui scale it up?
                // Actually, a 1-pixel texture might be hard to see if small.
                // Let's create a reasonable size image.
                let scale = 4;
                let display_width = width * scale;

                let mut pixels = Vec::with_capacity(display_width * display_width);

                for y in 0..width {
                    for _ in 0..scale {
                        for x in 0..width {
                            let color = match code[(x, y)] {
                                qrcode::Color::Dark => Color32::BLACK,
                                qrcode::Color::Light => Color32::WHITE,
                            };
                            for _ in 0..scale {
                                pixels.push(color);
                            }
                        }
                    }
                }

                let image = ColorImage {
                    size: [display_width, display_width],
                    pixels,
                    ..Default::default()
                };

                self.qr_texture =
                    Some(ctx.load_texture("qr_code_texture", image, egui::TextureOptions::NEAREST));
                self.error = None;
            }
            Err(e) => {
                self.qr_texture = None;
                self.error = Some(format!("Error generating QR code: {e}"));
            }
        }
    }
}
