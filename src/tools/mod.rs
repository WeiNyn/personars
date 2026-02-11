use eframe::egui::{self, Ui};

pub trait Tool {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui);
}

pub mod format_converter;
