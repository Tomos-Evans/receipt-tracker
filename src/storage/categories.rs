use super::db::STORE_CATEGORIES;
use crate::error::{AppError, AppResult};
use crate::models::Category;
use rexie::{Rexie, TransactionMode};
use wasm_bindgen::JsValue;

fn to_js(cat: &Category) -> AppResult<JsValue> {
    serde_wasm_bindgen::to_value(cat).map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

fn from_js(val: JsValue) -> AppResult<Category> {
    serde_wasm_bindgen::from_value(val).map_err(|e| AppError::Serialization(format!("{:?}", e)))
}

pub async fn get_all_categories(db: &Rexie) -> AppResult<Vec<Category>> {
    let tx = db
        .transaction(&[STORE_CATEGORIES], TransactionMode::ReadOnly)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_CATEGORIES).map_err(AppError::from)?;
    let items = store.get_all(None, None).await.map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;

    let mut cats: Vec<Category> = items
        .into_iter()
        .map(from_js)
        .collect::<AppResult<Vec<_>>>()?;
    // Defaults first, then custom alphabetically
    cats.sort_by(|a, b| b.is_default.cmp(&a.is_default).then(a.name.cmp(&b.name)));
    Ok(cats)
}

pub async fn save_category(db: &Rexie, cat: &Category) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_CATEGORIES], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_CATEGORIES).map_err(AppError::from)?;
    store
        .put(&to_js(cat)?, None)
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}

pub async fn delete_category(db: &Rexie, id: &str) -> AppResult<()> {
    let tx = db
        .transaction(&[STORE_CATEGORIES], TransactionMode::ReadWrite)
        .map_err(AppError::from)?;
    let store = tx.store(STORE_CATEGORIES).map_err(AppError::from)?;
    store
        .delete(JsValue::from_str(id))
        .await
        .map_err(AppError::from)?;
    tx.done().await.map_err(AppError::from)?;
    Ok(())
}
