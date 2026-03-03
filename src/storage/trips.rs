use super::db::STORE_TRIPS;
use crate::error::{AppError, AppResult};
use crate::models::Trip;
use rexie::{Rexie, TransactionMode};
use wasm_bindgen::JsValue;

fn to_js(trip: &Trip) -> AppResult<JsValue> {
    serde_wasm_bindgen::to_value(trip).map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

fn from_js(val: JsValue) -> AppResult<Trip> {
    serde_wasm_bindgen::from_value(val).map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

pub async fn get_all_trips(db: &Rexie) -> AppResult<Vec<Trip>> {
    let tx = db
        .transaction(&[STORE_TRIPS], TransactionMode::ReadOnly)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_TRIPS).map_err(AppError::from)?;
    let items = store.get_all(None, None).await.map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;

    let mut trips: Vec<Trip> = items
        .into_iter()
        .map(from_js)
        .collect::<AppResult<Vec<_>>>()?;
    trips.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(trips)
}

pub async fn save_trip(db: &Rexie, trip: &Trip) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_TRIPS], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_TRIPS).map_err(AppError::from)?;
    store
        .put(&to_js(trip)?, None)
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}

pub async fn delete_trip(db: &Rexie, id: &str) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_TRIPS], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_TRIPS).map_err(AppError::from)?;
    store
        .delete(JsValue::from_str(id))
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}
