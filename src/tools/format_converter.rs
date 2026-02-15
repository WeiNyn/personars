use super::Tool;
use egui::Color32;
use egui_code_editor::{CodeEditor, ColorTheme};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cmp::max;

#[derive(Serialize, Deserialize, Default, PartialEq, Clone, Copy, Debug)]
enum Format {
    #[default]
    Json,
    Yaml,
    Toml,
}

impl Format {
    fn name(&self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
            Self::Toml => "TOML",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct FormatConverter {
    // Content buffers
    json: String,
    yaml: String,
    toml: String,

    // Active views
    input_format: Format,
    output_format: Format,

    // Error states
    #[serde(skip)]
    error: Option<String>,
}

impl Default for FormatConverter {
    fn default() -> Self {
        Self {
            json: String::new(),
            yaml: String::new(),
            toml: String::new(),
            input_format: Format::Json,
            output_format: Format::Yaml,
            error: None,
        }
    }
}

impl Tool for FormatConverter {
    fn name(&self) -> &'static str {
        "Format Converter"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::FILE_CODE
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(600.0)
            .default_height(400.0)
            .vscroll(true)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    self.render_header(ui);

                    // We are inside a Window now. ui.available_height() is relative to the window content area.
                    // This is much cleaner!
                    let raw_available = ui.available_height();
                    let row_height = ui.text_style_height(&egui::TextStyle::Monospace);

                    // Overhead: Label (~20) + Group Padding (~20).
                    let overhead = 20.0;
                    let target_height = if raw_available > overhead {
                        raw_available - overhead
                    } else {
                        raw_available
                    };
                    let rows = max(10, (target_height / row_height).floor() as usize);

                    // Main Content: 2-Pane Layout
                    ui.horizontal(|ui| {
                        let available_width = ui.available_width();
                        let pane_width = (available_width / 2.0) - 8.0; // Subtract spacing

                        self.render_input_pane(ui, rows, pane_width, target_height);
                        self.render_output_pane(ui, rows, pane_width, target_height);
                    });
                });
            });
    }

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            self.render_header(ui);

            let pane_width = ui.available_width();
            let rows = 8; // Fixed compact row count

            // Vertical stacking instead of horizontal
            self.render_input_pane(ui, rows, pane_width, 200.0);
            ui.add_space(8.0);
            self.render_output_pane(ui, rows, pane_width, 200.0);
        });
    }
}

