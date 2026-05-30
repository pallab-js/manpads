import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { AppStateData, TelemetryFrame, AckFrame } from './types';

/**
 * Fetch the complete application status and audit logs from Rust.
 */
export async function getAppState(): Promise<AppStateData> {
  try {
    return await invoke<AppStateData>('get_app_state');
  } catch (error) {
    console.error('Failed to get app state:', error);
    throw error;
  }
}

/**
 * Dispatch an operator command (Arm, Disarm, Fire, Estop) to the Pi.
 */
export async function sendOperatorCommand(
  action: 'arm' | 'disarm' | 'fire' | 'estop',
  authToken: string = 'DEMO-OPERATOR-TOKEN-2026'
): Promise<number> {
  try {
    return await invoke<number>('send_operator_command', {
      actionStr: action,
      authToken,
    });
  } catch (error) {
    console.error(`Failed to send operator command (${action}):`, error);
    throw error;
  }
}

/**
 * Register a listener for real-time 10Hz telemetry frames from the Pi.
 */
export async function onTelemetryUpdate(
  callback: (frame: TelemetryFrame) => void
): Promise<() => void> {
  const unlisten = await listen<TelemetryFrame>('telemetry-update', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

/**
 * Register a listener for heartbeats and connection change state transitions.
 */
export async function onConnectionStatusChange(
  callback: (isConnected: boolean) => void
): Promise<() => void> {
  const unlisten = await listen<boolean>('connection-status', (event) => {
    callback(event.payload);
  });
  return unlisten;
}

/**
 * Register a listener for command ACK packets.
 */
export async function onAckUpdate(
  callback: (ack: AckFrame) => void
): Promise<() => void> {
  const unlisten = await listen<AckFrame>('ack-update', (event) => {
    callback(event.payload);
  });
  return unlisten;
}
