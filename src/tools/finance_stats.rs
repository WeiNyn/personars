// ---------------------------------------------------------------------------
// Finance Tracker – Statistics & Visualization
// ---------------------------------------------------------------------------
//
// Renders three sections inside a scrollable area:
//   1. Monthly summary (pay / earn by account) with a grouped bar chart.
//   2. Category distribution pie charts (one for pay, one for receive).
//   3. Month-over-month / week-over-week trend bar chart.

use super::finance_tracker::{Account, Transaction, TransactionType};
use chrono::{Datelike as _, NaiveDate};
use eframe::egui::{self, Color32, Pos2, RichText, Stroke, Vec2};
use egui_plot::{Bar, BarChart, Plot};
use std::collections::BTreeMap;
use std::f64::consts::TAU;

// ── Palette ─────────────────────────────────────────────────────────────────

const GREEN: Color32 = Color32::from_rgb(46, 160, 67);
const RED: Color32 = Color32::from_rgb(218, 54, 51);

/// Distinct colours for slices / categories.
const SLICE_COLORS: &[Color32] = &[
    Color32::from_rgb(99, 110, 250),  // indigo
    Color32::from_rgb(239, 85, 59),   // coral
    Color32::from_rgb(0, 204, 150),   // teal
    Color32::from_rgb(171, 99, 250),  // purple
    Color32::from_rgb(255, 161, 90),  // orange
    Color32::from_rgb(25, 211, 243),  // cyan
    Color32::from_rgb(255, 102, 146), // pink
    Color32::from_rgb(182, 232, 128), // lime
    Color32::from_rgb(255, 199, 95),  // gold
    Color32::from_rgb(102, 197, 204), // seafoam
    Color32::from_rgb(220, 176, 242), // lavender
    Color32::from_rgb(179, 179, 179), // grey
];

// ── Time-mode toggle ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Default, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum StatsTimeMode {
    #[default]
    Month,
    Week,
}

// ── Public entry point ──────────────────────────────────────────────────────

/// Render the full statistics tab content.
///
/// `stats_year` / `stats_month` are mutated by the month selector.
pub fn render_statistics_tab(
    ui: &mut egui::Ui,
    accounts: &[Account],
    transactions: &[Transaction],
    stats_year: &mut i32,
    stats_month: &mut u32,
    time_mode: &mut StatsTimeMode,
) {
    // ── Month selector ─────────────────────────────────────────────────
    render_month_selector(ui, stats_year, stats_month);
    ui.separator();

    egui::ScrollArea::vertical()
        .id_salt("stats_scroll")
        .show(ui, |ui| {
            // ── 1) Monthly Pay / Earn by Account ───────────────────────
            render_monthly_summary(ui, accounts, transactions, *stats_year, *stats_month);

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            // ── 2) Category distribution (pie charts) ──────────────────
            render_category_distribution(ui, transactions, *stats_year, *stats_month);

            ui.add_space(12.0);
            ui.separator();
            ui.add_space(4.0);

            // ── 3) Month / Week trend ──────────────────────────────────
            render_trend_chart(ui, transactions, time_mode);
        });
}

// ── Month selector ──────────────────────────────────────────────────────────

fn render_month_selector(ui: &mut egui::Ui, year: &mut i32, month: &mut u32) {
    ui.horizontal(|ui| {
        if ui.button("◀").clicked() {
            if *month == 1 {
                *month = 12;
                *year -= 1;
            } else {
                *month -= 1;
            }
        }

        let month_name = month_label(*month);
        ui.label(
            RichText::new(format!("{month_name} {year}"))
                .strong()
                .size(16.0),
        );

        if ui.button("▶").clicked() {
            if *month == 12 {
                *month = 1;
                *year += 1;
            } else {
                *month += 1;
            }
        }
    });
}

// ── 1) Monthly summary ─────────────────────────────────────────────────────

