use crate::models::{Category, Receipt, Trip};
use rexie::Rexie;
use std::rc::Rc;
use yewdux::prelude::*;

#[derive(Store, Clone, Default)]
pub struct AppStore {
    pub trips: Vec<Trip>,
    pub current_trip: Option<Trip>,
    pub current_receipts: Vec<Receipt>,
    pub categories: Vec<Category>,
    pub loading: bool,
    pub error: Option<String>,
    pub db: Option<Rc<Rexie>>,
}

/// Manual PartialEq: compare db by presence only (Rexie doesn't implement PartialEq)
impl PartialEq for AppStore {
    fn eq(&self, other: &Self) -> bool {
        self.trips == other.trips
            && self.current_trip == other.current_trip
            && self.current_receipts == other.current_receipts
            && self.categories == other.categories
            && self.loading == other.loading
            && self.error == other.error
            && self.db.is_some() == other.db.is_some()
    }
}
