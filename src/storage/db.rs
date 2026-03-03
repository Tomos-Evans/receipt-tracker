use rexie::{ObjectStore, Index, Rexie};
use crate::error::{AppError, AppResult};
use crate::models::category::default_categories;
use super::categories;

pub const DB_NAME: &str = "receipt_tracker_db";
pub const DB_VERSION: u32 = 1;

pub const STORE_TRIPS: &str = "trips";
pub const STORE_RECEIPTS: &str = "receipts";
pub const STORE_CATEGORIES: &str = "categories";
pub const STORE_PHOTOS: &str = "receipt_photos";

pub async fn open_database() -> AppResult<Rexie> {
    let rexie = Rexie::builder(DB_NAME)
        .version(DB_VERSION)
        .add_object_store(
            ObjectStore::new(STORE_TRIPS)
                .key_path("id")
                .add_index(Index::new("created_at", "created_at"))
                .add_index(Index::new("start_date", "start_date")),
        )
        .add_object_store(
            ObjectStore::new(STORE_RECEIPTS)
                .key_path("id")
                .add_index(Index::new("trip_id", "trip_id"))
                .add_index(Index::new("date", "date"))
                .add_index(Index::new("category_id", "category_id")),
        )
        .add_object_store(
            ObjectStore::new(STORE_CATEGORIES)
                .key_path("id")
                .add_index(Index::new("is_default", "is_default"))
                .add_index(Index::new("name", "name")),
        )
        .add_object_store(
            ObjectStore::new(STORE_PHOTOS)
                .key_path("receipt_id"),
        )
        .build()
        .await
        .map_err(AppError::from)?;

    Ok(rexie)
}

pub async fn seed_categories(db: &Rexie) -> AppResult<()> {
    let existing = categories::get_all_categories(db).await?;
    if !existing.is_empty() {
        return Ok(());
    }

    let defaults = default_categories();
    for cat in defaults {
        categories::save_category(db, &cat).await?;
    }
    log::info!("Seeded {} default categories", existing.len());
    Ok(())
}
