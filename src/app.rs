use crate::tools::{self, Tool};

struct ToolState {
    tool: Box<dyn Tool>,
    open: bool,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PersonarsApp {
    // No longer just an index, but a list of states
    // We don't persist the tools themselves for now, just recreate them
    #[serde(skip)]
    tools: Vec<ToolState>,
}

impl Default for PersonarsApp {
    fn default() -> Self {
        Self {
            tools: vec![
                ToolState {
                    tool: Box::new(tools::format_converter::FormatConverter::default()),
                    open: true,
                },
                ToolState {
                    tool: Box::new(tools::epoch_converter::EpochConverter::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::base64_converter::Base64Converter::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::jwt_debugger::JwtDebugger::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::uuid_generator::UuidGenerator::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::hash_generator::HashGenerator::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::password_generator::PasswordGenerator::default()),
                    open: false,
                },
            ],
        }
    }
}

impl PersonarsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            let mut app: PersonarsApp =
                eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Re-initialize tools since they are skipped in serde
            app.tools = vec![
                ToolState {
                    tool: Box::new(tools::format_converter::FormatConverter::default()),
                    open: true,
                },
                ToolState {
                    tool: Box::new(tools::epoch_converter::EpochConverter::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::base64_converter::Base64Converter::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::jwt_debugger::JwtDebugger::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::uuid_generator::UuidGenerator::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::hash_generator::HashGenerator::default()),
                    open: false,
                },
                ToolState {
                    tool: Box::new(tools::password_generator::PasswordGenerator::default()),
                    open: false,
                },
            ];
            app
        } else {
            Default::default()
        }
    }
}

impl eframe::App for PersonarsApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.label("Personars - Personal Tools");
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Tools");
            ui.separator();

            for tool_state in &mut self.tools {
                if ui
                    .add_sized(
                        [ui.available_width(), 0.0],
                        egui::Button::new(tool_state.tool.name()).selected(tool_state.open),
                    )
                    .clicked()
                {
                    tool_state.open = !tool_state.open;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Active Tools Workspace");
            ui.label("Open tools from the sidebar to see them as windows.");
            ui.separator();

            // Render all open tools as windows
            // The windows themselves manage their position/size within the Context (relative to screen)
            // But we call them here. Note that show() adds windows to the context.
            // We don't strictly need to be inside CentralPanel for windows, but it's fine.
            // Actually, windows are usually added to Context directly.
            // If we put them in CentralPanel, they might be constrained?
            // Regular usage: Window::new(...).show(ctx, ...)
            // We moved ctx usage into Tool::show, so just need to call it.

            let central_rect = ui.available_rect_before_wrap();

            // Render tool windows outside of CentralPanel to allow them to float freely?
            // Actually, windows are independent of panels usually.
            for tool_state in &mut self.tools {
                if tool_state.open {
                    tool_state
                        .tool
                        .show(ctx, &mut tool_state.open, central_rect);
                }
            }
        });
    }
}
