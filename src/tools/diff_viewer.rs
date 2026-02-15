use super::Tool;
use eframe::egui::{self, Color32, TextFormat};
use similar::{ChangeTag, TextDiff};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct DiffViewer {
    original: String,
    modified: String,
    #[serde(skip)]
    diff_output_job: Option<egui::text::LayoutJob>,
}

impl Default for DiffViewer {
    fn default() -> Self {
        Self {
            original: "Original text\nWith some lines\nTo be removed".to_owned(),
            modified: "Original text\nWith extra lines\nAdded here".to_owned(),
            diff_output_job: None,
        }
    }
}

impl Tool for DiffViewer {
    fn name(&self) -> &'static str {
        "Diff Viewer"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::GIT_DIFF
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(800.0)
            .default_height(600.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                self.render_content_wide(ui);
            });
    }

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.label("Diff Viewer");
            ui.separator();

            let theme = if ui.style().visuals.dark_mode {
                egui_code_editor::ColorTheme::GITHUB_DARK
            } else {
                egui_code_editor::ColorTheme::GITHUB_LIGHT
            };

            // Vertically stacked editor panes
            ui.label("Original:");
            let r1 = egui_code_editor::CodeEditor::default()
                .id_source("original_narrow")
                .auto_shrink(false)
                .vscroll(true)
                .with_numlines(true)
                .with_rows(8)
                .with_fontsize(12.0)
                .with_theme(theme)
                .show(ui, &mut self.original);
            if r1.response.changed() {
                self.compute_diff(ui);
            }

            ui.add_space(8.0);

            ui.label("Modified:");
            let r2 = egui_code_editor::CodeEditor::default()
                .id_source("modified_narrow")
                .auto_shrink(false)
                .vscroll(true)
                .with_numlines(true)
                .with_rows(8)
                .with_fontsize(12.0)
                .with_theme(theme)
                .show(ui, &mut self.modified);
            if r2.response.changed() {
                self.compute_diff(ui);
            }

            ui.separator();
            ui.label("Diff Output:");
            ui.separator();

            if self.diff_output_job.is_none() {
                self.compute_diff(ui);
            }

            if let Some(job) = &self.diff_output_job {
                ui.label(job.clone());
            }
        });
    }
}

impl DiffViewer {
    fn render_content_wide(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.label("Diff Viewer");
            ui.separator();

            let line_height = egui::TextStyle::Monospace.resolve(ui.style()).size;
            let rows = 20;
            let max_height = line_height * rows as f32;

            // Split view for inputs
            ui.columns(2, |columns| {
                #[expect(clippy::indexing_slicing)]
                columns[0].vertical(|ui| {
                    ui.set_max_height(max_height);
                    ui.label("Original:");
                    let response = egui_code_editor::CodeEditor::default()
                        .id_source("original")
                        .auto_shrink(false)
                        .vscroll(true)
                        .with_numlines(true)
                        .with_rows(17)
                        .with_fontsize(12.0)
                        .with_theme(if ui.style().visuals.dark_mode {
                            egui_code_editor::ColorTheme::GITHUB_DARK
                        } else {
                            egui_code_editor::ColorTheme::GITHUB_LIGHT
                        })
                        .show(ui, &mut self.original);
                    if response.response.changed() {
                        self.compute_diff(ui);
                    }
                });

                #[expect(clippy::indexing_slicing)]
                columns[1].vertical(|ui| {
                    ui.set_max_height(max_height);
                    ui.label("Modified:");
                    let response = egui_code_editor::CodeEditor::default()
                        .id_source("modified")
                        .auto_shrink(false)
                        .vscroll(true)
                        .with_numlines(true)
                        .with_rows(17)
                        .with_fontsize(12.0)
                        .with_theme(if ui.style().visuals.dark_mode {
                            egui_code_editor::ColorTheme::GITHUB_DARK
                        } else {
                            egui_code_editor::ColorTheme::GITHUB_LIGHT
                        })
                        .show(ui, &mut self.modified);
                    if response.response.changed() {
                        self.compute_diff(ui);
                    }
                });
            });

            ui.separator();
            ui.label("Diff Output:");
            ui.separator();

            // Initial computation if needed
            if self.diff_output_job.is_none() {
                self.compute_diff(ui);
            }

            if let Some(job) = &self.diff_output_job {
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.label(job.clone());
                    });
            }
        });
    }
}

impl DiffViewer {
    fn compute_diff(&mut self, ui: &egui::Ui) {
        let diff = TextDiff::from_lines(&self.original, &self.modified);
        let mut job = egui::text::LayoutJob::default();
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());

        for change in diff.iter_all_changes() {
            let (bg_color, text_color) = match change.tag() {
                ChangeTag::Delete => (
                    Some(Color32::from_rgba_unmultiplied(255, 0, 0, 40)), // Reddish background
                    if ui.visuals().dark_mode {
                        Color32::from_rgb(255, 150, 150)
                    } else {
                        Color32::DARK_RED
                    },
                ),
                ChangeTag::Insert => (
                    Some(Color32::from_rgba_unmultiplied(0, 255, 0, 40)), // Greenish background
                    if ui.visuals().dark_mode {
                        Color32::from_rgb(150, 255, 150)
                    } else {
                        Color32::DARK_GREEN
                    },
                ),
                ChangeTag::Equal => (
                    None,
                    if ui.visuals().dark_mode {
                        Color32::LIGHT_GRAY
                    } else {
                        Color32::DARK_GRAY
                    },
                ),
            };

            let text = change.to_string(); // Includes generic newline?
            // similar::Change::to_string() does include the content.
            // If it's from_lines, it typically includes the newline if present in source.

            let mut format = TextFormat {
                font_id: font_id.clone(),
                color: text_color,
                ..Default::default()
            };

            if let Some(bg) = bg_color {
                format.background = bg;
            }

            job.append(&text, 0.0, format);
        }

        self.diff_output_job = Some(job);
    }
}
