use super::db::STORE_PHOTOS;
use crate::error::{AppError, AppResult};
use rexie::{Rexie, TransactionMode};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsValue;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoRecord {
    pub receipt_id: String,
    /// Base64-encoded JPEG data URI: "data:image/jpeg;base64,..."
    pub data: String,
}

fn to_js(record: &PhotoRecord) -> AppResult<JsValue> {
    serde_wasm_bindgen::to_value(record).map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

fn from_js(val: JsValue) -> AppResult<PhotoRecord> {
    serde_wasm_bindgen::from_value(val).map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

pub async fn save_photo(db: &Rexie, receipt_id: &str, data: String) -> AppResult<()> {
    let record = PhotoRecord {
        receipt_id: receipt_id.to_string(),
        data,
    };
    let tx = db
        .transaction(&[STORE_PHOTOS], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_PHOTOS).map_err(AppError::from)?;
    store
        .put(&to_js(&record)?, None)
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}

pub async fn get_photo(db: &Rexie, receipt_id: &str) -> AppResult<Option<String>> {
    let tx = db
        .transaction(&[STORE_PHOTOS], TransactionMode::ReadOnly)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_PHOTOS).map_err(AppError::from)?;
    let val = store
        .get(JsValue::from_str(receipt_id))
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;

    match val {
        Some(v) => {
            let record = from_js(v)?;
            Ok(Some(record.data))
        }
        None => Ok(None),
    }
}

pub async fn delete_photo(db: &Rexie, receipt_id: &str) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_PHOTOS], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_PHOTOS).map_err(AppError::from)?;
    store
        .delete(JsValue::from_str(receipt_id))
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}
