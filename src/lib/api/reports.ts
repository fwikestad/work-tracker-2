import { invoke } from '@tauri-apps/api/core';
import type { DailySummary, WorkOrder, ReportData } from '../types';

export const getDailySummary = (date: string) =>
  invoke<DailySummary>('get_daily_summary', { date });

export const getRecentWorkOrders = (limit?: number) =>
  invoke<WorkOrder[]>('get_recent_work_orders', { limit });

export const exportCsv = (startDate: string, endDate: string) =>
  invoke<string>('export_csv', { start_date: startDate, end_date: endDate });

export const getReport = (startDate: string, endDate: string) =>
  invoke<ReportData>('get_report', { start_date: startDate, end_date: endDate });
