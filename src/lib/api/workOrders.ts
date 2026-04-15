import { invoke } from '@tauri-apps/api/core';
import type { WorkOrder, CreateWorkOrderParams, UpdateWorkOrderParams } from '../types';

/** Creates a new work order under a customer. */
export const createWorkOrder = (params: CreateWorkOrderParams) =>
  invoke<WorkOrder>('create_work_order', { params });

/** Fetches work orders with optional filters (by customer, favorites only, include archived). */
export const listWorkOrders = (customerId?: string, favoritesOnly?: boolean, includeArchived?: boolean) =>
  invoke<WorkOrder[]>('list_work_orders', { customerId, favoritesOnly, includeArchived });

/** Updates an existing work order's details. */
export const updateWorkOrder = (id: string, params: UpdateWorkOrderParams) =>
  invoke<WorkOrder>('update_work_order', { id, params });

/** Archives a work order (soft delete - hides but preserves data). */
export const archiveWorkOrder = (id: string) =>
  invoke<void>('archive_work_order', { id });

/** Restores an archived work order to active status. */
export const unarchiveWorkOrder = (id: string) =>
  invoke<void>('unarchive_work_order', { id });

/** Toggles the favorite status of a work order (for quick access). */
export const toggleFavorite = (workOrderId: string) =>
  invoke<WorkOrder>('toggle_favorite', { workOrderId });
