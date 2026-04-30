use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrder {
    pub id: String,
    pub customer_id: String,
    pub customer_name: Option<String>,
    pub customer_color: Option<String>,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub is_favorite: bool,
    pub created_at: String,
    pub updated_at: String,
    pub archived_at: Option<String>,
    pub servicenow_task_id: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkOrderParams {
    pub customer_id: String,
    pub name: String,
    pub code: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWorkOrderParams {
    pub name: Option<String>,
    pub code: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub servicenow_task_id: Option<String>,
}
