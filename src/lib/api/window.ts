import { invoke } from '@tauri-apps/api/core';

export async function toggleWidgetMode(enable: boolean): Promise<boolean> {
  return await invoke<boolean>('toggle_widget_mode', { enable });
}
