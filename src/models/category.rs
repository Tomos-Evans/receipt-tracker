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
pub const CAT_BREAKFAST_ID: &str = "00000000-0000-0000-0000-000000000001";
pub const CAT_LUNCH_ID: &str = "00000000-0000-0000-0000-000000000002";
pub const CAT_DINNER_ID: &str = "00000000-0000-0000-0000-000000000003";
pub const CAT_AIRPORT_TRANSFER_ID: &str = "00000000-0000-0000-0000-000000000004";
pub const CAT_OTHER_TRANSPORT_ID: &str = "00000000-0000-0000-0000-000000000005";
pub const CAT_FLIGHTS_ID: &str = "00000000-0000-0000-0000-000000000006";
pub const CAT_ACCOMMODATION_ID: &str = "00000000-0000-0000-0000-000000000007";
pub const CAT_CAR_HIRE_ID: &str = "00000000-0000-0000-0000-000000000008";
pub const CAT_FUEL_ID: &str = "00000000-0000-0000-0000-000000000009";

pub fn default_categories() -> Vec<Category> {
    vec![
        Category::default_category(CAT_BREAKFAST_ID, "Breakfast", "free_breakfast", "#F9A825"),
        Category::default_category(CAT_LUNCH_ID, "Lunch", "lunch_dining", "#43A047"),
        Category::default_category(CAT_DINNER_ID, "Dinner", "restaurant", "#E53935"),
        Category::default_category(
            CAT_AIRPORT_TRANSFER_ID,
            "Airport Transfer",
            "airport_shuttle",
            "#1E88E5",
        ),
        Category::default_category(
            CAT_OTHER_TRANSPORT_ID,
            "Other Transport",
            "directions_transit",
            "#039BE5",
        ),
        Category::default_category(CAT_FLIGHTS_ID, "Flights", "flight", "#0D47A1"),
        Category::default_category(CAT_ACCOMMODATION_ID, "Accommodation", "hotel", "#8E24AA"),
        Category::default_category(CAT_CAR_HIRE_ID, "Car Hire", "car_rental", "#00897B"),
        Category::default_category(CAT_FUEL_ID, "Fuel", "local_gas_station", "#6D4C41"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    /// There are exactly 9 built-in categories. Accidentally adding or removing
    /// one would silently change what gets seeded into new users' databases.
    #[wasm_bindgen_test]
    fn default_categories_count_is_nine() {
        assert_eq!(default_categories().len(), 9);
    }

    /// Every default category must carry the `is_default` flag so the storage
    /// layer can sort them above custom categories in the UI.
    #[wasm_bindgen_test]
    fn all_defaults_are_flagged_is_default() {
        for cat in default_categories() {
            assert!(cat.is_default, "'{}'  missing is_default flag", cat.name);
        }
    }

    /// Every default category has an icon and a colour. The UI renders both
    /// and will look broken if either is absent.
    #[wasm_bindgen_test]
    fn all_defaults_have_icon_and_color() {
        for cat in default_categories() {
            assert!(cat.icon.is_some(), "'{}' missing icon", cat.name);
            assert!(cat.color.is_some(), "'{}' missing color", cat.name);
        }
    }

    /// The IDs are hardcoded constants used by storage queries. If a constant
    /// drifts out of sync with the list, seeded rows can never be matched.
    #[wasm_bindgen_test]
    fn default_category_ids_match_constants() {
        let cats = default_categories();
        let ids: Vec<&str> = cats.iter().map(|c| c.id.as_str()).collect();
        for expected in [
            CAT_BREAKFAST_ID,
            CAT_LUNCH_ID,
            CAT_DINNER_ID,
            CAT_AIRPORT_TRANSFER_ID,
            CAT_OTHER_TRANSPORT_ID,
            CAT_FLIGHTS_ID,
            CAT_ACCOMMODATION_ID,
            CAT_CAR_HIRE_ID,
            CAT_FUEL_ID,
        ] {
            assert!(ids.contains(&expected), "missing constant ID {expected}");
        }
    }

    /// No two default categories share an ID. A duplicate would silently
    /// overwrite a row in IndexedDB during seeding.
    #[wasm_bindgen_test]
    fn default_category_ids_are_unique() {
        let cats = default_categories();
        let mut ids: Vec<&str> = cats.iter().map(|c| c.id.as_str()).collect();
        ids.sort_unstable();
        let original_len = ids.len();
        ids.dedup();
        assert_eq!(
            ids.len(),
            original_len,
            "duplicate IDs in default_categories()"
        );
    }
}