fn render_monthly_summary(
    ui: &mut egui::Ui,
    accounts: &[Account],
    transactions: &[Transaction],
    year: i32,
    month: u32,
) {
    ui.label(RichText::new("💰 Monthly Summary").strong().size(14.0));
    ui.add_space(4.0);

    let (first, last) = month_range(year, month);
    let in_month: Vec<&Transaction> = transactions
        .iter()
        .filter(|t| {
            let d = t.created_at.date_naive();
            d >= first && d <= last
        })
        .collect();

    if in_month.is_empty() {
        ui.label(
            RichText::new("No transactions in this month.").color(ui.visuals().weak_text_color()),
        );
        return;
    }

    // Aggregate per account: (earned, paid)
    let mut per_account: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for t in &in_month {
        let label = accounts
            .iter()
            .find(|a| a.id == t.account_id)
            .map_or("Unknown".to_owned(), |a| format!("{} {}", a.icon, a.name));
        let entry = per_account.entry(label).or_insert((0.0, 0.0));
        match t.transaction_type {
            TransactionType::Receive => entry.0 += t.amount,
            TransactionType::Pay => entry.1 += t.amount,
        }
    }

    // Summary table
    egui::Grid::new("monthly_summary_grid")
        .striped(true)
        .min_col_width(60.0)
        .show(ui, |ui| {
            ui.label(RichText::new("Account").strong());
            ui.label(RichText::new("Earned").strong().color(GREEN));
            ui.label(RichText::new("Paid").strong().color(RED));
            ui.label(RichText::new("Net").strong());
            ui.end_row();

            let mut total_earned = 0.0_f64;
            let mut total_paid = 0.0_f64;
            for (name, (earned, paid)) in &per_account {
                let net = earned - paid;
                let net_color = if net >= 0.0 { GREEN } else { RED };
                ui.label(name);
                ui.label(RichText::new(format!("${earned:.2}")).color(GREEN));
                ui.label(RichText::new(format!("${paid:.2}")).color(RED));
                ui.label(RichText::new(format_signed(net)).color(net_color));
                ui.end_row();
                total_earned += earned;
                total_paid += paid;
            }

            // Totals row
            let total_net = total_earned - total_paid;
            let net_color = if total_net >= 0.0 { GREEN } else { RED };
            ui.label(RichText::new("Total").strong());
            ui.label(
                RichText::new(format!("${total_earned:.2}"))
                    .strong()
                    .color(GREEN),
            );
            ui.label(
                RichText::new(format!("${total_paid:.2}"))
                    .strong()
                    .color(RED),
            );
            ui.label(
                RichText::new(format_signed(total_net))
                    .strong()
                    .color(net_color),
            );
            ui.end_row();
        });

    ui.add_space(8.0);

    // Bar chart: grouped by account
    render_monthly_bar_chart(ui, &per_account);
}

fn render_monthly_bar_chart(ui: &mut egui::Ui, per_account: &BTreeMap<String, (f64, f64)>) {
    let account_names: Vec<&String> = per_account.keys().collect();
    let mut earn_bars = Vec::new();
    let mut pay_bars = Vec::new();

    for (i, name) in account_names.iter().enumerate() {
        let Some(agg) = per_account.get(*name) else {
            continue;
        };
        let x = i as f64;
        earn_bars.push(
            Bar::new(x - 0.2, agg.0)
                .width(0.35)
                .name(format!("{name} Earn")),
        );
        pay_bars.push(
            Bar::new(x + 0.2, agg.1)
                .width(0.35)
                .name(format!("{name} Pay")),
        );
    }

    let earn_chart = BarChart::new("Earned", earn_bars).color(GREEN);
    let pay_chart = BarChart::new("Paid", pay_bars).color(RED);

    let plot_height = 180.0;
    Plot::new("monthly_account_bar")
        .height(plot_height)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show_axes([false, true])
        .y_axis_label("$")
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(earn_chart);
            plot_ui.bar_chart(pay_chart);
        });
}

// ── 2) Category distribution – pie charts ──────────────────────────────────

