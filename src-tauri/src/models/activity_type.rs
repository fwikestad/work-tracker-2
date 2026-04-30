use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActivityType {
    pub id: String,
    pub name: String,
    pub sort_order: i64,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateActivityTypeParams {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateActivityTypeParams {
    pub name: Option<String>,
    pub sort_order: Option<i64>,
}
