//! `IndexedDB` storage helpers for the finance tracker (wasm32 only).
//!
//! Uses the `rexie` crate to persist accounts and transactions in the
//! browser's `IndexedDB`, avoiding the ~5 MB localStorage limit.

use rexie::{ObjectStore, Rexie, TransactionMode};
use serde::{Serialize, de::DeserializeOwned};
use web_sys::wasm_bindgen::JsValue;

const DB_NAME: &str = "personars_finance";
const DB_VERSION: u32 = 1;
const STORE_ACCOUNTS: &str = "accounts";
const STORE_TRANSACTIONS: &str = "transactions";

/// Open (or create) the `IndexedDB` database.
pub(crate) async fn open_db() -> Result<Rexie, rexie::Error> {
    Rexie::builder(DB_NAME)
        .version(DB_VERSION)
        .add_object_store(ObjectStore::new(STORE_ACCOUNTS).auto_increment(true))
        .add_object_store(ObjectStore::new(STORE_TRANSACTIONS).auto_increment(true))
        .build()
        .await
}

/// Replace all items in a store with the given collection.
async fn save_all<T: Serialize>(db: &Rexie, store_name: &str, items: &[T]) -> Result<(), String> {
    let tx = db
        .transaction(&[store_name], TransactionMode::ReadWrite)
        .map_err(|e| format!("txn open: {e}"))?;
    let store = tx.store(store_name).map_err(|e| format!("store: {e}"))?;

    // Clear existing data
    store.clear().await.map_err(|e| format!("clear: {e}"))?;

    // Write each item
    for item in items {
        let js_val = serde_wasm_bindgen::to_value(item).map_err(|e| format!("serialize: {e}"))?;
        store
            .add(&js_val, None)
            .await
            .map_err(|e| format!("add: {e}"))?;
    }

    tx.done().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

/// Load all items from a store.
async fn load_all<T: DeserializeOwned>(db: &Rexie, store_name: &str) -> Result<Vec<T>, String> {
    let tx = db
        .transaction(&[store_name], TransactionMode::ReadOnly)
        .map_err(|e| format!("txn open: {e}"))?;
    let store = tx.store(store_name).map_err(|e| format!("store: {e}"))?;

    let entries: Vec<JsValue> = store
        .get_all(None, None)
        .await
        .map_err(|e| format!("get_all: {e}"))?;

    let mut result = Vec::with_capacity(entries.len());
    for value in entries {
        let item: T =
            serde_wasm_bindgen::from_value(value).map_err(|e| format!("deserialize: {e}"))?;
        result.push(item);
    }
    Ok(result)
}

// -- Public API wrappers ----------------------------------------------------

pub(crate) async fn save_accounts<T: Serialize>(db: &Rexie, accounts: &[T]) -> Result<(), String> {
    save_all(db, STORE_ACCOUNTS, accounts).await
}

pub(crate) async fn save_transactions<T: Serialize>(db: &Rexie, txns: &[T]) -> Result<(), String> {
    save_all(db, STORE_TRANSACTIONS, txns).await
}

pub(crate) async fn load_accounts<T: DeserializeOwned>(db: &Rexie) -> Result<Vec<T>, String> {
    load_all(db, STORE_ACCOUNTS).await
}

pub(crate) async fn load_transactions<T: DeserializeOwned>(db: &Rexie) -> Result<Vec<T>, String> {
    load_all(db, STORE_TRANSACTIONS).await
}
