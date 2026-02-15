use crate::tools::markdown_notes::MarkdownNotes;
use crate::tools::{self, Tool};

struct ToolState {
    tool: Box<dyn Tool>,
    open: bool,
}

#[derive(Default, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
enum AppMode {
    #[default]
    Tools,
    NoteDown,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PersonarsApp {
    #[serde(skip)]
    tools: Vec<ToolState>,
    #[serde(skip)]
    markdown_notes: MarkdownNotes,
    mode: AppMode,
}

impl Default for PersonarsApp {
    fn default() -> Self {
        Self {
            tools: Self::create_tools(),
            markdown_notes: MarkdownNotes::default(),
            mode: AppMode::default(),
        }
    }
}

impl PersonarsApp {
    fn create_tools() -> Vec<ToolState> {
        vec![
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
            ToolState {
                tool: Box::new(tools::regex_tester::RegexTester::default()),
                open: false,
            },
            ToolState {
                tool: Box::new(tools::qr_code_generator::QrCodeGenerator::default()),
                open: false,
            },
            ToolState {
                tool: Box::new(tools::diff_viewer::DiffViewer::default()),
                open: false,
            },
            ToolState {
                tool: Box::new(tools::todo_list::TodoList::default()),
                open: false,
            },
        ]
    }
}

impl PersonarsApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
        cc.egui_ctx.set_fonts(fonts);

        if let Some(storage) = cc.storage {
            let mut app: Self = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // Re-initialize tools since they are skipped in serde
            app.tools = Self::create_tools();
            app.markdown_notes = MarkdownNotes::default();
            app
        } else {
            Default::default()
        }
    }

    fn render_sidebar(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("side_panel")
            .resizable(true)
            .default_width(100.0)
            .width_range(60.0..=300.0)
            .show(ctx, |ui| {
                ui.add_space(10.0);

                let collapsed = ui.available_width() < 100.0;

                ui.vertical_centered(|ui| {
                    if collapsed {
                        ui.label("⚒");
                    } else {
                        ui.label("Tools");
                    }
                });

                ui.separator();
                ui.add_space(5.0);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    for tool_state in &mut self.tools {
                        let icon = tool_state.tool.icon_name();
                        let name = tool_state.tool.name();

                        let text = if collapsed {
                            egui::RichText::new(icon).size(12.0)
                        } else {
                            egui::RichText::new(format!("{icon}  {name}")).size(10.0)
                        };

                        let btn = egui::Button::new(text).selected(tool_state.open);

                        if ui.add_sized([ui.available_width(), 0.0], btn).clicked() {
                            tool_state.open = !tool_state.open;
                        }

                        ui.add_space(4.0);
                    }
                });

                ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                    ui.add_space(10.0);
                    let close_text = if collapsed { "❌" } else { "❌ Close All" };
                    if ui.button(close_text).clicked() {
                        for t in &mut self.tools {
                            t.open = false;
                        }
                    }
                    ui.separator();
                });
            });
    }

    fn render_dashboard(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(50.0);
            ui.heading(egui::RichText::new("Welcome to Personars").size(32.0));
            ui.label(egui::RichText::new("Select a tool to get started").size(18.0));
            ui.add_space(40.0);

            let item_width = 160.0;
            let spacing = 20.0;
            let available_width = ui.available_width();
            let max_columns =
                ((available_width + spacing) / (item_width + spacing)).floor() as usize;
            let max_columns = max_columns.max(1);

            let grid_width = max_columns as f32 * (item_width + spacing) - spacing;
            let margin_left = (available_width - grid_width) / 2.0;

            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.add_space(margin_left.max(0.0));
                    egui::Grid::new("dashboard_grid")
                        .spacing([spacing, spacing])
                        .show(ui, |ui| {
                            for (i, tool_state) in self.tools.iter_mut().enumerate() {
                                let icon = tool_state.tool.icon_name();
                                let name = tool_state.tool.name();

                                ui.vertical(|ui| {
                                    let btn = egui::Button::new(
                                        egui::RichText::new(format!("{icon}\n\n{name}"))
                                            .size(18.0)
                                            .heading(),
                                    )
                                    .min_size(egui::vec2(item_width, 120.0));

                                    if ui.add(btn).clicked() {
                                        tool_state.open = true;
                                    }
                                });

                                if (i + 1) % max_columns == 0 {
                                    ui.end_row();
                                }
                            }
                        });
                });
            });
        });
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
                ui.label("Personars");

                ui.separator();

                // Mode toggle buttons
                let tools_btn = egui::Button::new(
                    egui::RichText::new(format!("{} Tools", egui_phosphor::regular::WRENCH))
                        .size(12.0),
                )
                .selected(self.mode == AppMode::Tools);
                if ui.add(tools_btn).clicked() {
                    self.mode = AppMode::Tools;
                }

                let notedown_btn = egui::Button::new(
                    egui::RichText::new(format!("{} Note Down", egui_phosphor::regular::NOTEBOOK))
                        .size(12.0),
                )
                .selected(self.mode == AppMode::NoteDown);
                if ui.add(notedown_btn).clicked() {
                    self.mode = AppMode::NoteDown;
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            });
        });

        match self.mode {
            AppMode::Tools => {
                self.render_sidebar(ctx);

                egui::CentralPanel::default().show(ctx, |ui| {
                    let central_rect = ui.available_rect_before_wrap();

                    let any_open = self.tools.iter().any(|t| t.open);
                    if !any_open {
                        self.render_dashboard(ui);
                    }

                    for tool_state in &mut self.tools {
                        if tool_state.open {
                            tool_state
                                .tool
                                .show(ctx, &mut tool_state.open, central_rect);
                        }
                    }
                });
            }
            AppMode::NoteDown => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    self.markdown_notes.show_fullscreen(ui);
                });
            }
        }
    }
}
