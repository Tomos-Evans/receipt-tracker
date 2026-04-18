use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_currency() -> String {
    "USD".to_string()
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Receipt {
    pub id: String,
    pub trip_id: String,
    pub amount: f64,
    pub category_id: String,
    pub notes: Option<String>,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
    #[serde(default = "default_currency")]
    pub currency: String,
}

impl Receipt {
    pub fn new(
        trip_id: String,
        amount: f64,
        category_id: String,
        notes: Option<String>,
        date: NaiveDate,
        currency: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            trip_id,
            amount,
            category_id,
            notes,
            date,
            created_at: Utc::now(),
            currency,
        }
    }
}
