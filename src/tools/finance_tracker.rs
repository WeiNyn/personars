use super::Tool;
use chrono::{DateTime, Utc};
use eframe::egui::{self, Color32, Layout, RichText};
use egui_extras::{Size, StripBuilder};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
pub enum TransactionType {
    #[default]
    Receive,
    Pay,
}

#[derive(Clone, Deserialize, Serialize)]
struct Account {
    id: Uuid,
    name: String,
    icon: String,
}

#[derive(Clone, Deserialize, Serialize)]
struct Transaction {
    id: Uuid,
    transaction_type: TransactionType,
    amount: f64,
    description: String,
    account_id: Uuid,
    #[serde(with = "chrono::serde::ts_seconds")]
    created_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Narrow-mode tabs
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
enum NarrowTab {
    #[default]
    Accounts,
    Transactions,
}

// ---------------------------------------------------------------------------
// Main state
// ---------------------------------------------------------------------------

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub struct FinanceTracker {
    // On wasm32 these are stored in IndexedDB instead of eframe localStorage.
    #[cfg_attr(target_arch = "wasm32", serde(skip))]
    accounts: Vec<Account>,
    #[cfg_attr(target_arch = "wasm32", serde(skip))]
    transactions: Vec<Transaction>,

    // -- UI / input state (not persisted across restarts) --
    #[serde(skip)]
    input_type: TransactionType,
    #[serde(skip)]
    input_amount: String,
    #[serde(skip)]
    input_description: String,
    #[serde(skip)]
    input_account_idx: usize,
    #[serde(skip)]
    new_account_name: String,
    #[serde(skip)]
    new_account_icon: String,
    #[serde(skip)]
    narrow_tab: NarrowTab,
    #[serde(skip)]
    filter_account_idx: Option<usize>,

    // -- wasm32-only: IndexedDB async state --
    #[cfg(target_arch = "wasm32")]
    #[serde(skip)]
    idb_state: Arc<Mutex<IdbState>>,
}

impl Default for FinanceTracker {
    fn default() -> Self {
        let cash = Account {
            id: Uuid::new_v4(),
            name: "Cash".to_owned(),
            icon: "💵".to_owned(),
        };
        let card = Account {
            id: Uuid::new_v4(),
            name: "Credit Card".to_owned(),
            icon: "💳".to_owned(),
        };
        let bank = Account {
            id: Uuid::new_v4(),
            name: "Bank".to_owned(),
            icon: "🏦".to_owned(),
        };

        Self {
            accounts: vec![cash, card, bank],
            transactions: Vec::new(),
            input_type: TransactionType::default(),
            input_amount: String::new(),
            input_description: String::new(),
            input_account_idx: 0,
            new_account_name: String::new(),
            new_account_icon: "💰".to_owned(),
            narrow_tab: NarrowTab::default(),
            filter_account_idx: None,
            #[cfg(target_arch = "wasm32")]
            idb_state: Arc::new(Mutex::new(IdbState::default())),
        }
    }
}

// ---------------------------------------------------------------------------
// Tool trait (used when shown as a windowed tool)
// ---------------------------------------------------------------------------

impl Tool for FinanceTracker {
    fn name(&self) -> &'static str {
        "Finance Tracker"
    }

