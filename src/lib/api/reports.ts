import { invoke } from '@tauri-apps/api/core';
import type { DailySummary, WorkOrder, ReportData } from '../types';

/** Fetches a summary of time tracked on a specific date (total hours, breakdowns by customer/project). */
export const getDailySummary = (date: string) =>
  invoke<DailySummary>('get_daily_summary', { date });

/** Fetches the most recently tracked work orders (default limit: 10). */
export const getRecentWorkOrders = (limit?: number) =>
  invoke<WorkOrder[]>('get_recent_work_orders', { limit });

/** Exports sessions within a date range as CSV text. */
export const exportCsv = (startDate: string, endDate: string) =>
  invoke<string>('export_csv', { startDate, endDate });

/** Exports sessions within a date range as ServiceNow-formatted CSV text. */
export const exportServiceNow = (startDate: string, endDate: string) =>
  invoke<string>('export_servicenow', { startDate, endDate });

/** Fetches a report summarizing time tracked within a date range. */
export const getReport = (startDate: string, endDate: string) =>
  invoke<ReportData>('get_report', { startDate, endDate });
