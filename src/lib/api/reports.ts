import { invoke } from '@tauri-apps/api/core';
import type { DailySummary, WorkOrder, ReportData } from '../types';

export const getDailySummary = (date: string) =>
  invoke<DailySummary>('get_daily_summary', { date });

export const getRecentWorkOrders = (limit?: number) =>
  invoke<WorkOrder[]>('get_recent_work_orders', { limit });

export type ExportFormat = 'standard' | 'servicenow';

export const exportCsv = (startDate: string, endDate: string, exportFormat: ExportFormat = 'standard') =>
  invoke<string>('export_csv', { startDate, endDate, exportFormat });

export const getReport = (startDate: string, endDate: string) =>
  invoke<ReportData>('get_report', { startDate, endDate });
