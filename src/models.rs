use serde::{Serialize, Deserialize};
use time::OffsetDateTime;
use sqlx::types::Json;

#[derive(Serialize, Clone)]
pub struct Asset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct UserRecord {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
}

#[derive(Serialize, Deserialize)]
pub struct PurchaseHistory {
    #[serde(with = "time::serde::iso8601")]
    pub bought_at: OffsetDateTime,
    pub bought_for: f64,
    pub quantity_bought: f64,
    pub value_delta: f64,
}

#[derive(Serialize)]
pub struct OwnedAsset {
    pub id: i64,
    pub name: String,
    pub unit_value: f64,
    pub value_delta: f64,
    pub quantity_owned: f64,
    pub purchase_history: Json<Vec<PurchaseHistory>>
}