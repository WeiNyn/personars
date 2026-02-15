use super::Tool;
use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32, Layout, RichText};

use egui_commonmark::CommonMarkCache;
use egui_extras::{Size, StripBuilder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Deserialize, Serialize)]
struct Note {
    id: Uuid,
    title: String,
    content: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    modified_at: DateTime<Utc>,
}

#[derive(Deserialize)]
#[serde(default)]
pub struct MarkdownNotes {
    notes: Vec<Note>,
    active_note_id: Option<Uuid>,
    #[serde(skip)]
    search_query: String,
    #[serde(skip)]
    commonmark_cache: CommonMarkCache,
}

// Manual Serialize implementation to skip commonmark_cache
impl Serialize for MarkdownNotes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct as _;
        let mut state = serializer.serialize_struct("MarkdownNotes", 2)?;
        state.serialize_field("notes", &self.notes)?;
        state.serialize_field("active_note_id", &self.active_note_id)?;
        state.end()
    }
}

impl Default for MarkdownNotes {
    fn default() -> Self {
        let welcome_note = Note {
            id: Uuid::new_v4(),
            title: "Welcome Source Note".to_owned(),
            content: "# Welcome to Markdown Notes\n\nThis is a simple note-taking tool.\n\n- Create new notes\n- Edit with syntax highlighting\n- Preview in real-time".to_owned(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };

        Self {
            active_note_id: Some(welcome_note.id),
            notes: vec![welcome_note],
            search_query: String::new(),
            commonmark_cache: CommonMarkCache::default(),
        }
    }
}

impl Tool for MarkdownNotes {
    fn name(&self) -> &'static str {
        "Markdown Notes"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::NOTEBOOK
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(600.0)
            .default_height(600.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                self.render_layout(ui);
            });
    }
}

impl MarkdownNotes {
    /// Full-screen mode: renders directly in the given `Ui` without a window wrapper.
    pub fn show_fullscreen(&mut self, ui: &mut egui::Ui) {
        self.render_layout(ui);
    }

