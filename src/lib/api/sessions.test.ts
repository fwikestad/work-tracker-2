/**
 * Sessions API Tests
 *
 * Tests for the sessions API functions, focusing on new continue functionality
 * for Issue #49.
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import { getLastStoppedWorkOrder, continueLastSession } from './sessions';

// Mock Tauri invoke
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));

beforeEach(() => {
  vi.clearAllMocks();
});

describe('getLastStoppedWorkOrder', () => {
  /**
   * TC-API-CONTINUE-01
   * getLastStoppedWorkOrder should invoke the get_last_stopped_work_order command
   * and return the work order ID when a previous session exists.
   */
  it('should invoke get_last_stopped_work_order command', async () => {
    const mockWorkOrderId = 'wo-123';
    vi.mocked(invoke).mockResolvedValueOnce(mockWorkOrderId);
    
    const result = await getLastStoppedWorkOrder();
    
    expect(invoke).toHaveBeenCalledWith('get_last_stopped_work_order');
    expect(result).toBe(mockWorkOrderId);
  });

  /**
   * TC-API-CONTINUE-02
   * getLastStoppedWorkOrder should return null when no previous session exists.
   */
  it('should return null when no previous session', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);
    
    const result = await getLastStoppedWorkOrder();
    
    expect(result).toBeNull();
  });

  /**
   * TC-API-CONTINUE-03
   * getLastStoppedWorkOrder should handle errors from the backend gracefully.
   */
  it('should throw error when backend fails', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Database error'));
    
    await expect(getLastStoppedWorkOrder()).rejects.toThrow('Database error');
  });
});

describe('continueLastSession', () => {
  /**
   * TC-API-CONTINUE-04
   * continueLastSession should call getLastStoppedWorkOrder and start a session
   * with the returned work order ID.
   */
  it('should invoke get_last_stopped_work_order and start session', async () => {
    const mockWorkOrderId = 'wo-456';
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockWorkOrderId) // getLastStoppedWorkOrder
      .mockResolvedValueOnce({ id: 'session-123' }); // startSession
    
    await continueLastSession();
    
    expect(invoke).toHaveBeenNthCalledWith(1, 'get_last_stopped_work_order');
    expect(invoke).toHaveBeenNthCalledWith(2, 'start_session', { workOrderId: mockWorkOrderId });
  });

  /**
   * TC-API-CONTINUE-05
   * continueLastSession should do nothing when no previous session exists.
   */
  it('should do nothing when no previous session exists', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null); // getLastStoppedWorkOrder returns null
    
    await continueLastSession();
    
    expect(invoke).toHaveBeenCalledTimes(1); // Only getLastStoppedWorkOrder, no startSession
    expect(invoke).toHaveBeenCalledWith('get_last_stopped_work_order');
  });

  /**
   * TC-API-CONTINUE-06
   * continueLastSession should propagate errors from getLastStoppedWorkOrder.
   */
  it('should propagate error from getLastStoppedWorkOrder', async () => {
    vi.mocked(invoke).mockRejectedValueOnce(new Error('Backend unavailable'));
    
    await expect(continueLastSession()).rejects.toThrow('Backend unavailable');
  });

  /**
   * TC-API-CONTINUE-07
   * continueLastSession should propagate errors from startSession.
   */
  it('should propagate error from startSession', async () => {
    const mockWorkOrderId = 'wo-789';
    vi.mocked(invoke)
      .mockResolvedValueOnce(mockWorkOrderId) // getLastStoppedWorkOrder succeeds
      .mockRejectedValueOnce(new Error('Work order not found')); // startSession fails
    
    await expect(continueLastSession()).rejects.toThrow('Work order not found');
  });
});
