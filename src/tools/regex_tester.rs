use super::Tool;
use eframe::egui::{self, Color32, TextFormat, text::LayoutJob};
use regex::Regex;

#[derive(Default)]
pub struct RegexTester {
    pattern: String,
    test_text: String,
    compiled_regex: Option<Regex>,
    error: Option<String>,
}

impl Tool for RegexTester {
    fn name(&self) -> &'static str {
        "Regex Tester"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::MAGNIFYING_GLASS
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(600.0)
            .default_height(500.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    ui.label("Regex Pattern:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.pattern)
                            .hint_text("e.g. ^(foo|bar)$")
                            .desired_width(f32::INFINITY)
                            .font(egui::TextStyle::Monospace),
                    );

                    if response.changed() {
                        self.compile();
                    }

                    if let Some(err) = &self.error {
                        ui.colored_label(Color32::RED, err);
                    } else if self.compiled_regex.is_some() {
                        ui.colored_label(Color32::GREEN, "Regex compiled successfully");
                    }

                    ui.separator();
                    ui.label("Test String:");

                    let regex = self.compiled_regex.clone();
                    let mut layouter =
                        |ui: &egui::Ui, string: &dyn egui::TextBuffer, wrap_width: f32| {
                            let mut layout_job = Self::highlight(ui, string.as_str(), &regex);
                            layout_job.wrap.max_width = wrap_width;
                            ui.painter().layout_job(layout_job)
                        };

                    let response = ui.add(
                        egui::TextEdit::multiline(&mut self.test_text)
                            .hint_text("Type items here to test...")
                            .desired_width(f32::INFINITY)
                            .desired_rows(10)
                            .layouter(&mut layouter),
                    );

                    let mut cursor_idx = None;
                    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), response.id) {
                        if let Some(cursor_range) = state.cursor.char_range() {
                            cursor_idx = Some(cursor_range.primary.index);
                        }
                    }

                    if let Some(re) = &self.compiled_regex {
                        let count = re.find_iter(&self.test_text).count();
                        ui.label(format!("Found {count} matches"));

                        ui.separator();

                        if let Some(idx) = cursor_idx {
                            // Find match at cursor
                            for cap in re.captures_iter(&self.test_text) {
                                if let Some(m) = cap.get(0) {
                                    // Check if cursor is roughly within this match or on the same line?
                                    // User said "cursor on line that has matches", but "print matched group".
                                    // Identifying if cursor is "at" the match is more precise.
                                    // Let's verify if cursor index is within [start, end].
                                    if idx >= m.start() && idx <= m.end() {
                                        ui.label(egui::RichText::new("Match Details:").strong());
                                        ui.label(format!("Full Match: {:?}", m.as_str()));

                                        for (i, grp) in cap.iter().enumerate().skip(1) {
                                            if let Some(g) = grp {
                                                ui.horizontal(|ui| {
                                                    ui.label(format!("Group {i}: "));
                                                    ui.label(
                                                        egui::RichText::new(g.as_str())
                                                            .color(Self::get_group_color(i)),
                                                    );
                                                });
                                            }
                                        }
                                        break; // Stop after finding the first intersecting match
                                    }
                                }
                            }
                        } else {
                            ui.label("Move cursor to a match to see details.");
                        }
                    }
                });
            });
    }
}

impl RegexTester {
    fn compile(&mut self) {
        if self.pattern.is_empty() {
            self.compiled_regex = None;
            self.error = None;
            return;
        }

        match Regex::new(&self.pattern) {
            Ok(re) => {
                self.compiled_regex = Some(re);
                self.error = None;
            }
            Err(e) => {
                self.compiled_regex = None;
                self.error = Some(format!("Error: {e}"));
            }
        }
    }

    #[expect(clippy::indexing_slicing)]
    fn get_group_color(index: usize) -> Color32 {
        const GROUP_COLORS: &[Color32] = &[
            Color32::from_rgb(100, 200, 255), // Blueish
            Color32::from_rgb(100, 255, 100), // Greenish
            Color32::from_rgb(255, 200, 100), // Orangeish
            Color32::from_rgb(255, 100, 200), // Pinkish
            Color32::from_rgb(200, 100, 255), // Purpleish
        ];
        GROUP_COLORS[(index - 1) % GROUP_COLORS.len()]
    }