    fn icon_name(&self) -> &'static str {
        egui_phosphor::regular::CURRENCY_CIRCLE_DOLLAR
    }

    fn show(&mut self, ctx: &egui::Context, open: &mut bool, rect: egui::Rect) {
        egui::Window::new(format!("{} {}", self.icon_name(), self.name()))
            .open(open)
            .default_width(700.0)
            .default_height(500.0)
            .resizable(true)
            .constrain_to(rect)
            .show(ctx, |ui| {
                self.render_layout(ui);
            });
    }

    fn show_narrow(&mut self, ui: &mut egui::Ui) {
        self.render_narrow(ui);
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

impl FinanceTracker {
    /// Net total across all accounts (receives − pays).
    fn net_total(&self) -> f64 {
        self.transactions
            .iter()
            .fold(0.0, |acc, t| match t.transaction_type {
                TransactionType::Receive => acc + t.amount,
                TransactionType::Pay => acc - t.amount,
            })
    }

    /// Balance for a single account.
    fn account_balance(&self, account_id: Uuid) -> f64 {
        self.transactions
            .iter()
            .filter(|t| t.account_id == account_id)
            .fold(0.0, |acc, t| match t.transaction_type {
                TransactionType::Receive => acc + t.amount,
                TransactionType::Pay => acc - t.amount,
            })
    }

    /// Try to parse `input_amount` and add a transaction.
    fn try_add_transaction(&mut self) {
        let amount: f64 = match self.input_amount.trim().parse() {
            Ok(v) if v > 0.0 => v,
            _ => return,
        };
        let account_id = match self.accounts.get(self.input_account_idx) {
            Some(a) => a.id,
            None => return,
        };

        self.transactions.push(Transaction {
            id: Uuid::new_v4(),
            transaction_type: self.input_type,
            amount,
            description: self.input_description.clone(),
            account_id,
            created_at: Utc::now(),
        });

        // Reset input fields
        self.input_amount.clear();
        self.input_description.clear();

        #[cfg(target_arch = "wasm32")]
        self.save_to_idb();
    }

    fn try_add_account(&mut self) {
        let name = self.new_account_name.trim().to_owned();
        if name.is_empty() {
            return;
        }
        let icon = if self.new_account_icon.trim().is_empty() {
            "💰".to_owned()
        } else {
            self.new_account_icon.trim().to_owned()
        };
        self.accounts.push(Account {
            id: Uuid::new_v4(),
            name,
            icon,
        });
        self.new_account_name.clear();
        self.new_account_icon = "💰".to_owned();

        #[cfg(target_arch = "wasm32")]
        self.save_to_idb();
    }
}

// ---------------------------------------------------------------------------
// Desktop layout (≥ 500 px)
// ---------------------------------------------------------------------------

impl FinanceTracker {
    /// 2-pane layout: accounts sidebar | main content.
    pub fn render_layout(&mut self, ui: &mut egui::Ui) {
        #[cfg(target_arch = "wasm32")]
        self.poll_idb();

        StripBuilder::new(ui)
            .size(Size::relative(0.22).at_least(120.0))
            .size(Size::exact(1.0))
            .size(Size::remainder())
            .horizontal(|mut strip| {
                strip.cell(|ui| {
                    ui.set_clip_rect(ui.max_rect());
                    ui.set_min_width(0.0);
                    self.render_accounts_sidebar(ui);
                });
                strip.cell(|ui| {
                    ui.add(egui::Separator::default().vertical());
                });
                strip.cell(|ui| {
                    ui.set_clip_rect(ui.max_rect());
                    ui.set_min_width(0.0);
                    self.render_main_content(ui);
                });
            });
    }

    // -- Accounts sidebar --------------------------------------------------

    fn render_accounts_sidebar(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Accounts");
            ui.separator();

            // Add account mini-form
            ui.horizontal(|ui| {
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_account_icon)
                        .desired_width(30.0)
                        .hint_text("🏷"),
                );
                ui.add(
                    egui::TextEdit::singleline(&mut self.new_account_name)
                        .desired_width(ui.available_width() - 30.0)
                        .hint_text("Name"),
                );
            });
            if ui
                .button(format!("{} Add Account", egui_phosphor::regular::PLUS))
                .clicked()
            {
                self.try_add_account();
            }
            ui.separator();

            // Account list
            let mut delete_idx: Option<usize> = None;

            egui::ScrollArea::vertical()
                .id_salt("accounts_scroll")
                .show(ui, |ui| {
                    for (i, account) in self.accounts.iter().enumerate() {
                        let balance = self.account_balance(account.id);
                        let color = balance_color(balance, ui);

                        ui.horizontal(|ui| {
                            let label = format!(
                                "{} {}  {}",
                                account.icon,
                                account.name,
                                format_money(balance),
                            );
                            ui.label(RichText::new(label).color(color));

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui
                                    .small_button("🗑")
                                    .on_hover_text("Delete account")
                                    .clicked()
                                {
                                    delete_idx = Some(i);
                                }
                            });
                        });

                        ui.add_space(2.0);
                    }
                });

            if let Some(idx) = delete_idx {
                if let Some(account) = self.accounts.get(idx) {
                    let aid = account.id;
                    // Remove transactions tied to this account
                    self.transactions.retain(|t| t.account_id != aid);
                }
                self.accounts.remove(idx);
                // Clamp input index
                if self.input_account_idx >= self.accounts.len() && !self.accounts.is_empty() {
                    self.input_account_idx = self.accounts.len() - 1;
                }

                #[cfg(target_arch = "wasm32")]
                self.save_to_idb();
            }
        });
    }

    // -- Main content (net total + form + history) -------------------------

    fn render_main_content(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            self.render_net_total_banner(ui);
            ui.add_space(6.0);
            self.render_add_transaction_form(ui);
            ui.add_space(6.0);
            ui.separator();
            self.render_transaction_list(ui);
        });
    }
}

// ---------------------------------------------------------------------------
// Narrow / compact layout (< 500 px)
// ---------------------------------------------------------------------------

