use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub is_default: bool,
    pub color: Option<String>,
}

impl Category {
    pub fn new(name: String, icon: Option<String>, color: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            icon,
            is_default: false,
            color,
        }
    }

    fn default_category(id: &str, name: &str, icon: &str, color: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            icon: Some(icon.to_string()),
            is_default: true,
            color: Some(color.to_string()),
        }
    }
}

/// Stable hardcoded UUIDs for default categories
pub const CAT_FOOD_ID: &str = "00000000-0000-0000-0000-000000000001";
pub const CAT_TRANSPORT_ID: &str = "00000000-0000-0000-0000-000000000002";
pub const CAT_LODGING_ID: &str = "00000000-0000-0000-0000-000000000003";
pub const CAT_ENTERTAINMENT_ID: &str = "00000000-0000-0000-0000-000000000004";
pub const CAT_SHOPPING_ID: &str = "00000000-0000-0000-0000-000000000005";
pub const CAT_FUEL_ID: &str = "00000000-0000-0000-0000-000000000006";
pub const CAT_COMMUNICATION_ID: &str = "00000000-0000-0000-0000-000000000007";
pub const CAT_HEALTH_ID: &str = "00000000-0000-0000-0000-000000000008";
pub const CAT_OTHER_ID: &str = "00000000-0000-0000-0000-000000000009";

pub fn default_categories() -> Vec<Category> {
    vec![
        Category::default_category(CAT_FOOD_ID, "Food & Drink", "restaurant", "#E53935"),
        Category::default_category(CAT_TRANSPORT_ID, "Transport", "directions_transit", "#1E88E5"),
        Category::default_category(CAT_LODGING_ID, "Lodging", "hotel", "#8E24AA"),
        Category::default_category(CAT_ENTERTAINMENT_ID, "Entertainment", "theater_comedy", "#F4511E"),
        Category::default_category(CAT_SHOPPING_ID, "Shopping", "shopping_bag", "#00ACC1"),
        Category::default_category(CAT_FUEL_ID, "Fuel", "local_gas_station", "#6D4C41"),
        Category::default_category(CAT_COMMUNICATION_ID, "Communication", "phone", "#43A047"),
        Category::default_category(CAT_HEALTH_ID, "Health", "local_hospital", "#E91E63"),
        Category::default_category(CAT_OTHER_ID, "Other", "more_horiz", "#757575"),
    ]
}

pub const DEFAULT_CATEGORIES: &[(&str, &str, &str, &str)] = &[
    (CAT_FOOD_ID, "Food & Drink", "restaurant", "#E53935"),
    (CAT_TRANSPORT_ID, "Transport", "directions_transit", "#1E88E5"),
    (CAT_LODGING_ID, "Lodging", "hotel", "#8E24AA"),
    (CAT_ENTERTAINMENT_ID, "Entertainment", "theater_comedy", "#F4511E"),
    (CAT_SHOPPING_ID, "Shopping", "shopping_bag", "#00ACC1"),
    (CAT_FUEL_ID, "Fuel", "local_gas_station", "#6D4C41"),
    (CAT_COMMUNICATION_ID, "Communication", "phone", "#43A047"),
    (CAT_HEALTH_ID, "Health", "local_hospital", "#E91E63"),
    (CAT_OTHER_ID, "Other", "more_horiz", "#757575"),
];