    /// Shared 3-pane layout used by both windowed and full-screen modes.
    fn render_layout(&mut self, ui: &mut egui::Ui) {
        StripBuilder::new(ui)
            .size(Size::relative(0.2).at_least(100.0)) // Sidebar 20%, min 100px
            .size(Size::exact(1.0)) // Separator
            .size(Size::remainder()) // Content
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.set_clip_rect(ui.max_rect());
                    ui.set_min_width(0.0);
                    self.render_sidebar(ui);
                });
                strip.cell(|ui| {
                    ui.add(egui::Separator::default().vertical());
                });
                strip.cell(|ui| {
                    if self.active_note_id.is_some() {
                        StripBuilder::new(ui)
                            .size(Size::relative(0.5).at_least(100.0)) // Editor matches Preview
                            .size(Size::exact(1.0)) // Separator
                            .size(Size::remainder()) // Preview
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.set_clip_rect(ui.max_rect());
                                    ui.set_min_width(0.0);
                                    self.render_editor(ui);
                                });
                                strip.cell(|ui| {
                                    ui.add(egui::Separator::default().vertical());
                                });
                                strip.cell(|ui| {
                                    ui.set_clip_rect(ui.max_rect());
                                    ui.set_min_width(0.0);
                                    self.render_preview(ui);
                                });
                            });
                    } else {
                        ui.centered_and_justified(|ui| {
                            ui.label("Select a note to edit.");
                        });
                    }
                });
            });
    }

    fn render_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            // Header & Controls
            ui.horizontal(|ui| {
                ui.heading("Notes");
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("➕").strong()).clicked() {
                        self.create_new_note();
                    }
                });
            });

            ui.separator();

            // Search Bar
            ui.add(egui::TextEdit::singleline(&mut self.search_query).hint_text("Search..."));
            ui.separator();

            // Note List
            ui.vertical(|ui| {
                let mut delete_id = None;
                let available_width = ui.available_width();

                let meta_width = 85.0;
                let title_width = available_width - meta_width - 8.0;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filtered_notes: Vec<(usize, Uuid, String, String)> = self
                        .notes
                        .iter()
                        .enumerate()
                        .filter(|(_, n)| {
                            self.search_query.is_empty()
                                || n.title
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase())
                                || n.content
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase())
                        })
                        .map(|(i, n)| {
                            (
                                i,
                                n.id,
                                n.title.clone(),
                                n.created_at.format("%Y-%m-%d").to_string(),
                            )
                        })
                        .collect();

                    for (index, id, title, date) in filtered_notes {
                        let is_selected = self.active_note_id == Some(id);

                        // Main container for the list item
                        ui.horizontal(|ui| {
                            // 1. Title Area
                            ui.allocate_ui_with_layout(
                                egui::vec2(title_width, ui.spacing().interact_size.y),
                                Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let button =
                                        egui::Button::new(&title).selected(is_selected).wrap();
                                    if ui.add_sized([title_width, 0.0], button).clicked() {
                                        self.active_note_id = Some(id);
                                    }
                                },
                            );

                            // 2. Meta Area (Date + Delete)
                            ui.allocate_ui_with_layout(
                                egui::vec2(meta_width, ui.spacing().interact_size.y),
                                Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                        delete_id = Some(index);
                                    }
                                    ui.label(RichText::new(&date).size(10.0).color(Color32::GRAY));
                                },
                            );
                        });
                    }
                });

                if let Some(idx) = delete_id {
                    if let Some(note) = self.notes.get(idx) {
                        let deleted_id = note.id;
                        self.notes.remove(idx);
                        if self.active_note_id == Some(deleted_id) {
                            self.active_note_id = None;
                        }
                    }
                }
            });
        });
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        let note_index = match self.active_note_id {
            Some(id) => match self.notes.iter().position(|n| n.id == id) {
                Some(idx) => idx,
                None => return,
            },
            None => return,
        };

        let Some(note) = self.notes.get_mut(note_index) else {
            return;
        };

        ui.vertical(|ui| {
            ui.heading("Edit");
            ui.separator();

            // Title Edit
            ui.horizontal(|ui| {
                ui.label("Title:");
                if ui.text_edit_singleline(&mut note.title).changed() {
                    note.modified_at = Utc::now();
                }
            });
            ui.separator();

            // Calculate available height for editor
            let overhead = 20.0;
            let available_height = ui.available_height();
            let row_height = ui.text_style_height(&egui::TextStyle::Monospace);

            // Subtract a small buffer for scrollbar/padding if necessary, or just use raw.
            // CodeEditor might need a bit of extra space for its own padding.
            let rows = ((available_height - overhead).max(50.0) / row_height).floor() as usize;
            let rows = rows.max(5); // Minimum 5 rows

            // Content Edit with CodeEditor
            let theme = if ui.visuals().dark_mode {
                egui_code_editor::ColorTheme::GITHUB_DARK
            } else {
                egui_code_editor::ColorTheme::GITHUB_LIGHT
            };

            let mut content = note.content.clone();
            let response = egui_code_editor::CodeEditor::default()
                .id_source("markdown_editor")
                .auto_shrink(false)
                .with_fontsize(14.0)
                .with_theme(theme)
                .with_rows(rows) // Set rows to Fill
                .with_syntax(egui_code_editor::Syntax::default())
                .with_numlines(true)
                .show(ui, &mut content);

            if response.response.changed() {
                note.content = content;
                note.modified_at = Utc::now();
            }
        });
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        let note = match self.active_note_id {
            Some(id) => match self.notes.iter().find(|n| n.id == id) {
                Some(n) => n,
                None => return,
            },
            None => return,
        };

        ui.vertical(|ui| {
            ui.heading("Preview");
            ui.separator();
            egui::ScrollArea::vertical()
                .id_salt("preview_scroll")
                .show(ui, |ui| {
                    ui.set_max_width(ui.available_width());
                    egui_commonmark::CommonMarkViewer::new().show(
                        ui,
                        &mut self.commonmark_cache,
                        &note.content,
                    );
                });
        });
    }

    fn create_new_note(&mut self) {
        let new_note = Note {
            id: Uuid::new_v4(),
            title: "New Note".to_owned(),
            content: String::new(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        };
        self.active_note_id = Some(new_note.id);
        self.notes.insert(0, new_note); // Add to top
    }
}
