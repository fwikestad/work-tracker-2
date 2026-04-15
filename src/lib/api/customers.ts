import { invoke } from '@tauri-apps/api/core';
import type { Customer, CreateCustomerParams, UpdateCustomerParams } from '../types';

/** Creates a new customer in the local database. */
export const createCustomer = (params: CreateCustomerParams) =>
  invoke<Customer>('create_customer', { params });

/** Fetches all customers, optionally including archived ones. */
export const listCustomers = (includeArchived = false) =>
  invoke<Customer[]>('list_customers', { includeArchived });

/** Updates an existing customer's details. */
export const updateCustomer = (id: string, params: UpdateCustomerParams) =>
  invoke<Customer>('update_customer', { id, params });

/** Archives a customer (soft delete - hides from active lists but preserves data). */
export const archiveCustomer = (id: string) =>
  invoke<void>('archive_customer', { id });

/** Restores an archived customer to active status. */
export const unarchiveCustomer = (id: string) =>
  invoke<void>('unarchive_customer', { id });
