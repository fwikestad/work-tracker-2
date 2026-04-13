import { invoke } from '@tauri-apps/api/core';
import type { WorkOrder, CreateWorkOrderParams, UpdateWorkOrderParams } from '../types';

export const createWorkOrder = (params: CreateWorkOrderParams) =>
  invoke<WorkOrder>('create_work_order', { params });

export const listWorkOrders = (customerId?: string, favoritesOnly?: boolean) =>
  invoke<WorkOrder[]>('list_work_orders', { customerId, favoritesOnly });

export const updateWorkOrder = (id: string, params: UpdateWorkOrderParams) =>
  invoke<WorkOrder>('update_work_order', { id, params });

export const archiveWorkOrder = (id: string) =>
  invoke<void>('archive_work_order', { id });

export const toggleFavorite = (workOrderId: string) =>
  invoke<WorkOrder>('toggle_favorite', { workOrderId });
