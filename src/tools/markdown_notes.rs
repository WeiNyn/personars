use super::Tool;
use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32, Layout, RichText};

use egui_commonmark::CommonMarkCache;
use egui_extras::{Size, StripBuilder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default, PartialEq, Deserialize, Serialize)]
enum NarrowTab {
    #[default]
    Edit,
    Preview,
}

/// Lightweight metadata kept in memory for sidebar listing & title search.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct NoteIndex {
    pub id: Uuid,
    pub title: String,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub modified_at: DateTime<Utc>,
}

/// Full note content stored in IDB, loaded on demand.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct NoteContent {
    pub id: Uuid,
    pub content: String,
}

/// Full note — used on native where all notes live in memory.
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

// ---------------------------------------------------------------------------
// Main state
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(default)]
pub struct MarkdownNotes {
    /// Full notes — used on native; skipped on wasm32 (IDB is source of truth).
    #[cfg_attr(target_arch = "wasm32", serde(skip))]
    notes: Vec<Note>,

    active_note_id: Option<Uuid>,

    /// Note index — lightweight metadata always in memory (wasm32).
    /// On native this is derived from `notes` on startup.
    #[serde(skip)]
    note_index: Vec<NoteIndex>,

    /// Content of the currently active note (wasm32: lazy-loaded from IDB).
    #[serde(skip)]
    active_content: Option<String>,

    /// True once we've synced `note_index` from `notes` on native.
    #[serde(skip)]
    native_index_synced: bool,

    #[serde(skip)]
    search_query: String,
    #[serde(skip)]
    commonmark_cache: CommonMarkCache,
    narrow_tab: NarrowTab,

    // -- wasm32-only: IndexedDB async state --
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    idb_state: Arc<Mutex<NotesIdbState>>,
}

// Manual Serialize implementation to skip non-serializable / skip fields
impl Serialize for MarkdownNotes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct as _;
        let mut state = serializer.serialize_struct("MarkdownNotes", 3)?;
        // On native we persist full notes; on wasm32 notes is empty (IDB does it)
        state.serialize_field("notes", &self.notes)?;
        state.serialize_field("active_note_id", &self.active_note_id)?;
        state.serialize_field("narrow_tab", &self.narrow_tab)?;
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
            note_index: Vec::new(),
            active_content: None,
            native_index_synced: false,
            search_query: String::new(),
            commonmark_cache: CommonMarkCache::default(),
            narrow_tab: NarrowTab::default(),
            #[cfg(target_arch = "wasm32")]
            idb_state: Arc::new(Mutex::new(NotesIdbState::default())),
        }
    }
}

// ---------------------------------------------------------------------------
// Tool trait
// ---------------------------------------------------------------------------

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

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        self.ensure_ready();

        ui.vertical(|ui| {
            // Note selector dropdown
            ui.horizontal(|ui| {
                let current_title = self
                    .active_note_id
                    .and_then(|id| self.note_index.iter().find(|n| n.id == id))
                    .map_or("Select a note...".to_owned(), |n| n.title.clone());

                let mut newly_selected = None;
                egui::ComboBox::from_id_salt("note_selector_narrow")
                    .selected_text(current_title)
                    .width(ui.available_width() - 40.0)
                    .show_ui(ui, |ui| {
                        for idx in &self.note_index {
                            let is_selected = self.active_note_id == Some(idx.id);
                            if ui.selectable_label(is_selected, &idx.title).clicked() {
                                newly_selected = Some(idx.id);
                            }
                        }
                    });
                if let Some(id) = newly_selected {
                    self.select_note(id);
                }

                if ui.button(RichText::new("➕").strong()).clicked() {
                    self.create_new_note();
                }
            });

            ui.separator();

            if self.active_note_id.is_some() && self.active_content.is_some() {
                // Tab bar
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.narrow_tab, NarrowTab::Edit, "✏ Edit");
                    ui.selectable_value(&mut self.narrow_tab, NarrowTab::Preview, "👁 Preview");
                });
                ui.separator();

                match self.narrow_tab {
                    NarrowTab::Edit => self.render_editor(ui),
                    NarrowTab::Preview => self.render_preview(ui),
                }
            } else if self.active_note_id.is_some() {
                ui.centered_and_justified(|ui| {
                    ui.spinner();
                });
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select a note to edit.");
                });
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Platform-agnostic helpers
// ---------------------------------------------------------------------------

