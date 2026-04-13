use tauri::State;
use crate::{AppState, db::get_conn, models::{session::{DailySummary, ReportData}, work_order::WorkOrder, error::AppError}, services::summary_service};

#[tauri::command]
pub fn get_daily_summary(state: State<AppState>, date: String) -> Result<DailySummary, AppError> {
    let conn = get_conn(&state)?;
    summary_service::get_daily_summary(&conn, &date)
}

#[tauri::command]
pub fn get_recent_work_orders(state: State<AppState>, limit: Option<i64>) -> Result<Vec<WorkOrder>, AppError> {
    let conn = get_conn(&state)?;
    summary_service::get_recent_work_orders(&conn, limit.unwrap_or(10))
}

/// Export time sessions as CSV.
///
/// `export_format` selects the output schema:
/// - `"standard"` (default) — existing Date/Customer/Work Order/... columns, unchanged
/// - `"servicenow"` — ServiceNow Import Set columns: opened_at, closed_at, duration_hours, ...
///
/// The `round_to_half_hour` setting is read from the database; when enabled, session duration
/// is calculated from the floor of start_time to the nearest 30-minute boundary.
#[tauri::command]
pub fn export_csv(
    state: State<AppState>,
    start_date: String,
    end_date: String,
    export_format: Option<String>,
) -> Result<String, AppError> {
    let conn = get_conn(&state)?;
    let round = summary_service::get_round_to_half_hour(&conn)?;
    match export_format.as_deref() {
        Some("servicenow") => summary_service::export_servicenow_csv(&conn, &start_date, &end_date, round),
        _ => summary_service::export_csv(&conn, &start_date, &end_date, round),
    }
}

#[tauri::command]
pub fn get_report(state: State<AppState>, start_date: String, end_date: String) -> Result<ReportData, AppError> {
    let conn = get_conn(&state)?;
    summary_service::get_report(&conn, &start_date, &end_date)
}
