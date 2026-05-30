# Network Protocol Spec: MANPADS Control Link

This document defines the wire protocol for the link between the Laptop Tauri Client and the Raspberry Pi Edge Controller.

## Transport layer
- **Protocol**: UDP (User Datagram Protocol) for ultra-low latency.
- **Ports**: Configurable via `MANPADS_LISTEN_ADDR` (Pi listener) and `MANPADS_DESKTOP_ADDR` (Laptop listener). Defaults: `8080` / `8081`.
- **Format**: JSON (UTF-8 encoded). CBOR transport available via `cbor-transport` feature flag (future milestone).

## 1. Frame Headers & Security
All frames MUST contain:
- `protocolVersion` (u8): Must be `1`. Frames with mismatched version are rejected.
- `seq` (u64): Monotonically increasing sequence number.
- `timestampMs` (u64): Epoch timestamp in milliseconds.
- `hmac` (string): Hex-encoded SHA-256 HMAC signature computed over the serialized payload (with `hmac` set to empty string) using the pre-shared key from `MANPADS_HMAC_SECRET` environment variable.

### Security Requirements
- `MANPADS_HMAC_SECRET` and `MANPADS_OPERATOR_TOKEN` are required environment variables — no defaults exist in code.
- HMAC key is loaded once at startup via `OnceLock`, not per-frame.
- Timestamp freshness window: ±5 seconds (`MAX_CLOCK_SKEW_MS`).
- Sequence numbers must be strictly increasing (E-stop bypasses this check).
- Operator token is validated server-side; never exposed over IPC to frontend.

### Fault Mask Bit Definitions
| Bit | Constant             | Description                |
|-----|----------------------|----------------------------|
| 0   | `WATCHDOG_TIMEOUT`   | Watchdog disarm triggered  |
| 1   | `GPIO_INTERLOCK_ERR` | GPIO or software E-stop    |
| 2   | `THERMAL_CRITICAL`   | Temperature > 75°C         |
| 3   | `BATTERY_LOW`        | Battery voltage < 10.5V    |
| 4   | `GPS_STALE`          | GPS signal lost             |

---

## 2. Command Schema (Laptop → Pi)
Commands are sent by the laptop to change states or fire triggers.

```json
{
  "protocolVersion": 1,
  "seq": 104,
  "timestampMs": 1717083040000,
  "action": "Arm",
  "authToken": "<operator-token>",
  "hmac": "a1b2c3d4..."
}
```

**Fields:**
- `action`: `"Arm"` | `"Disarm"` | `"Fire"` | `"Estop"`
- `authToken`: Operator session token validated against `MANPADS_OPERATOR_TOKEN`.

---

## 3. Telemetry Frame (Pi → Laptop)
Sent periodically by the Pi at a configurable interval (default 100ms = 10Hz).

```json
{
  "protocolVersion": 1,
  "seq": 5892,
  "timestampMs": 1717083040100,
  "systemState": "Safe",
  "batteryVoltage": 12.4,
  "temperature": 42.5,
  "gpsLatitude": 37.774929,
  "gpsLongitude": -122.419416,
  "faultMask": 0,
  "hmac": "e5f6a7b8..."
}
```

**systemState values:** `"Off"` | `"Safe"` | `"Armed"` | `"Active"` | `"Emergency"`

---

## 4. Acknowledgment (ACK) Frame (Pi → Laptop)
Sent in response to every valid incoming Command.

```json
{
  "protocolVersion": 1,
  "seq": 104,
  "timestampMs": 1717083040010,
  "commandSeq": 104,
  "success": true,
  "errorMsg": "",
  "hmac": "9f8e7d6c..."
}
```

### Critical Command Retry
Fire and Estop commands are retried up to 3 times at 200ms intervals if no ACK is received. After exhausting retries, a `"command-ack-timeout"` event is emitted to the frontend.

---

## State Machine

```
        ┌─────────┐
        │   OFF   │
        └────┬────┘
             │ (init)
             ▼
        ┌─────────┐
   ┌───►│  SAFE   │◄─────────┐
   │    └────┬────┘          │
   │         │ Arm           │ Disarm (any state → SAFE)
   │         ▼               │
   │    ┌─────────┐          │
   │    │ ARMED   │──────────┘
   │    └────┬────┘
   │         │ Fire
   │         ▼
   │    ┌─────────┐
   │    │ ACTIVE  │
   │    └────┬────┘
   │         │
   │         │ (automatic)
   │         ▼
   │    ┌────────────┐
   └────│ EMERGENCY  │  ← E-stop (any state → EMERGENCY)
        └────────────┘
```