impl MarkdownNotes {
    /// Ensure the index and active content are ready for rendering.
    fn ensure_ready(&mut self) {
        #[cfg(target_arch = "wasm32")]
        self.poll_idb();

        #[cfg(not(target_arch = "wasm32"))]
        self.sync_native_index();
    }

    /// (Native only) Build `note_index` from `notes` on first frame.
    #[cfg(not(target_arch = "wasm32"))]
    fn sync_native_index(&mut self) {
        if !self.native_index_synced {
            self.native_index_synced = true;
            self.note_index = self
                .notes
                .iter()
                .map(|n| NoteIndex {
                    id: n.id,
                    title: n.title.clone(),
                    created_at: n.created_at,
                    modified_at: n.modified_at,
                })
                .collect();
            // Load active content
            if let Some(id) = self.active_note_id {
                self.active_content = self
                    .notes
                    .iter()
                    .find(|n| n.id == id)
                    .map(|n| n.content.clone());
            }
        }
    }

    /// Select a note — on wasm32 this triggers async content load.
    fn select_note(&mut self, id: Uuid) {
        if self.active_note_id == Some(id) {
            return;
        }
        self.active_note_id = Some(id);

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.active_content = self
                .notes
                .iter()
                .find(|n| n.id == id)
                .map(|n| n.content.clone());
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.active_content = None; // will be loaded async
            self.load_active_content(id);
        }
    }

    /// Shared 3-pane layout used by both windowed and full-screen modes.
    pub fn render_layout(&mut self, ui: &mut egui::Ui) {
        self.ensure_ready();

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
                    if self.active_note_id.is_some() && self.active_content.is_some() {
                        StripBuilder::new(ui)
                            .size(Size::relative(0.5).at_least(100.0))
                            .size(Size::exact(1.0))
                            .size(Size::remainder())
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
                    } else if self.active_note_id.is_some() {
                        ui.centered_and_justified(|ui| {
                            ui.spinner();
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

            // Note List (uses note_index for display)
            ui.vertical(|ui| {
                let mut delete_id: Option<Uuid> = None;
                let available_width = ui.available_width();

                let meta_width = 85.0;
                let title_width = available_width - meta_width - 8.0;

                egui::ScrollArea::vertical().show(ui, |ui| {
                    let filtered: Vec<(Uuid, String, String)> = self
                        .note_index
                        .iter()
                        .filter(|idx| {
                            self.search_query.is_empty()
                                || idx
                                    .title
                                    .to_lowercase()
                                    .contains(&self.search_query.to_lowercase())
                        })
                        .map(|idx| {
                            (
                                idx.id,
                                idx.title.clone(),
                                idx.created_at.format("%Y-%m-%d").to_string(),
                            )
                        })
                        .collect();

                    for (id, title, date) in filtered {
                        let is_selected = self.active_note_id == Some(id);

                        ui.horizontal(|ui| {
                            // Title
                            ui.allocate_ui_with_layout(
                                egui::vec2(title_width, ui.spacing().interact_size.y),
                                Layout::left_to_right(egui::Align::Center),
                                |ui| {
                                    let button =
                                        egui::Button::new(&title).selected(is_selected).wrap();
                                    if ui.add_sized([title_width, 0.0], button).clicked() {
                                        self.select_note(id);
                                    }
                                },
                            );

                            // Meta (date + delete)
                            ui.allocate_ui_with_layout(
                                egui::vec2(meta_width, ui.spacing().interact_size.y),
                                Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                        delete_id = Some(id);
                                    }
                                    ui.label(RichText::new(&date).size(10.0).color(Color32::GRAY));
                                },
                            );
                        });
                    }
                });

                if let Some(id) = delete_id {
                    self.delete_note(id);
                }
            });
        });
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        let Some(active_id) = self.active_note_id else {
            return;
        };

        if self.active_content.is_none() {
            return;
        }

        ui.vertical(|ui| {
            ui.heading("Edit");
            ui.separator();

            // Title Edit — scoped borrow of note_index
            {
                if let Some(idx) = self.note_index.iter_mut().find(|i| i.id == active_id) {
                    let mut title_changed = false;
                    ui.horizontal(|ui| {
                        ui.label("Title:");
                        if ui.text_edit_singleline(&mut idx.title).changed() {
                            idx.modified_at = Utc::now();
                            title_changed = true;
                        }
                    });

                    if title_changed {
                        // Sync title to native notes
                        #[cfg(not(target_arch = "wasm32"))]
                        if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
                            note.title = idx.title.clone();
                            note.modified_at = idx.modified_at;
                        }
                        // Save to IDB
                        #[cfg(target_arch = "wasm32")]
                        self.save_index_to_idb();
                    }
                }
            }
            ui.separator();

            // Content Editor
            let overhead = 20.0;
            let available_height = ui.available_height();
            let row_height = ui.text_style_height(&egui::TextStyle::Monospace);
            let rows = ((available_height - overhead).max(50.0) / row_height).floor() as usize;
            let rows = rows.max(5);

            let theme = if ui.visuals().dark_mode {
                egui_code_editor::ColorTheme::GITHUB_DARK
            } else {
                egui_code_editor::ColorTheme::GITHUB_LIGHT
            };

            let content = self.active_content.clone().unwrap_or_default();
            let mut content_edit = content;
            let response = egui_code_editor::CodeEditor::default()
                .id_source("markdown_editor")
                .auto_shrink(false)
                .with_fontsize(14.0)
                .with_theme(theme)
                .with_rows(rows)
                .with_syntax(egui_code_editor::Syntax::default())
                .with_numlines(true)
                .show(ui, &mut content_edit);

            if response.response.changed() {
                self.active_content = Some(content_edit.clone());

                // Sync to native notes
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(note) = self.notes.iter_mut().find(|n| n.id == active_id) {
                    note.content = content_edit;
                    note.modified_at = Utc::now();
                }

                // Update modified_at in index
                if let Some(idx) = self.note_index.iter_mut().find(|i| i.id == active_id) {
                    idx.modified_at = Utc::now();
                }

                // Save content to IDB
                #[cfg(target_arch = "wasm32")]
                self.save_content_to_idb(active_id);
            }
        });
    }

    fn render_preview(&mut self, ui: &mut egui::Ui) {
        let content = match &self.active_content {
            Some(c) => c.clone(),
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
                        &content,
                    );
                });
        });
    }

    fn create_new_note(&mut self) {
        let now = Utc::now();
        let id = Uuid::new_v4();

        // Add to index
        self.note_index.insert(
            0,
            NoteIndex {
                id,
                title: "New Note".to_owned(),
                created_at: now,
                modified_at: now,
            },
        );

        // Set active
        self.active_note_id = Some(id);
        self.active_content = Some(String::new());

        // Native: also add full Note
        #[cfg(not(target_arch = "wasm32"))]
        self.notes.insert(
            0,
            Note {
                id,
                title: "New Note".to_owned(),
                content: String::new(),
                created_at: now,
                modified_at: now,
            },
        );

        // wasm32: persist to IDB
        #[cfg(target_arch = "wasm32")]
        {
            self.save_index_to_idb();
            self.save_content_to_idb(id);
        }
    }

    fn delete_note(&mut self, id: Uuid) {
        self.note_index.retain(|idx| idx.id != id);

        #[cfg(not(target_arch = "wasm32"))]
        self.notes.retain(|n| n.id != id);

        if self.active_note_id == Some(id) {
            self.active_note_id = None;
            self.active_content = None;
        }

        #[cfg(target_arch = "wasm32")]
        self.delete_note_from_idb(id);
    }
}

