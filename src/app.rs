use crate::tools::finance_tracker::FinanceTracker;
use crate::tools::markdown_notes::MarkdownNotes;
use crate::tools::{Tool as _, ToolKind};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ToolState {
    pub tool: ToolKind,
    pub open: bool,
}

#[derive(Default, Clone, Copy, PartialEq, serde::Deserialize, serde::Serialize)]
enum AppMode {
    #[default]
    Tools,
    NoteDown,
    Finance,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PersonarsApp {
    tools: Vec<ToolState>,
    markdown_notes: MarkdownNotes,
    finance_tracker: FinanceTracker,
    mode: AppMode,
    #[serde(skip)]
    show_about: bool,
    #[serde(skip)]
    sidebar_open: bool,
}

impl Default for PersonarsApp {
    fn default() -> Self {
        Self {
            tools: Self::create_tools(),
            markdown_notes: MarkdownNotes::default(),
            finance_tracker: FinanceTracker::default(),
            mode: AppMode::default(),
            show_about: false,
            sidebar_open: false,
        }
    }
}

impl PersonarsApp {
    fn create_tools() -> Vec<ToolState> {
        vec![
            ToolState {
                tool: ToolKind::FormatConverter(Default::default()),
                open: true,
            },
            ToolState {
                tool: ToolKind::EpochConverter(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::Base64Converter(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::JwtDebugger(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::UuidGenerator(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::HashGenerator(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::PasswordGenerator(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::RegexTester(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::QrCodeGenerator(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::DiffViewer(Default::default()),
                open: false,
            },
            ToolState {
                tool: ToolKind::TodoList(Default::default()),
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

        #[cfg_attr(
            not(target_arch = "wasm32"),
            expect(unused_mut, reason = "mut needed on wasm32 for init_idb")
        )]
        let mut app: Self = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Self::default()
        };

        #[cfg(target_arch = "wasm32")]
        app.finance_tracker.init_idb();

        app
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

    fn render_dashboard(&mut self, ui: &mut egui::Ui, compact: bool) {
        ui.vertical_centered(|ui| {
            ui.add_space(if compact { 20.0 } else { 50.0 });
            ui.heading(
                egui::RichText::new("Welcome to Personars").size(if compact { 22.0 } else { 32.0 }),
            );
            ui.label(
                egui::RichText::new("Select a tool to get started").size(if compact {
                    14.0
                } else {
                    18.0
                }),
            );
            ui.add_space(if compact { 20.0 } else { 40.0 });

            let item_width = if compact { 100.0 } else { 160.0 };
            let item_height = if compact { 80.0 } else { 120.0 };
            let icon_size = if compact { 14.0 } else { 18.0 };
            let spacing = if compact { 10.0 } else { 20.0 };
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
                                            .size(icon_size)
                                            .heading(),
                                    )
                                    .min_size(egui::vec2(item_width, item_height));

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
        let compact = ctx.viewport_rect().width() < 500.0;

        self.render_top_bar(ctx, compact);

        match self.mode {
            AppMode::Tools => self.render_tools_mode(ctx, compact),
            AppMode::NoteDown => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if compact {
                        self.markdown_notes.show_narrow(ui);
                    } else {
                        self.markdown_notes.render_layout(ui);
                    }
                });
            }
            AppMode::Finance => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if compact {
                        self.finance_tracker.show_narrow(ui);
                    } else {
                        self.finance_tracker.render_layout(ui);
                    }
                });
            }
        }

        self.render_about(ctx);
    }
}

impl PersonarsApp {
    fn render_top_bar(&mut self, ctx: &egui::Context, compact: bool) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.label("Personars");
                ui.separator();

                if compact {
                    self.render_top_bar_compact(ui);
                } else {
                    self.render_top_bar_desktop(ui);
                }
            });
        });
    }

    fn render_top_bar_compact(&mut self, ui: &mut egui::Ui) {
        ui.menu_button("☰", |ui| {
            let tools_btn = egui::Button::new(
                egui::RichText::new(format!("{} Tools", egui_phosphor::regular::WRENCH)).size(12.0),
            )
            .selected(self.mode == AppMode::Tools);
            if ui.add(tools_btn).clicked() {
                self.mode = AppMode::Tools;
                ui.close();
            }

            let notedown_btn = egui::Button::new(
                egui::RichText::new(format!("{} Note Down", egui_phosphor::regular::NOTEBOOK))
                    .size(12.0),
            )
            .selected(self.mode == AppMode::NoteDown);
            if ui.add(notedown_btn).clicked() {
                self.mode = AppMode::NoteDown;
                ui.close();
            }

            let finance_btn = egui::Button::new(
                egui::RichText::new(format!(
                    "{} Finance",
                    egui_phosphor::regular::CURRENCY_CIRCLE_DOLLAR
                ))
                .size(12.0),
            )
            .selected(self.mode == AppMode::Finance);
            if ui.add(finance_btn).clicked() {
                self.mode = AppMode::Finance;
                ui.close();
            }

            ui.separator();
            egui::widgets::global_theme_preference_buttons(ui);
            ui.separator();

            if ui
                .button(
                    egui::RichText::new(format!("{} About", egui_phosphor::regular::INFO))
                        .size(12.0),
                )
                .clicked()
            {
                self.show_about = !self.show_about;
                ui.close();
            }
        });
    }

    fn render_top_bar_desktop(&mut self, ui: &mut egui::Ui) {
        let tools_btn = egui::Button::new(
            egui::RichText::new(format!("{} Tools", egui_phosphor::regular::WRENCH)).size(12.0),
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

        let finance_btn = egui::Button::new(
            egui::RichText::new(format!(
                "{} Finance",
                egui_phosphor::regular::CURRENCY_CIRCLE_DOLLAR
            ))
            .size(12.0),
        )
        .selected(self.mode == AppMode::Finance);
        if ui.add(finance_btn).clicked() {
            self.mode = AppMode::Finance;
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            egui::widgets::global_theme_preference_buttons(ui);
            ui.separator();
            if ui
                .button(
                    egui::RichText::new(format!("{} About", egui_phosphor::regular::INFO))
                        .size(12.0),
                )
                .clicked()
            {
                self.show_about = !self.show_about;
            }
        });
    }

    fn render_tools_mode(&mut self, ctx: &egui::Context, compact: bool) {
        if compact {
            let first_open_idx = self.tools.iter().position(|t| t.open);

            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(idx) = first_open_idx {
                    if let Some(tool_state) = self.tools.get_mut(idx) {
                        ui.horizontal(|ui| {
                            if ui.button("← Back").clicked() {
                                tool_state.open = false;
                            }
                            ui.separator();
                            ui.label(
                                egui::RichText::new(format!(
                                    "{} {}",
                                    tool_state.tool.icon_name(),
                                    tool_state.tool.name()
                                ))
                                .strong(),
                            );
                        });
                        ui.separator();
                        tool_state.tool.show_narrow(ui);
                    }
                } else {
                    self.render_dashboard(ui, true);
                }
            });
        } else {
            self.render_sidebar(ctx);

            egui::CentralPanel::default().show(ctx, |ui| {
                let central_rect = ui.available_rect_before_wrap();

                let any_open = self.tools.iter().any(|t| t.open);
                if !any_open {
                    self.render_dashboard(ui, false);
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
    }

    fn render_about(&mut self, ctx: &egui::Context) {
        if self.show_about {
            egui::Window::new(format!("{} About Personars", egui_phosphor::regular::INFO))
                .open(&mut self.show_about)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(10.0);
                        ui.heading(egui::RichText::new("Personars").size(24.0).strong());
                        ui.add_space(5.0);
                        ui.label(
                            egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                .size(14.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.add_space(10.0);
                        ui.label("A personal developer toolbox built with egui.");
                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(5.0);
                        ui.label("Rust Edition: 2024");
                        ui.label("Framework: eframe/egui 0.33");
                        ui.add_space(10.0);
                    });
                });
        }
    }
}
