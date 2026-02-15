use super::Tool;
use eframe::egui::{self, Color32, Id, RichText};

#[derive(Clone, serde::Deserialize, serde::Serialize)]
pub struct TodoItem {
    pub text: String,
    pub completed: bool,
    pub priority: TodoPriority,
}

#[derive(PartialEq, Eq, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum TodoPriority {
    High,
    Normal,
    Low,
    None,
}

impl TodoPriority {
    fn icon(&self) -> egui::RichText {
        match self {
            Self::High => {
                egui::RichText::new(egui_phosphor::regular::CARET_DOUBLE_UP).color(Color32::RED)
            }
            Self::Normal => {
                egui::RichText::new(egui_phosphor::regular::CARET_UP).color(Color32::YELLOW)
            }
            Self::Low => egui::RichText::new(egui_phosphor::regular::EQUALS).color(Color32::GRAY),
            Self::None => egui::RichText::new(egui_phosphor::regular::MINUS).color(Color32::GRAY),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct TodoList {
    items: Vec<TodoItem>,
    #[serde(skip)]
    new_task_text: String,
    #[serde(skip)]
    new_task_priority: TodoPriority,
    filter: TodoFilter,
}

#[derive(PartialEq, Eq, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            items: vec![
                TodoItem {
                    text: "Welcome to Personars Todo!".to_owned(),
                    completed: false,
                    priority: TodoPriority::Normal,
                },
                TodoItem {
                    text: "Try adding a new task".to_owned(),
                    completed: false,
                    priority: TodoPriority::High,
                },
                TodoItem {
                    text: "Mark this task as done".to_owned(),
                    completed: true,
                    priority: TodoPriority::Low,
                },
            ],
            new_task_text: String::new(),
            new_task_priority: TodoPriority::Normal,
            filter: TodoFilter::All,
        }
    }
}

impl Tool for TodoList {
    fn name(&self) -> &'static str {
        "Todo List"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::LIST_CHECKS
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(450.0)
            .default_height(500.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                self.render_content(ui);
            });
    }

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        self.render_content(ui);
    }
}

impl TodoList {
    fn render_content(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        ui.vertical(|ui| {
            ui.heading("Todo List");
            ui.separator();

            self.render_input_area(ui, &ctx);
            ui.add_space(10.0);
            self.render_filters(ui);
            ui.separator();
            self.render_list(ui);
        });
    }
}

impl TodoList {
    fn render_input_area(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.horizontal(|ui| {
            // Priority Selector
            egui::ComboBox::from_id_salt("priority_selector")
                .selected_text(self.new_task_priority.icon())
                .width(50.0)
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.new_task_priority,
                        TodoPriority::High,
                        TodoPriority::High.icon(),
                    );
                    ui.selectable_value(
                        &mut self.new_task_priority,
                        TodoPriority::Normal,
                        TodoPriority::Normal.icon(),
                    );
                    ui.selectable_value(
                        &mut self.new_task_priority,
                        TodoPriority::Low,
                        TodoPriority::Low.icon(),
                    );
                    ui.selectable_value(
                        &mut self.new_task_priority,
                        TodoPriority::None,
                        TodoPriority::None.icon(),
                    );
                });

            let response = ui.add(
                egui::TextEdit::singleline(&mut self.new_task_text)
                    .hint_text("What needs to be done?")
                    .desired_width(220.0),
            );

            let add_clicked = ui.button("Add").clicked();