// ---------------------------------------------------------------------------
// wasm32: IndexedDB async bridge
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct NotesIdbState {
    /// Loaded note index (set once, consumed by `poll_idb`).
    loaded_index: Option<Vec<NoteIndex>>,
    /// Loaded content for a specific note (set by async, consumed by `poll_idb`).
    loaded_content: Option<(Uuid, String)>,
    /// Whether the initial index load has been kicked off.
    init_started: bool,
}

#[cfg(target_arch = "wasm32")]
impl MarkdownNotes {
    /// Kick off async load of note index from `IndexedDB`. Call once after
    /// construction.
    pub fn init_idb(&mut self) {
        let state = Arc::clone(&self.idb_state);

        {
            let mut s = state.lock().expect("lock poisoned");
            if s.init_started {
                return;
            }
            s.init_started = true;
        }

        wasm_bindgen_futures::spawn_local(async move {
            let db = match super::idb_storage::open_notes_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open notes IndexedDB: {e}");
                    return;
                }
            };

            let index: Vec<NoteIndex> = super::idb_storage::load_note_index(&db)
                .await
                .unwrap_or_default();

            if let Ok(mut s) = state.lock() {
                s.loaded_index = Some(index);
            }
        });
    }

    /// Poll for completed async operations and merge into `self`.
    fn poll_idb(&mut self) {
        let (loaded_index, loaded_content) = {
            let mut s = self.idb_state.lock().expect("lock poisoned");
            (s.loaded_index.take(), s.loaded_content.take())
        };

        // Apply loaded index
        if let Some(index) = loaded_index {
            if !index.is_empty() {
                self.note_index = index;
            }
            // If there's an active note, load its content
            if let Some(id) = self.active_note_id {
                if self.active_content.is_none() {
                    self.load_active_content(id);
                }
            }
        }

        // Apply loaded content
        if let Some((id, content)) = loaded_content {
            if self.active_note_id == Some(id) {
                self.active_content = Some(content);
            }
        }
    }

    /// Async load content for a specific note.
    fn load_active_content(&self, id: Uuid) {
        let state = Arc::clone(&self.idb_state);
        let id_str = id.to_string();

        wasm_bindgen_futures::spawn_local(async move {
            let db = match super::idb_storage::open_notes_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open notes IndexedDB: {e}");
                    return;
                }
            };

            let content: Option<NoteContent> =
                super::idb_storage::load_single_note_content(&db, &id_str)
                    .await
                    .unwrap_or(None);

            if let Ok(mut s) = state.lock() {
                s.loaded_content = Some((id, content.map_or_else(String::new, |c| c.content)));
            }
        });
    }

    /// Save the full note index to IDB.
    fn save_index_to_idb(&self) {
        let index = self.note_index.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let db = match super::idb_storage::open_notes_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open notes IndexedDB for save: {e}");
                    return;
                }
            };

            if let Err(e) = super::idb_storage::save_note_index(&db, &index).await {
                log::error!("Failed to save note index: {e}");
            }
        });
    }

    /// Save the active note's content to IDB.
    fn save_content_to_idb(&self, id: Uuid) {
        let content = NoteContent {
            id,
            content: self.active_content.clone().unwrap_or_default(),
        };

        wasm_bindgen_futures::spawn_local(async move {
            let db = match super::idb_storage::open_notes_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open notes IndexedDB for save: {e}");
                    return;
                }
            };

            if let Err(e) = super::idb_storage::save_single_note_content(&db, &content).await {
                log::error!("Failed to save note content: {e}");
            }
        });
    }

    /// Delete a note from IDB (both index and content).
    fn delete_note_from_idb(&self, id: Uuid) {
        let id_str = id.to_string();
        let index = self.note_index.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let db = match super::idb_storage::open_notes_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open notes IndexedDB for delete: {e}");
                    return;
                }
            };

            if let Err(e) = super::idb_storage::delete_note(&db, &id_str).await {
                log::error!("Failed to delete note: {e}");
            }
            // Also re-save the index to stay consistent
            if let Err(e) = super::idb_storage::save_note_index(&db, &index).await {
                log::error!("Failed to save note index after delete: {e}");
            }
        });
    }
}