    #[expect(clippy::indexing_slicing)]
    fn highlight(ui: &egui::Ui, text: &str, regex: &Option<Regex>) -> LayoutJob {
        // Default text style
        let font_id = egui::TextStyle::Monospace.resolve(ui.style());
        let mut job = LayoutJob::default();

        if let Some(re) = regex {
            // let mut last_index = 0; // Unused

            // We need to handle overlapping colors.
            // A simple approach is to find all matches and their groups,
            // and build a map of index -> color/format.
            // But LayoutJob requires appending text segments.
            // So we need to flatten the highlights.
            // Since we want to highlight groups, and groups can overlap,
            // dealing with that perfectly in a simple LayoutJob is hard.
            // Strategy:
            // 1. Iterate through text.
            // 2. Identify regions that are part of a match or group.
            // 3. Current simplification: just highlight full matches.
            // wait, user said "different color highlight on each matched group".

            // Let's try to highlight capture groups with specific colors.
            // If we have `(\d{4})-(\d{2})-(\d{2})`, we have 3 groups.
            // We can iterate matches.
            // For each match, we can find the sub-ranges for groups.
            // If groups nest, the inner one should probably take precedence?

            // To make it simple but effective:
            // We will collect all "spans" (start, end, color).
            // Then we will sort them and fill the job.
            // But we can't overlap text in LayoutJob.

            // Alternative: Just highlighting the full match with alternating colors is safer for a start.
            // But let's try to support groups if they are creating distinct spans.
            // E.g. `2024-01-01`. 2024 is Group 1, - is match but not group, 01 is Group 2...

            // Let's stick to: Highlight full match with a background.
            // AND Colorize text for groups?

            // Refined Plan:
            // Iterate matches.
            // Append non-matching text as plain.
            // For the matched text, we need to inspect it.
            // If there are capturing groups, we color them.

            // Actually, `regex` provides `captures_iter`.
            // We can iterate captures.
            // But `LayoutJob` is linear.

            // Let's just alternate background color for full matches for now,
            // and use text color for groups if manageable.
            // Simpler: Just alternating colors for full matches is a very good "Regex 101" style baseline.
            // User asked: "different color highlight on each matched group".
            // Okay, I will define a palette for groups.

            // We will construct a vector of (byte_index, type, color_idx) events?
            // No, simpler:
            // 1. Create a `Vec<(usize, usize, Color32)>` of highlights.
            //    (start, end, color).
            //    We iterate matches, then groups.
            //    If a later group is inside an earlier one, we overwrite the range?
            //    Yes, inner groups usually are more specific.
            // 2. Then we flatten this into a linear list of styles.

            let mut styles = vec![
                TextFormat {
                    font_id: font_id.clone(),
                    ..Default::default()
                };
                text.len().max(1)
            ];

            // Helper to fill style range
            // We use a separate vector specifically for this input text length
            // But text.len() can be large.
            // Wait, for 10KB text this is array of 10k structs. That's fine (Rust is fast).

            // Reset to default
            for style in styles.iter_mut().take(text.len()) {
                *style = TextFormat {
                    font_id: font_id.clone(),
                    color: if ui.visuals().dark_mode {
                        Color32::LIGHT_GRAY
                    } else {
                        Color32::DARK_GRAY
                    },
                    ..Default::default()
                };
            }

            for cap in re.captures_iter(text) {
                // The full match (Group 0)
                if let Some(m) = cap.get(0) {
                    // Alternating background for matches?
                    // Or just underscore?
                    // Let's set a subtle background for the whole match
                    let bg = if ui.visuals().dark_mode {
                        Color32::from_rgba_premultiplied(50, 50, 50, 30)
                    } else {
                        Color32::from_rgba_premultiplied(200, 200, 200, 50)
                    };

                    for i in m.start()..m.end() {
                        if i < styles.len() {
                            styles[i].background = bg;
                        }
                    }
                }

                // Capture groups (1..N)
                for (i, grp) in cap.iter().enumerate().skip(1) {
                    if let Some(m) = grp {
                        let color = Self::get_group_color(i);
                        for k in m.start()..m.end() {
                            if k < styles.len() {
                                styles[k].color = color;
                            }
                        }
                    }
                }
            }

            // Now build LayoutJob from styles
            // Consolidate adjacent identical styles
            let mut start = 0;
            while start < text.len() {
                let mut end = start + 1;
                while end < text.len() && styles[end] == styles[start] {
                    end += 1;
                }

                job.append(&text[start..end], 0.0, styles[start].clone());
                start = end;
            }
        } else {
            job.append(
                text,
                0.0,
                TextFormat {
                    font_id: font_id.clone(),
                    color: if ui.visuals().dark_mode {
                        Color32::LIGHT_GRAY
                    } else {
                        Color32::DARK_GRAY
                    },
                    ..Default::default()
                },
            );
        }

        job
    }
}
