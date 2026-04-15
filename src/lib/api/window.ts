import { invoke } from '@tauri-apps/api/core';

/** Toggles widget mode (compact always-on-top view). Returns the new state. */
export async function toggleWidgetMode(enable: boolean): Promise<boolean> {
  return await invoke<boolean>('toggle_widget_mode', { enable });
}

/** Resizes the widget window to specific dimensions. */
export async function resizeWidget(width: number, height: number): Promise<void> {
  await invoke<void>('resize_widget', { width, height });
}
