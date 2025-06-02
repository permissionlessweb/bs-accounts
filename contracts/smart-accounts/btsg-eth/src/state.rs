use cw_storage_plus::Item;

/// Stores the contract's configuration
pub const PUBLIC_KEY: Item<String> = Item::new("pk");