impl FinanceTracker {
    fn render_narrow(&mut self, ui: &mut egui::Ui) {
        #[cfg(target_arch = "wasm32")]
        self.poll_idb();

        ui.vertical(|ui| {
            self.render_net_total_banner(ui);
            ui.separator();

            // Tab bar
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.narrow_tab, NarrowTab::Accounts, "🏦 Accounts");
                ui.selectable_value(
                    &mut self.narrow_tab,
                    NarrowTab::Transactions,
                    "📋 Transactions",
                );
            });
            ui.separator();

            match self.narrow_tab {
                NarrowTab::Accounts => self.render_accounts_sidebar(ui),
                NarrowTab::Transactions => {
                    self.render_add_transaction_form(ui);
                    ui.separator();
                    self.render_transaction_list(ui);
                }
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Shared widgets
// ---------------------------------------------------------------------------

impl FinanceTracker {
    fn render_net_total_banner(&self, ui: &mut egui::Ui) {
        let total = self.net_total();
        let color = balance_color(total, ui);

        egui::Frame::new()
            .inner_margin(egui::Margin::same(10))
            .corner_radius(6.0)
            .fill(ui.visuals().extreme_bg_color)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new("Net Total").size(13.0));
                    ui.label(
                        RichText::new(format_money(total))
                            .size(26.0)
                            .strong()
                            .color(color),
                    );
                });
            });
    }

    fn render_add_transaction_form(&mut self, ui: &mut egui::Ui) {
        ui.group(|ui| {
            ui.label(RichText::new("Add Transaction").strong());
            ui.add_space(4.0);

            // Type toggle
            ui.horizontal(|ui| {
                ui.selectable_value(
                    &mut self.input_type,
                    TransactionType::Receive,
                    RichText::new(format!("{} Receive", egui_phosphor::regular::ARROW_DOWN))
                        .color(Color32::from_rgb(46, 160, 67)),
                );
                ui.selectable_value(
                    &mut self.input_type,
                    TransactionType::Pay,
                    RichText::new(format!("{} Pay", egui_phosphor::regular::ARROW_UP))
                        .color(Color32::from_rgb(218, 54, 51)),
                );
            });

            ui.add_space(2.0);

            // Amount + Account
            ui.horizontal(|ui| {
                ui.label("Amount:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_amount)
                        .desired_width(100.0)
                        .hint_text("0.00"),
                );

                ui.label("Account:");
                let selected_name = self
                    .accounts
                    .get(self.input_account_idx)
                    .map_or("—".to_owned(), |a| format!("{} {}", a.icon, a.name));
                egui::ComboBox::from_id_salt("txn_account_combo")
                    .selected_text(selected_name)
                    .show_ui(ui, |ui| {
                        for (i, account) in self.accounts.iter().enumerate() {
                            let label = format!("{} {}", account.icon, account.name);
                            ui.selectable_value(&mut self.input_account_idx, i, label);
                        }
                    });
            });

            // Description
            ui.horizontal(|ui| {
                ui.label("Desc:");
                ui.add(
                    egui::TextEdit::singleline(&mut self.input_description)
                        .desired_width(ui.available_width() - 110.0)
                        .hint_text("What for?"),
                );

                if ui
                    .button(format!("{} Add", egui_phosphor::regular::PLUS))
                    .clicked()
                {
                    self.try_add_transaction();
                }
            });
        });
    }

    fn render_transaction_list(&mut self, ui: &mut egui::Ui) {
        // Optional account filter
        ui.horizontal(|ui| {
            ui.label("Filter:");
            let filter_label = match self.filter_account_idx {
                Some(i) => self
                    .accounts
                    .get(i)
                    .map_or("All".to_owned(), |a| format!("{} {}", a.icon, a.name)),
                None => "All".to_owned(),
            };
            egui::ComboBox::from_id_salt("txn_filter_combo")
                .selected_text(filter_label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.filter_account_idx, None, "All");
                    for (i, account) in self.accounts.iter().enumerate() {
                        let label = format!("{} {}", account.icon, account.name);
                        ui.selectable_value(&mut self.filter_account_idx, Some(i), label);
                    }
                });
        });

        ui.add_space(4.0);

        let filter_account_id = self
            .filter_account_idx
            .and_then(|i| self.accounts.get(i).map(|a| a.id));

        // Build filtered & sorted view (newest first)
        let mut display: Vec<(usize, &Transaction)> = self
            .transactions
            .iter()
            .enumerate()
            .filter(|(_, t)| filter_account_id.is_none() || filter_account_id == Some(t.account_id))
            .collect();
        display.sort_by(|a, b| b.1.created_at.cmp(&a.1.created_at));

        let mut delete_idx: Option<usize> = None;

        egui::ScrollArea::vertical()
            .id_salt("txn_scroll")
            .show(ui, |ui| {
                if display.is_empty() {
                    ui.label(
                        RichText::new("No transactions yet.").color(ui.visuals().weak_text_color()),
                    );
                }

                for &(orig_idx, txn) in &display {
                    let (arrow, type_color) = match txn.transaction_type {
                        TransactionType::Receive => (
                            egui_phosphor::regular::ARROW_DOWN,
                            Color32::from_rgb(46, 160, 67),
                        ),
                        TransactionType::Pay => (
                            egui_phosphor::regular::ARROW_UP,
                            Color32::from_rgb(218, 54, 51),
                        ),
                    };

                    let sign = match txn.transaction_type {
                        TransactionType::Receive => "+",
                        TransactionType::Pay => "-",
                    };

                    let account_label = self
                        .accounts
                        .iter()
                        .find(|a| a.id == txn.account_id)
                        .map_or("?".to_owned(), |a| format!("{} {}", a.icon, a.name));

                    let date_str = txn.created_at.format("%m-%d %H:%M").to_string();

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&date_str)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );
                        ui.label(RichText::new(arrow).color(type_color).strong());
                        ui.label(
                            RichText::new(format!("{sign}${:.2}", txn.amount))
                                .color(type_color)
                                .strong(),
                        );
                        ui.label(&txn.description);
                        ui.label(
                            RichText::new(&account_label)
                                .size(11.0)
                                .color(ui.visuals().weak_text_color()),
                        );

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                                delete_idx = Some(orig_idx);
                            }
                        });
                    });

                    ui.add_space(1.0);
                }
            });

        if let Some(idx) = delete_idx {
            if idx < self.transactions.len() {
                self.transactions.remove(idx);

                #[cfg(target_arch = "wasm32")]
                self.save_to_idb();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// wasm32: IndexedDB async bridge
// ---------------------------------------------------------------------------

#[cfg(target_arch = "wasm32")]
#[derive(Default)]
struct IdbState {
    /// Data loaded from `IndexedDB` (set once, consumed by `poll_idb`).
    loaded: Option<(Vec<Account>, Vec<Transaction>)>,
    /// Whether the initial load has been kicked off.
    init_started: bool,
}

#[cfg(target_arch = "wasm32")]
impl FinanceTracker {
    /// Kick off async load from `IndexedDB`. Call once after construction.
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
            let db = match super::idb_storage::open_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open IndexedDB: {e}");
                    return;
                }
            };

            let accounts: Vec<Account> = super::idb_storage::load_accounts(&db)
                .await
                .unwrap_or_default();
            let transactions: Vec<Transaction> = super::idb_storage::load_transactions(&db)
                .await
                .unwrap_or_default();

            if let Ok(mut s) = state.lock() {
                s.loaded = Some((accounts, transactions));
            }
        });
    }

    /// Poll for completed async load and merge data into self.
    fn poll_idb(&mut self) {
        let loaded = {
            let mut s = self.idb_state.lock().expect("lock poisoned");
            s.loaded.take()
        };
        if let Some((accounts, transactions)) = loaded {
            // Only apply if we got real data; otherwise keep defaults
            if !accounts.is_empty() || !transactions.is_empty() {
                self.accounts = accounts;
                self.transactions = transactions;
            }
        }
    }

    /// Spawn an async task to persist current data to `IndexedDB`.
    fn save_to_idb(&self) {
        let accounts = self.accounts.clone();
        let transactions = self.transactions.clone();

        wasm_bindgen_futures::spawn_local(async move {
            let db = match super::idb_storage::open_db().await {
                Ok(db) => db,
                Err(e) => {
                    log::error!("Failed to open IndexedDB for save: {e}");
                    return;
                }
            };

            if let Err(e) = super::idb_storage::save_accounts(&db, &accounts).await {
                log::error!("Failed to save accounts: {e}");
            }
            if let Err(e) = super::idb_storage::save_transactions(&db, &transactions).await {
                log::error!("Failed to save transactions: {e}");
            }
        });
    }
}

// ---------------------------------------------------------------------------
// Utility
// ---------------------------------------------------------------------------

fn format_money(amount: f64) -> String {
    if amount >= 0.0 {
        format!("${amount:.2}")
    } else {
        format!("-${:.2}", amount.abs())
    }
}

fn balance_color(amount: f64, ui: &egui::Ui) -> Color32 {
    if amount > 0.0 {
        Color32::from_rgb(46, 160, 67) // green
    } else if amount < 0.0 {
        Color32::from_rgb(218, 54, 51) // red
    } else {
        ui.visuals().text_color() // neutral
    }
}
