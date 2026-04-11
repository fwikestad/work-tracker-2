use rusqlite::{Connection, params};
use crate::models::{session::*, work_order::WorkOrder, error::AppError};

pub fn get_daily_summary(conn: &Connection, date: &str) -> Result<DailySummary, AppError> {
    // Get summary entries (aggregated by customer and work order)
    let mut stmt = conn.prepare("
        SELECT 
            c.id,
            c.name,
            c.color,
            wo.id,
            wo.name,
            SUM(COALESCE(ts.duration_override, ts.duration_seconds) - ts.total_paused_seconds),
            COUNT(ts.id)
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE date(ts.start_time) = date(?)
          AND ts.end_time IS NOT NULL
        GROUP BY c.id, wo.id
        ORDER BY c.name, wo.name
    ")?;
    
    let entries: Result<Vec<SummaryEntry>, _> = stmt.query_map(params![date], |row| {
        Ok(SummaryEntry {
            customer_id: row.get(0)?,
            customer_name: row.get(1)?,
            customer_color: row.get(2)?,
            work_order_id: row.get(3)?,
            work_order_name: row.get(4)?,
            total_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
            session_count: row.get(6)?,
        })
    })?.collect();
    
    let entries = entries?;
    
    // Calculate total
    let total_seconds: i64 = entries.iter().map(|e| e.total_seconds).sum();
    
    // Get all sessions for the day
    let mut stmt = conn.prepare("
        SELECT 
            ts.id,
            ts.work_order_id,
            wo.name,
            c.name,
            c.color,
            ts.start_time,
            ts.end_time,
            ts.duration_seconds,
            ts.duration_override,
            COALESCE(ts.duration_override, ts.duration_seconds),
            ts.activity_type,
            ts.notes,
            ts.created_at,
            ts.updated_at
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE date(ts.start_time) = date(?)
        ORDER BY ts.start_time
    ")?;
    
    let sessions: Result<Vec<_>, _> = stmt.query_map(params![date], |row| {
        Ok(crate::models::session::Session {
            id: row.get(0)?,
            work_order_id: row.get(1)?,
            work_order_name: row.get(2)?,
            customer_name: row.get(3)?,
            customer_color: row.get(4)?,
            start_time: row.get(5)?,
            end_time: row.get(6)?,
            duration_seconds: row.get(7)?,
            duration_override: row.get(8)?,
            effective_duration: row.get(9)?,
            activity_type: row.get(10)?,
            notes: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?.collect();
    
    Ok(DailySummary {
        date: date.to_string(),
        total_seconds,
        entries,
        sessions: sessions?,
    })
}

pub fn get_recent_work_orders(conn: &Connection, limit: i64) -> Result<Vec<WorkOrder>, AppError> {
    let mut stmt = conn.prepare("
        SELECT 
            wo.id,
            wo.customer_id,
            c.name,
            c.color,
            wo.name,
            wo.code,
            wo.description,
            wo.status,
            wo.is_favorite,
            wo.created_at,
            wo.updated_at,
            wo.archived_at
        FROM recent_work_orders r
        JOIN work_orders wo ON r.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE wo.archived_at IS NULL
        ORDER BY wo.is_favorite DESC, r.last_used_at DESC
        LIMIT ?
    ")?;
    
    let work_orders: Result<Vec<_>, _> = stmt.query_map(params![limit], |row| {
        Ok(WorkOrder {
            id: row.get(0)?,
            customer_id: row.get(1)?,
            customer_name: row.get(2)?,
            customer_color: row.get(3)?,
            name: row.get(4)?,
            code: row.get(5)?,
            description: row.get(6)?,
            status: row.get(7)?,
            is_favorite: row.get::<_, i64>(8)? == 1,
            created_at: row.get(9)?,
            updated_at: row.get(10)?,
            archived_at: row.get(11)?,
        })
    })?.collect();
    
    work_orders.map_err(AppError::Database)
}

pub fn export_csv(conn: &Connection, start_date: &str, end_date: &str) -> Result<String, AppError> {
    let mut stmt = conn.prepare("
        SELECT 
            date(ts.start_time),
            c.name,
            wo.name,
            ts.start_time,
            ts.end_time,
            COALESCE(ts.duration_override, ts.duration_seconds),
            ts.activity_type,
            ts.notes
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE date(ts.start_time) >= date(?)
          AND date(ts.start_time) <= date(?)
          AND ts.end_time IS NOT NULL
        ORDER BY ts.start_time
    ")?;
    
    let mut csv = String::from("Date,Customer,Work Order,Start Time,End Time,Duration (minutes),Activity Type,Notes\n");
    
    let rows = stmt.query_map(params![start_date, end_date], |row| {
        let date: String = row.get(0)?;
        let customer: String = row.get(1)?;
        let work_order: String = row.get(2)?;
        let start_time: String = row.get(3)?;
        let end_time: Option<String> = row.get(4)?;
        let duration_seconds: Option<i64> = row.get(5)?;
        let activity_type: Option<String> = row.get(6)?;
        let notes: Option<String> = row.get(7)?;
        
        let duration_minutes = duration_seconds.map(|s| s / 60).unwrap_or(0);
        
        Ok(format!(
            "{},{},{},{},{},{},{},{}\n",
            escape_csv(&date),
            escape_csv(&customer),
            escape_csv(&work_order),
            escape_csv(&start_time),
            escape_csv(&end_time.unwrap_or_default()),
            duration_minutes,
            escape_csv(&activity_type.unwrap_or_default()),
            escape_csv(&notes.unwrap_or_default())
        ))
    })?;
    
    for row in rows {
        csv.push_str(&row?);
    }
    
    Ok(csv)
}

pub fn get_report(conn: &Connection, start_date: &str, end_date: &str) -> Result<ReportData, AppError> {
    // Get aggregated entries (grouped by customer and work order)
    let mut stmt = conn.prepare("
        SELECT 
            c.id,
            c.name,
            c.color,
            wo.id,
            wo.name,
            SUM(COALESCE(ts.duration_override, ts.duration_seconds) - ts.total_paused_seconds),
            COUNT(ts.id)
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE date(ts.start_time) >= date(?)
          AND date(ts.start_time) <= date(?)
          AND ts.end_time IS NOT NULL
        GROUP BY c.id, wo.id
        ORDER BY SUM(COALESCE(ts.duration_override, ts.duration_seconds) - ts.total_paused_seconds) DESC
    ")?;
    
    let entries: Result<Vec<ReportEntry>, _> = stmt.query_map(params![start_date, end_date], |row| {
        Ok(ReportEntry {
            customer_id: row.get(0)?,
            customer_name: row.get(1)?,
            customer_color: row.get(2)?,
            work_order_id: row.get(3)?,
            work_order_name: row.get(4)?,
            total_seconds: row.get::<_, Option<i64>>(5)?.unwrap_or(0),
            session_count: row.get(6)?,
        })
    })?.collect();
    
    let entries = entries?;
    
    // Calculate total
    let total_seconds: i64 = entries.iter().map(|e| e.total_seconds).sum();
    
    // Get all sessions for the date range
    let mut stmt = conn.prepare("
        SELECT 
            ts.id,
            ts.work_order_id,
            wo.name,
            c.name,
            c.color,
            ts.start_time,
            ts.end_time,
            ts.duration_seconds,
            ts.duration_override,
            COALESCE(ts.duration_override, ts.duration_seconds) - ts.total_paused_seconds,
            ts.activity_type,
            ts.notes,
            ts.created_at,
            ts.updated_at
        FROM time_sessions ts
        JOIN work_orders wo ON ts.work_order_id = wo.id
        JOIN customers c ON wo.customer_id = c.id
        WHERE date(ts.start_time) >= date(?)
          AND date(ts.start_time) <= date(?)
          AND ts.end_time IS NOT NULL
        ORDER BY ts.start_time
    ")?;
    
    let sessions: Result<Vec<_>, _> = stmt.query_map(params![start_date, end_date], |row| {
        Ok(Session {
            id: row.get(0)?,
            work_order_id: row.get(1)?,
            work_order_name: row.get(2)?,
            customer_name: row.get(3)?,
            customer_color: row.get(4)?,
            start_time: row.get(5)?,
            end_time: row.get(6)?,
            duration_seconds: row.get(7)?,
            duration_override: row.get(8)?,
            effective_duration: row.get(9)?,
            activity_type: row.get(10)?,
            notes: row.get(11)?,
            created_at: row.get(12)?,
            updated_at: row.get(13)?,
        })
    })?.collect();
    
    Ok(ReportData {
        start_date: start_date.to_string(),
        end_date: end_date.to_string(),
        total_seconds,
        entries,
        sessions: sessions?,
    })
}

fn escape_csv(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}
