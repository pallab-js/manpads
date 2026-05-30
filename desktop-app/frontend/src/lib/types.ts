export type SystemState = 'off' | 'safe' | 'armed' | 'active' | 'emergency';

export interface TelemetryFrame {
  seq: number;
  timestampMs: number;
  systemState: SystemState;
  batteryVoltage: number;
  temperature: number;
  gpsLatitude: number;
  gpsLongitude: number;
  faultMask: number;
  hmac: string;
}

export interface AckFrame {
  seq: number;
  timestampMs: number;
  commandSeq: number;
  success: boolean;
  errorMsg: string;
  hmac: string;
}

export interface AppStateData {
  isConnected: boolean;
  piIp: string;
  piPort: number;
  localPort: number;
  latencyMs: number;
  lastTelemetry: TelemetryFrame | null;
  auditLog: string[];
}

export const FaultFlags = {
  WATCHDOG_TIMEOUT: 1 << 0,
  GPIO_INTERLOCK_ERR: 1 << 1,
  THERMAL_CRITICAL: 1 << 2,
  BATTERY_LOW: 1 << 3,
  GPS_STALE: 1 << 4,
} as const;
