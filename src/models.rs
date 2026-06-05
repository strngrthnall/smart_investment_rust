use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Asset {
    pub(crate) id: i64,
    pub(crate) name: String,
    pub(crate) unit_value: f64
}