fn render_category_distribution(
    ui: &mut egui::Ui,
    transactions: &[Transaction],
    year: i32,
    month: u32,
) {
    ui.label(
        RichText::new("📊 Category Distribution")
            .strong()
            .size(14.0),
    );
    ui.add_space(4.0);

    let (first, last) = month_range(year, month);
    let in_month: Vec<&Transaction> = transactions
        .iter()
        .filter(|t| {
            let d = t.created_at.date_naive();
            d >= first && d <= last
        })
        .collect();

    // Aggregate by category for Pay and Receive separately
    let mut pay_by_cat: BTreeMap<&'static str, f64> = BTreeMap::new();
    let mut receive_by_cat: BTreeMap<&'static str, f64> = BTreeMap::new();

    for t in &in_month {
        let label = t.category.label();
        match t.transaction_type {
            TransactionType::Pay => *pay_by_cat.entry(label).or_default() += t.amount,
            TransactionType::Receive => *receive_by_cat.entry(label).or_default() += t.amount,
        }
    }

    // Render side-by-side (or stacked on narrow)
    let available = ui.available_width();
    let pie_size = (available / 2.2).clamp(100.0, 200.0);

    ui.horizontal_wrapped(|ui| {
        if !pay_by_cat.is_empty() {
            ui.vertical(|ui| {
                ui.label(RichText::new("Expenses").strong().color(RED));
                draw_pie_chart(ui, &pay_by_cat, pie_size);
            });
        }

        if !receive_by_cat.is_empty() {
            ui.vertical(|ui| {
                ui.label(RichText::new("Income").strong().color(GREEN));
                draw_pie_chart(ui, &receive_by_cat, pie_size);
            });
        }

        if pay_by_cat.is_empty() && receive_by_cat.is_empty() {
            ui.label(
                RichText::new("No transactions in this month.")
                    .color(ui.visuals().weak_text_color()),
            );
        }
    });
}

