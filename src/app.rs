use crate::tools::{self, Tool};

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct PersonarsApp {
    // Current validation tool index
    selected_tool_index: usize,

    // We don't persist the tools themselves for now, just recreate them
    #[serde(skip)]
    tools: Vec<Box<dyn Tool>>,
}

impl Default for PersonarsApp {
    fn default() -> Self {
        Self {
            selected_tool_index: 0,
            tools: vec![Box::new(tools::format_converter::FormatConverter::default())],
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
            app.tools = vec![Box::new(tools::format_converter::FormatConverter::default())];
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

            for (i, tool) in self.tools.iter().enumerate() {
                if ui
                    .add_sized(
                        [ui.available_width(), 0.0],
                        egui::Button::new(tool.name()).selected(self.selected_tool_index == i),
                    )
                    .clicked()
                {
                    self.selected_tool_index = i;
                }
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(tool) = self.tools.get_mut(self.selected_tool_index) {
                tool.show(ctx, ui);
            } else {
                ui.label("No tool selected");
            }
        });
    }
}