impl FormatConverter {
    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Format Converter");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear All").clicked() {
                    self.json.clear();
                    self.yaml.clear();
                    self.toml.clear();
                    self.error = None;
                }
            });
        });
    }

    fn render_input_pane(&mut self, ui: &mut egui::Ui, rows: usize, width: f32, height: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_min_height(height); // Force the pane to be tall
            ui.group(|ui| {
                ui.set_min_height(height); // Force the group to be tall

                ui.horizontal(|ui| {
                    ui.label("Input:");
                    egui::ComboBox::from_id_salt("input_format")
                        .selected_text(self.input_format.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.input_format, Format::Json, "JSON");
                            ui.selectable_value(&mut self.input_format, Format::Yaml, "YAML");
                            ui.selectable_value(&mut self.input_format, Format::Toml, "TOML");
                        });

                    if ui.button("Format Input").clicked() {
                        self.format_current();
                    }
                });

                let current_text = match self.input_format {
                    Format::Json => &mut self.json,
                    Format::Yaml => &mut self.yaml,
                    Format::Toml => &mut self.toml,
                };

                let theme = if ui.visuals().dark_mode {
                    ColorTheme::GITHUB_DARK
                } else {
                    ColorTheme::GITHUB_LIGHT
                };

                let response = CodeEditor::default()
                    .id_source("input_editor")
                    .with_rows(rows)
                    .with_fontsize(13.0)
                    .with_theme(theme)
                    .auto_shrink(false)
                    .show(ui, current_text);

                // Auto-convert on change
                if response.response.changed() {
                    self.convert_from_input();
                }
            });
        });
    }

    fn render_output_pane(&mut self, ui: &mut egui::Ui, rows: usize, width: f32, height: f32) {
        ui.vertical(|ui| {
            ui.set_width(width);
            ui.set_min_height(height);
            ui.group(|ui| {
                ui.set_min_height(height);
                ui.horizontal(|ui| {
                    ui.label("Output:");
                    egui::ComboBox::from_id_salt("output_format")
                        .selected_text(self.output_format.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.output_format, Format::Json, "JSON");
                            ui.selectable_value(&mut self.output_format, Format::Yaml, "YAML");
                            ui.selectable_value(&mut self.output_format, Format::Toml, "TOML");
                        });

                    if ui.button("Copy").clicked() {
                        let text = match self.output_format {
                            Format::Json => &self.json,
                            Format::Yaml => &self.yaml,
                            Format::Toml => &self.toml,
                        };
                        ui.output_mut(|o| {
                            o.commands.push(egui::OutputCommand::CopyText(text.clone()));
                        });
                    }
                });

                if let Some(err) = &self.error {
                    ui.colored_label(Color32::RED, err);
                } else {
                    // Show output (read-only technically, but we use the same buffer)
                    let current_text = match self.output_format {
                        Format::Json => &mut self.json,
                        Format::Yaml => &mut self.yaml,
                        Format::Toml => &mut self.toml,
                    };

                    let theme = if ui.visuals().dark_mode {
                        ColorTheme::GITHUB_DARK
                    } else {
                        ColorTheme::GITHUB_LIGHT
                    };

                    CodeEditor::default()
                        .id_source("output_editor")
                        .with_rows(rows)
                        .with_fontsize(13.0)
                        .with_theme(theme)
                        .auto_shrink(false)
                        .show(ui, current_text);
                }
            });
        });
    }

    fn convert_from_input(&mut self) {
        let input_text = match self.input_format {
            Format::Json => &self.json,
            Format::Yaml => &self.yaml,
            Format::Toml => &self.toml,
        };

        if input_text.trim().is_empty() {
            self.error = None;
            return;
        }

        // 1. Parse Input -> Value
        let value_result: Result<Value, String> = match self.input_format {
            Format::Json => serde_json::from_str(input_text).map_err(|e| e.to_string()),
            Format::Yaml => serde_yaml::from_str(input_text).map_err(|e| e.to_string()),
            Format::Toml => toml::from_str(input_text).map_err(|e| e.to_string()),
        };

        match value_result {
            Ok(val) => {
                self.error = None;
                // 2. Value -> Other Formats
                // We only *need* to update the others, but keeping all in sync is fine
                if self.input_format != Format::Json {
                    self.json = serde_json::to_string_pretty(&val).unwrap_or_default();
                }
                if self.input_format != Format::Yaml {
                    self.yaml = serde_yaml::to_string(&val).unwrap_or_default();
                }
                if self.input_format != Format::Toml {
                    self.toml = toml::to_string(&val).unwrap_or_default();
                }
            }
            Err(e) => {
                self.error = Some(format!("Parse Error: {e}"));
            }
        }
    }

    fn format_current(&mut self) {
        // Trigger a self-update cycle to pretty print the current input
        self.convert_from_input();
        // If successful, the side-effect is that *all* buffers are regenerated from the parsed Value,
        // including the input one (if we want to force format it).
        // However, convert_from_input doesn't write BACK to input to avoid fighting the user.

        // Let's explicitly re-serialize the input buffer from the parsed value if valid
        let input_text = match self.input_format {
            Format::Json => &self.json,
            Format::Yaml => &self.yaml,
            Format::Toml => &self.toml,
        };

        let value_result: Result<Value, String> = match self.input_format {
            Format::Json => serde_json::from_str(input_text).map_err(|e| e.to_string()),
            Format::Yaml => serde_yaml::from_str(input_text).map_err(|e| e.to_string()),
            Format::Toml => toml::from_str(input_text).map_err(|e| e.to_string()),
        };

        if let Ok(val) = value_result {
            match self.input_format {
                Format::Json => self.json = serde_json::to_string_pretty(&val).unwrap_or_default(),
                Format::Yaml => self.yaml = serde_yaml::to_string(&val).unwrap_or_default(),
                Format::Toml => self.toml = toml::to_string(&val).unwrap_or_default(),
            }
        }
    }
}
