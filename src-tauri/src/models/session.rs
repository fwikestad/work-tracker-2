use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub work_order_id: String,
    pub work_order_name: Option<String>,
    pub customer_name: Option<String>,
    pub customer_color: Option<String>,
    pub start_time: String,
    pub end_time: Option<String>,
    pub duration_seconds: Option<i64>,
    pub duration_override: Option<i64>,
    pub effective_duration: Option<i64>,
    pub activity_type: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ActiveSession {
    pub session_id: String,
    pub work_order_id: String,
    pub work_order_name: String,
    pub customer_name: String,
    pub customer_color: Option<String>,
    pub started_at: String,
    pub elapsed_seconds: i64,
    pub is_paused: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OrphanSession {
    pub session_id: String,
    pub work_order_name: String,
    pub customer_name: String,
    pub started_at: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionParams {
    pub duration_override: Option<i64>,
    pub activity_type: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickAddParams {
    pub customer_name: Option<String>,
    pub customer_id: Option<String>,
    pub work_order_name: String,
    pub work_order_code: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickAddResult {
    pub customer: super::customer::Customer,
    pub work_order: super::work_order::WorkOrder,
    pub session: Session,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySummary {
    pub date: String,
    pub total_seconds: i64,
    pub entries: Vec<SummaryEntry>,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SummaryEntry {
    pub customer_id: String,
    pub customer_name: String,
    pub customer_color: Option<String>,
    pub work_order_id: String,
    pub work_order_name: String,
    pub total_seconds: i64,
    pub session_count: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportData {
    pub start_date: String,
    pub end_date: String,
    pub total_seconds: i64,
    pub entries: Vec<ReportEntry>,
    pub sessions: Vec<Session>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportEntry {
    pub customer_id: String,
    pub customer_name: String,
    pub customer_color: Option<String>,
    pub work_order_id: String,
    pub work_order_name: String,
    pub total_seconds: i64,
    pub session_count: i64,
}
