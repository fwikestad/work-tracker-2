-- Phase 4a: ServiceNow integration
-- Adds optional ServiceNow task ID to work orders for export mapping

ALTER TABLE work_orders ADD COLUMN servicenow_task_id TEXT;
