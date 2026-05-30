import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppStateData, TelemetryFrame, AckFrame } from './types';

export async function getAppState(): Promise<AppStateData> {
  try {
    return await invoke<AppStateData>('get_app_state');
  } catch (error) {
    console.error('Failed to get app state:', error);
    throw error;
  }
}

export async function sendOperatorCommand(
  action: 'arm' | 'disarm' | 'fire' | 'estop'
): Promise<number> {
  try {
    return await invoke<number>('send_operator_command', {
      actionStr: action,
    });
  } catch (error) {
    console.error(`Failed to send operator command (${action}):`, error);
    throw error;
  }
}

export async function updateSettings(piIp: string, piPort: number, localPort: number): Promise<void> {
  return invoke<void>('update_settings', { piIp, piPort, localPort });
}

export async function exportAuditLog(): Promise<string> {
  return invoke<string>('export_audit_log');
}

export async function onTelemetryUpdate(
  callback: (frame: TelemetryFrame) => void
): Promise<() => void> {
  const unlisten = await listen<TelemetryFrame>('telemetry-update', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

export async function onConnectionStatusChange(
  callback: (isConnected: boolean) => void
): Promise<() => void> {
  const unlisten = await listen<boolean>('connection-status', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

export async function onAckUpdate(
  callback: (ack: AckFrame) => void
): Promise<() => void> {
  const unlisten = await listen<AckFrame>('ack-update', (event) => {
    callback(event.payload);
  });
  return unlisten;
}
