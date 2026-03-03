use rexie::{KeyRange, Rexie, TransactionMode};
use wasm_bindgen::JsValue;
use crate::error::{AppError, AppResult};
use crate::models::Receipt;
use super::db::STORE_RECEIPTS;

fn to_js(receipt: &Receipt) -> AppResult<JsValue> {
    serde_wasm_bindgen::to_value(receipt)
        .map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

fn from_js(val: JsValue) -> AppResult<Receipt> {
    serde_wasm_bindgen::from_value(val)
        .map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

pub async fn get_all_receipts(db: &Rexie) -> AppResult<Vec<Receipt>> {
    let tx = db
        .transaction(&[STORE_RECEIPTS], TransactionMode::ReadOnly)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_RECEIPTS).map_err(AppError::from)?;
    let items = store.get_all(None, None).await.map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;

    items.into_iter().map(from_js).collect()
}

pub async fn get_receipts_for_trip(db: &Rexie, trip_id: &str) -> AppResult<Vec<Receipt>> {
    let tx = db
        .transaction(&[STORE_RECEIPTS], TransactionMode::ReadOnly)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_RECEIPTS).map_err(AppError::from)?;
    let index = store.index("trip_id").map_err(AppError::from)?;
    let key = JsValue::from_str(trip_id);
    let key_range = KeyRange::only(&key)
        .map_err(|e| AppError::Database(format!("{:?}", e)))?;
    let items = index.get_all(Some(key_range), None).await.map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;

    let mut receipts: Vec<Receipt> = items
        .into_iter()
        .map(from_js)
        .collect::<AppResult<Vec<_>>>()?;
    receipts.sort_by(|a, b| b.date.cmp(&a.date));
    Ok(receipts)
}

pub async fn save_receipt(db: &Rexie, receipt: &Receipt) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_RECEIPTS], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_RECEIPTS).map_err(AppError::from)?;
    store.put(&to_js(receipt)?, None).await.map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}

pub async fn delete_receipt(db: &Rexie, id: &str) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_RECEIPTS], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_RECEIPTS).map_err(AppError::from)?;
    store.delete(JsValue::from_str(id)).await.map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}

pub async fn delete_receipts_for_trip(db: &Rexie, trip_id: &str) -> AppResult<()> {
    let receipts = get_receipts_for_trip(db, trip_id).await?;
    for r in receipts {
        delete_receipt(db, &r.id).await?;
    }
    Ok(())
}
