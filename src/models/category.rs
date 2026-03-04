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
        Category::default_category(
            CAT_TRANSPORT_ID,
            "Transport",
            "directions_transit",
            "#1E88E5",
        ),
        Category::default_category(CAT_LODGING_ID, "Lodging", "hotel", "#8E24AA"),
        Category::default_category(
            CAT_ENTERTAINMENT_ID,
            "Entertainment",
            "theater_comedy",
            "#F4511E",
        ),
        Category::default_category(CAT_SHOPPING_ID, "Shopping", "shopping_bag", "#00ACC1"),
        Category::default_category(CAT_FUEL_ID, "Fuel", "local_gas_station", "#6D4C41"),
        Category::default_category(CAT_COMMUNICATION_ID, "Communication", "phone", "#43A047"),
        Category::default_category(CAT_HEALTH_ID, "Health", "local_hospital", "#E91E63"),
        Category::default_category(CAT_OTHER_ID, "Other", "more_horiz", "#757575"),
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
            CAT_FOOD_ID,
            CAT_TRANSPORT_ID,
            CAT_LODGING_ID,
            CAT_ENTERTAINMENT_ID,
            CAT_SHOPPING_ID,
            CAT_FUEL_ID,
            CAT_COMMUNICATION_ID,
            CAT_HEALTH_ID,
            CAT_OTHER_ID,
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
