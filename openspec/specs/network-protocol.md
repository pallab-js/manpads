# Network Protocol Spec: MANPADS Control Link

This document defines the wire protocol for the link between the Laptop Tauri Client and the Raspberry Pi Edge Controller.

## Transport layer
- **Protocol**: UDP (User Datagram Protocol) for ultra-low latency, or MQTT as backup. We implement direct UDP socket streaming.
- **Port**: `8080` (Pi listener for commands) & `8081` (Laptop listener for telemetry).
- **Format**: CBOR (Concise Binary Object Representation) for compact binary serialization, with a JSON fallback.

## 1. Frame Headers & Security
All packages MUST contain:
- `seq` (u64): Monotonically increasing sequence number.
- `timestamp_ms` (u64): Epoch timestamp in milliseconds.
- `hmac` (string): Hex-encoded SHA-256 HMAC signature computed over the serial data using the pre-shared key `"manpads-td-secret-key"`. (TD-ONLY stub validation).

---

## 2. Command Schema (Laptop → Pi)
Commands are sent by the laptop to change states or fire triggers.

```json
{
  "seq": 104,
  "timestampMs": 1717083040000,
  "action": "Arm", // "Arm" | "Disarm" | "Fire" | "Estop"
  "authToken": "DEMO-OPERATOR-TOKEN-2026",
  "hmac": "a1b2c3d4..."
}
```

---

## 3. Telemetry Frame (Pi → Laptop)
Sent periodically by the Pi at 10Hz.

```json
{
  "seq": 5892,
  "timestampMs": 1717083040100,
  "systemState": "Safe", // "Off" | "Safe" | "Armed" | "Active" | "Emergency"
  "batteryVoltage": 12.4, // Volts
  "temperature": 42.5, // Celsius
  "gpsLatitude": 37.774929,
  "gpsLongitude": -122.419416,
  "faultMask": 0, // Bitmask: 0 = OK, 1 = WatchdogTimeout, 2 = GPIOError, 4 = TempHigh
  "hmac": "e5f6a7b8..."
}
```

---

## 4. Acknowledgment (ACK) Frame (Pi → Laptop)
Sent in response to every valid incoming Command.

```json
{
  "seq": 104,
  "timestampMs": 1717083040010,
  "commandSeq": 104,
  "success": true,
  "errorMsg": "",
  "hmac": "9f8e7d6c..."
}
```

## Security & Verification (TD-ONLY)
For the technology demonstration, the HMAC verification will check if `hmac` is provided and compute the SHA256 of the payload appended with the demo key. If a verification failure occurs, the receiver must discard the packet and log a warning.
