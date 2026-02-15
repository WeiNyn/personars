use eframe::egui;

pub trait Tool {
    fn name(&self) -> &'static str;
    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect);
    fn icon_name(&self) -> &'static str;
}

pub mod base64_converter;
pub mod diff_viewer;
pub mod epoch_converter;
pub mod format_converter;
pub mod hash_generator;
pub mod jwt_debugger;
pub mod markdown_notes;
pub mod password_generator;
pub mod qr_code_generator;
pub mod regex_tester;
pub mod todo_list;
pub mod uuid_generator;

/// Enum wrapping all tool types, enabling serialization of the full tools collection.
#[derive(serde::Deserialize, serde::Serialize)]
pub enum ToolKind {
    FormatConverter(format_converter::FormatConverter),
    EpochConverter(epoch_converter::EpochConverter),
    Base64Converter(base64_converter::Base64Converter),
    JwtDebugger(jwt_debugger::JwtDebugger),
    UuidGenerator(uuid_generator::UuidGenerator),
    HashGenerator(hash_generator::HashGenerator),
    PasswordGenerator(password_generator::PasswordGenerator),
    RegexTester(regex_tester::RegexTester),
    QrCodeGenerator(qr_code_generator::QrCodeGenerator),
    DiffViewer(diff_viewer::DiffViewer),
    TodoList(todo_list::TodoList),
}

/// Delegate `Tool` trait methods to inner tool via match arms.
impl Tool for ToolKind {
    fn name(&self) -> &'static str {
        match self {
            Self::FormatConverter(t) => t.name(),
            Self::EpochConverter(t) => t.name(),
            Self::Base64Converter(t) => t.name(),
            Self::JwtDebugger(t) => t.name(),
            Self::UuidGenerator(t) => t.name(),
            Self::HashGenerator(t) => t.name(),
            Self::PasswordGenerator(t) => t.name(),
            Self::RegexTester(t) => t.name(),
            Self::QrCodeGenerator(t) => t.name(),
            Self::DiffViewer(t) => t.name(),
            Self::TodoList(t) => t.name(),
        }
    }

    fn icon_name(&self) -> &'static str {
        match self {
            Self::FormatConverter(t) => t.icon_name(),
            Self::EpochConverter(t) => t.icon_name(),
            Self::Base64Converter(t) => t.icon_name(),
            Self::JwtDebugger(t) => t.icon_name(),
            Self::UuidGenerator(t) => t.icon_name(),
            Self::HashGenerator(t) => t.icon_name(),
            Self::PasswordGenerator(t) => t.icon_name(),
            Self::RegexTester(t) => t.icon_name(),
            Self::QrCodeGenerator(t) => t.icon_name(),
            Self::DiffViewer(t) => t.icon_name(),
            Self::TodoList(t) => t.icon_name(),
        }
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        match self {
            Self::FormatConverter(t) => t.show(ctx, open, rect),
            Self::EpochConverter(t) => t.show(ctx, open, rect),
            Self::Base64Converter(t) => t.show(ctx, open, rect),
            Self::JwtDebugger(t) => t.show(ctx, open, rect),
            Self::UuidGenerator(t) => t.show(ctx, open, rect),
            Self::HashGenerator(t) => t.show(ctx, open, rect),
            Self::PasswordGenerator(t) => t.show(ctx, open, rect),
            Self::RegexTester(t) => t.show(ctx, open, rect),
            Self::QrCodeGenerator(t) => t.show(ctx, open, rect),
            Self::DiffViewer(t) => t.show(ctx, open, rect),
            Self::TodoList(t) => t.show(ctx, open, rect),
        }
    }
}
