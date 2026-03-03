use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub trip_id: String,
    pub amount: f64,
    pub category_id: String,
    pub notes: Option<String>,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

impl Receipt {
    pub fn new(
        trip_id: String,
        amount: f64,
        category_id: String,
        notes: Option<String>,
        date: NaiveDate,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            trip_id,
            amount,
            category_id,
            notes,
            date,
            created_at: Utc::now(),
        }
    }

    pub fn formatted_amount(&self, currency: &str) -> String {
        format!("{} {:.2}", currency, self.amount)
    }
}