/// Draw a pie chart using egui painter primitives.
fn draw_pie_chart(ui: &mut egui::Ui, data: &BTreeMap<&'static str, f64>, size: f32) {
    let total: f64 = data.values().sum();
    if total <= 0.0 {
        return;
    }

    let radius = size / 2.0;
    let (response, painter) = ui.allocate_painter(Vec2::splat(size + 20.0), egui::Sense::hover());
    let center = response.rect.center();

    let mut start_angle: f32 = -std::f32::consts::FRAC_PI_2; // 12 o'clock

    for (i, (_label, &value)) in data.iter().enumerate() {
        let fraction = (value / total) as f32;
        let sweep = fraction * TAU as f32;
        let Some(&color) = SLICE_COLORS.get(i % SLICE_COLORS.len()) else {
            continue;
        };

        // Draw a filled arc via a polygon approximation
        let segments = (sweep / 0.05).max(4.0) as usize;
        let mut points = vec![center];
        for s in 0..=segments {
            let angle = start_angle + sweep * (s as f32 / segments as f32);
            points.push(Pos2::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }
        painter.add(egui::Shape::convex_polygon(
            points,
            color,
            Stroke::new(1.5, Color32::from_gray(40)),
        ));

        // Label position (midpoint of arc, pushed out slightly)
        let mid_angle = start_angle + sweep / 2.0;
        let label_r = radius * 0.65;
        let label_pos = Pos2::new(
            center.x + label_r * mid_angle.cos(),
            center.y + label_r * mid_angle.sin(),
        );

        // Only label slices large enough
        if fraction > 0.05 {
            let pct = format!("{:.0}%", fraction * 100.0);
            painter.text(
                label_pos,
                egui::Align2::CENTER_CENTER,
                &pct,
                egui::FontId::proportional(11.0),
                Color32::WHITE,
            );
        }

        start_angle += sweep;
    }

    // Legend below chart
    ui.add_space(4.0);
    for (i, (label, &value)) in data.iter().enumerate() {
        let Some(&color) = SLICE_COLORS.get(i % SLICE_COLORS.len()) else {
            continue;
        };
        let pct = value / total * 100.0;
        ui.horizontal(|ui| {
            let (rect, _) = ui.allocate_exact_size(Vec2::splat(10.0), egui::Sense::hover());
            ui.painter().rect_filled(rect, 2.0, color);
            ui.label(format!("{label} – ${value:.2} ({pct:.1}%)"));
        });
    }
}

// ── 3) Trend chart ─────────────────────────────────────────────────────────

fn render_trend_chart(
    ui: &mut egui::Ui,
    transactions: &[Transaction],
    time_mode: &mut StatsTimeMode,
) {
    ui.label(RichText::new("📈 Trend").strong().size(14.0));
    ui.add_space(4.0);

    ui.horizontal(|ui| {
        ui.label("View by:");
        ui.selectable_value(time_mode, StatsTimeMode::Month, "Month");
        ui.selectable_value(time_mode, StatsTimeMode::Week, "Week");
    });
    ui.add_space(4.0);

    if transactions.is_empty() {
        ui.label(
            RichText::new("No transaction data to show trends.")
                .color(ui.visuals().weak_text_color()),
        );
        return;
    }

    // Bucket key → (earned, paid)
    let mut buckets: BTreeMap<String, (f64, f64)> = BTreeMap::new();
    for t in transactions {
        let d = t.created_at.date_naive();
        let key = match time_mode {
            StatsTimeMode::Month => format!("{:04}-{:02}", d.year(), d.month()),
            StatsTimeMode::Week => {
                let iso = d.iso_week();
                format!("{:04}-W{:02}", iso.year(), iso.week())
            }
        };
        let entry = buckets.entry(key).or_insert((0.0, 0.0));
        match t.transaction_type {
            TransactionType::Receive => entry.0 += t.amount,
            TransactionType::Pay => entry.1 += t.amount,
        }
    }

    let keys: Vec<String> = buckets.keys().cloned().collect();
    let mut earn_bars = Vec::new();
    let mut pay_bars = Vec::new();

    for (i, key) in keys.iter().enumerate() {
        let Some(&(earned, paid)) = buckets.get(key) else {
            continue;
        };
        let x = i as f64;
        earn_bars.push(
            Bar::new(x - 0.2, earned)
                .width(0.35)
                .name(format!("{key} Earn")),
        );
        pay_bars.push(
            Bar::new(x + 0.2, paid)
                .width(0.35)
                .name(format!("{key} Pay")),
        );
    }

    let earn_chart = BarChart::new("Earned", earn_bars).color(GREEN);
    let pay_chart = BarChart::new("Paid", pay_bars).color(RED);

    Plot::new("trend_bar_chart")
        .height(200.0)
        .allow_drag(false)
        .allow_zoom(false)
        .allow_scroll(false)
        .allow_boxed_zoom(false)
        .show_axes([false, true])
        .y_axis_label("$")
        .legend(egui_plot::Legend::default())
        .show(ui, |plot_ui| {
            plot_ui.bar_chart(earn_chart);
            plot_ui.bar_chart(pay_chart);
        });

    // Text legend for x-axis labels
    ui.horizontal_wrapped(|ui| {
        for (i, key) in keys.iter().enumerate() {
            let label = pretty_bucket_label(key, *time_mode);
            ui.label(format!("{i}: {label}"));
        }
    });
}

// ── Utility ─────────────────────────────────────────────────────────────────

fn month_range(year: i32, month: u32) -> (NaiveDate, NaiveDate) {
    let first = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or_default();
    let last = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .unwrap_or_default()
    .pred_opt()
    .unwrap_or(first);
    (first, last)
}

fn month_label(month: u32) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

fn format_signed(v: f64) -> String {
    if v >= 0.0 {
        format!("${v:.2}")
    } else {
        format!("-${:.2}", v.abs())
    }
}

fn pretty_bucket_label(key: &str, mode: StatsTimeMode) -> String {
    match mode {
        StatsTimeMode::Month => {
            // "2026-02" → "Feb 2026"
            if let Some((y, m)) = key.split_once('-') {
                let m_num: u32 = m.parse().unwrap_or(0);
                return format!("{} {y}", month_label(m_num));
            }
            key.to_owned()
        }
        StatsTimeMode::Week => {
            // "2026-W08" → "W08 2026"
            if let Some((y, w)) = key.split_once("-W") {
                return format!("W{w} {y}");
            }
            key.to_owned()
        }
    }
}
