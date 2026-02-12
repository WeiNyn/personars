use eframe::egui;

pub trait Tool {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect);
}

pub mod format_converter;
