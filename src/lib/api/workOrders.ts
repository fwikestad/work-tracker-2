import { invoke } from '@tauri-apps/api/core';
import type { WorkOrder, CreateWorkOrderParams, UpdateWorkOrderParams } from '../types';

export const createWorkOrder = (params: CreateWorkOrderParams) =>
  invoke<WorkOrder>('create_work_order', { params });

export const listWorkOrders = (customerId?: string) =>
  invoke<WorkOrder[]>('list_work_orders', { customer_id: customerId });

export const updateWorkOrder = (id: string, params: UpdateWorkOrderParams) =>
  invoke<WorkOrder>('update_work_order', { id, params });

export const archiveWorkOrder = (id: string) =>
  invoke<void>('archive_work_order', { id });

export const toggleFavorite = (workOrderId: string) =>
  invoke<WorkOrder>('toggle_favorite', { work_order_id: workOrderId });
