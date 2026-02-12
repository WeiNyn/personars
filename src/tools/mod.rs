use eframe::egui;

pub trait Tool {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect);
    fn icon_name(&self) -> &'static str;
}

pub mod base64_converter;
pub mod epoch_converter;
pub mod format_converter;
pub mod hash_generator;
pub mod jwt_debugger;
pub mod password_generator;
pub mod regex_tester;
pub mod uuid_generator;
