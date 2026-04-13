import { invoke } from '@tauri-apps/api/core';
import type { Customer, CreateCustomerParams, UpdateCustomerParams } from '../types';

export const createCustomer = (params: CreateCustomerParams) =>
  invoke<Customer>('create_customer', { params });

export const listCustomers = (includeArchived = false) =>
  invoke<Customer[]>('list_customers', { includeArchived });

export const updateCustomer = (id: string, params: UpdateCustomerParams) =>
  invoke<Customer>('update_customer', { id, params });

export const archiveCustomer = (id: string) =>
  invoke<void>('archive_customer', { id });

export const unarchiveCustomer = (id: string) =>
  invoke<void>('unarchive_customer', { id });
