//! `IndexedDB` storage helpers (wasm32 only).
//!
//! Uses the `rexie` crate to persist data in the browser's `IndexedDB`,
//! avoiding the ~5 MB localStorage limit.
//!
//! Two separate databases:
//! - `personars_finance` – accounts & transactions
//! - `personars_notes`   – note index (metadata) & note content (lazy-loaded)

use rexie::{ObjectStore, Rexie, TransactionMode};
use serde::{Serialize, de::DeserializeOwned};
use web_sys::wasm_bindgen::JsValue;

// ---------------------------------------------------------------------------
// Finance DB
// ---------------------------------------------------------------------------

const DB_NAME: &str = "personars_finance";
const DB_VERSION: u32 = 1;
const STORE_ACCOUNTS: &str = "accounts";
const STORE_TRANSACTIONS: &str = "transactions";

/// Open (or create) the finance `IndexedDB` database.
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

// -- Finance public API wrappers --------------------------------------------

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

// ---------------------------------------------------------------------------
// Notes DB — lazy-loading design
// ---------------------------------------------------------------------------

const NOTES_DB_NAME: &str = "personars_notes";
const NOTES_DB_VERSION: u32 = 1;
const STORE_NOTE_INDEX: &str = "note_index";
const STORE_NOTE_CONTENT: &str = "note_content";

/// Open (or create) the notes `IndexedDB` database.
///
/// Two stores, both keyed by `"id"` (UUID string):
/// - `note_index`   – lightweight metadata (id, title, dates)
/// - `note_content` – full markdown content
pub(crate) async fn open_notes_db() -> Result<Rexie, rexie::Error> {
    Rexie::builder(NOTES_DB_NAME)
        .version(NOTES_DB_VERSION)
        .add_object_store(ObjectStore::new(STORE_NOTE_INDEX).key_path("id"))
        .add_object_store(ObjectStore::new(STORE_NOTE_CONTENT).key_path("id"))
        .build()
        .await
}

// -- Notes: index (metadata) ------------------------------------------------

/// Replace all note index entries.
pub(crate) async fn save_note_index<T: Serialize>(db: &Rexie, items: &[T]) -> Result<(), String> {
    save_all(db, STORE_NOTE_INDEX, items).await
}

/// Load all note index entries.
pub(crate) async fn load_note_index<T: DeserializeOwned>(db: &Rexie) -> Result<Vec<T>, String> {
    load_all(db, STORE_NOTE_INDEX).await
}

// -- Notes: content (per-note) ----------------------------------------------

/// Save (upsert) a single note's content.
///
/// The value must serialize to an object with an `"id"` field that matches
/// the store's key-path.
pub(crate) async fn save_single_note_content<T: Serialize>(
    db: &Rexie,
    content: &T,
) -> Result<(), String> {
    let tx = db
        .transaction(&[STORE_NOTE_CONTENT], TransactionMode::ReadWrite)
        .map_err(|e| format!("txn open: {e}"))?;
    let store = tx
        .store(STORE_NOTE_CONTENT)
        .map_err(|e| format!("store: {e}"))?;

    let js_val = serde_wasm_bindgen::to_value(content).map_err(|e| format!("serialize: {e}"))?;
    store
        .put(&js_val, None)
        .await
        .map_err(|e| format!("put: {e}"))?;

    tx.done().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}

/// Load a single note's content by its UUID string key.
pub(crate) async fn load_single_note_content<T: DeserializeOwned>(
    db: &Rexie,
    id: &str,
) -> Result<Option<T>, String> {
    let tx = db
        .transaction(&[STORE_NOTE_CONTENT], TransactionMode::ReadOnly)
        .map_err(|e| format!("txn open: {e}"))?;
    let store = tx
        .store(STORE_NOTE_CONTENT)
        .map_err(|e| format!("store: {e}"))?;

    let key = JsValue::from_str(id);
    let js_val = store.get(key).await.map_err(|e| format!("get: {e}"))?;

    let Some(js_val) = js_val else {
        return Ok(None);
    };

    let item: T =
        serde_wasm_bindgen::from_value(js_val).map_err(|e| format!("deserialize: {e}"))?;
    Ok(Some(item))
}

/// Delete a note from both the index and content stores.
pub(crate) async fn delete_note(db: &Rexie, id: &str) -> Result<(), String> {
    let tx = db
        .transaction(
            &[STORE_NOTE_INDEX, STORE_NOTE_CONTENT],
            TransactionMode::ReadWrite,
        )
        .map_err(|e| format!("txn open: {e}"))?;

    let key = JsValue::from_str(id);

    let idx_store = tx
        .store(STORE_NOTE_INDEX)
        .map_err(|e| format!("store idx: {e}"))?;
    idx_store
        .delete(key.clone())
        .await
        .map_err(|e| format!("delete idx: {e}"))?;

    let cnt_store = tx
        .store(STORE_NOTE_CONTENT)
        .map_err(|e| format!("store cnt: {e}"))?;
    cnt_store
        .delete(key)
        .await
        .map_err(|e| format!("delete cnt: {e}"))?;

    tx.done().await.map_err(|e| format!("commit: {e}"))?;
    Ok(())
}