            if (add_clicked
                || (response.lost_focus() && ctx.input(|i| i.key_pressed(egui::Key::Enter))))
                && !self.new_task_text.trim().is_empty()
            {
                self.items.push(TodoItem {
                    text: self.new_task_text.trim().to_owned(),
                    completed: false,
                    priority: self.new_task_priority,
                });
                self.new_task_text.clear();
                response.request_focus();
            }
        });
    }

    fn render_filters(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.filter, TodoFilter::All, "All");
            ui.selectable_value(&mut self.filter, TodoFilter::Active, "Active");
            ui.selectable_value(&mut self.filter, TodoFilter::Completed, "Completed");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Clear Completed").clicked() {
                    self.items.retain(|i| !i.completed);
                }
            });
        });
    }

    fn render_list(&mut self, ui: &mut egui::Ui) {
        let mut from = None;
        let mut to = None;
        let mut indices_to_delete = Vec::new();

        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.filter == TodoFilter::All {
                self.render_list_dnd(ui, &mut from, &mut to, &mut indices_to_delete);
            } else {
                self.render_list_filtered(ui, &mut indices_to_delete);
            }
        });

        // Handle Reordering
        if let (Some(from), Some(mut to)) = (from, to) {
            let len = self.items.len();
            if from < len && to <= len {
                if from < to {
                    to -= 1;
                }

                if from != to {
                    let item = self.items.remove(from);
                    self.items.insert(to, item);
                }
            }
        }

        // Apply Deletions
        indices_to_delete.sort_unstable_by(|a, b| b.cmp(a));
        for idx in indices_to_delete {
            if idx < self.items.len() {
                self.items.remove(idx);
            }
        }
    }

    fn render_list_dnd(
        &mut self,
        ui: &mut egui::Ui,
        from: &mut Option<usize>,
        to: &mut Option<usize>,
        indices_to_delete: &mut Vec<usize>,
    ) {
        let _item_count = self.items.len();
        let frame = egui::Frame::default().inner_margin(4.0);

        let (_, _dropped_payload) = ui.dnd_drop_zone::<usize, ()>(frame, |ui| {
            for (idx, item) in self.items.iter_mut().enumerate() {
                let item_id = Id::new(("todo_item", idx));

                // Wrap row in a horizontal layout, which acts as the visible Drop Target
                let row_response = ui
                    .horizontal(|ui| {
                        // Handle is the Drag Source
                        ui.dnd_drag_source(item_id, idx, |ui| {
                            ui.label("::").on_hover_text("Drag to reorder");
                        });

                        // Priority Selector (Editable)
                        egui::ComboBox::from_id_salt(("priority", idx))
                            .selected_text(item.priority.icon())
                            .width(20.0)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut item.priority,
                                    TodoPriority::High,
                                    TodoPriority::High.icon(),
                                );
                                ui.selectable_value(
                                    &mut item.priority,
                                    TodoPriority::Normal,
                                    TodoPriority::Normal.icon(),
                                );
                                ui.selectable_value(
                                    &mut item.priority,
                                    TodoPriority::Low,
                                    TodoPriority::Low.icon(),
                                );
                                ui.selectable_value(
                                    &mut item.priority,
                                    TodoPriority::None,
                                    TodoPriority::None.icon(),
                                );
                            });
                        ui.checkbox(&mut item.completed, "");

                        let text = if item.completed {
                            RichText::new(&item.text)
                                .strikethrough()
                                .color(Color32::GRAY)
                        } else {
                            RichText::new(&item.text)
                        };

                        ui.label(text);

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("🗑").on_hover_text("Delete").clicked() {
                                indices_to_delete.push(idx);
                            }
                        });
                    })
                    .response;

                // Detect drops onto this ROW (checking if mouse is over row_response)
                if let (Some(pointer), Some(hovered_payload)) = (
                    ui.input(|i| i.pointer.interact_pos()),
                    row_response.dnd_hover_payload::<usize>(),
                ) {
                    let rect = row_response.rect;

                    // Visual indicator
                    let stroke = egui::Stroke::new(2.0, Color32::from_rgb(100, 150, 255));

                    // Determine insertion index based on pointer position relative to item center
                    let insert_row_idx = if *hovered_payload == idx {
                        ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                        idx
                    } else if pointer.y < rect.center().y {
                        ui.painter().hline(rect.x_range(), rect.top(), stroke);
                        idx
                    } else {
                        ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                        idx + 1
                    };

                    if let Some(dragged_idx) = row_response.dnd_release_payload() {
                        *from = Some(*dragged_idx);
                        *to = Some(insert_row_idx);
                    }
                }
            }
        });
    }

    fn render_list_filtered(&mut self, ui: &mut egui::Ui, indices_to_delete: &mut Vec<usize>) {
        for (idx, item) in self.items.iter_mut().enumerate() {
            let show = match self.filter {
                TodoFilter::All => true,
                TodoFilter::Active => !item.completed,
                TodoFilter::Completed => item.completed,
            };

            if !show {
                continue;
            }

            ui.horizontal(|ui| {
                // Priority Selector (Editable)
                egui::ComboBox::from_id_salt(("priority_filtered", idx))
                    .selected_text(item.priority.icon())
                    .width(20.0)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut item.priority,
                            TodoPriority::High,
                            TodoPriority::High.icon(),
                        );
                        ui.selectable_value(
                            &mut item.priority,
                            TodoPriority::Normal,
                            TodoPriority::Normal.icon(),
                        );
                        ui.selectable_value(
                            &mut item.priority,
                            TodoPriority::Low,
                            TodoPriority::Low.icon(),
                        );
                        ui.selectable_value(
                            &mut item.priority,
                            TodoPriority::None,
                            TodoPriority::None.icon(),
                        );
                    });
                ui.checkbox(&mut item.completed, "");

                let text = if item.completed {
                    RichText::new(&item.text)
                        .strikethrough()
                        .color(Color32::GRAY)
                } else {
                    RichText::new(&item.text)
                };

                ui.label(text);

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("🗑").on_hover_text("Delete").clicked() {
                        indices_to_delete.push(idx);
                    }
                });
            });
        }
    }
}
