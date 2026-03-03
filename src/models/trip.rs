use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Trip {
    pub id: String,
    pub name: String,
    pub currency: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Trip {
    pub fn new(name: String, currency: String, start_date: NaiveDate, end_date: NaiveDate) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            currency,
            start_date,
            end_date,
            created_at: now,
            updated_at: now,
        }
    }
}
