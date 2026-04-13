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

#[tauri::command]
pub fn export_csv(state: State<AppState>, start_date: String, end_date: String) -> Result<String, AppError> {
    let conn = get_conn(&state)?;
    summary_service::export_csv(&conn, &start_date, &end_date)
}

#[tauri::command]
pub fn get_report(state: State<AppState>, start_date: String, end_date: String) -> Result<ReportData, AppError> {
    let conn = get_conn(&state)?;
    summary_service::get_report(&conn, &start_date, &end_date)
}
