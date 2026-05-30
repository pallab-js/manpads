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
  latencyMs: number;
  lastTelemetry: TelemetryFrame | null;
  commandSeq: number;
  auditLog: string[];
  piPort: number;
  localPort: number;
  operatorToken: string;
}
