use super::Tool;
use eframe::egui::{self, Color32, TextFormat};
use similar::{ChangeTag, TextDiff};

pub struct DiffViewer {
    original: String,
    modified: String,
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
                ui.vertical(|ui| {
                    ui.label("Diff Viewer");
                    ui.separator();

                    // Split view for inputs
                    ui.columns(2, |columns| {
                        #[expect(clippy::indexing_slicing)]
                        columns[0].vertical(|ui| {
                            ui.label("Original:");
                            if ui
                                .add(
                                    egui::TextEdit::multiline(&mut self.original)
                                        .hint_text("Paste original text here...")
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(10)
                                        .code_editor(),
                                )
                                .changed()
                            {
                                self.compute_diff(ui);
                            }
                        });

                        #[expect(clippy::indexing_slicing)]
                        columns[1].vertical(|ui| {
                            ui.label("Modified:");
                            if ui
                                .add(
                                    egui::TextEdit::multiline(&mut self.modified)
                                        .hint_text("Paste modified text here...")
                                        .desired_width(f32::INFINITY)
                                        .desired_rows(10)
                                        .code_editor(),
                                )
                                .changed()
                            {
                                self.compute_diff(ui);
                            }
                        });
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.label("Diff Output:");
                    ui.separator();

                    // Initial computation if needed
                    if self.diff_output_job.is_none() {
                        self.compute_diff(ui);
                    }

                    if let Some(job) = &self.diff_output_job {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            ui.label(job.clone());
                        });
                    }
                });
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
