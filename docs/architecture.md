# MANPADS Architecture

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Desktop Application                         │
│  ┌──────────┐  ┌──────────────────┐  ┌──────────────────────────┐  │
│  │ SvelteKit│  │  Tauri Backend   │  │     UDP Network Layer    │  │
│  │ Frontend │◄─┤  (Rust)          │◄─┤                          │  │
│  │          │  │  AppState        │  │  PiClient (send/recv)    │─┼──┐
│  │ - Telemetry│  │  AuditWriter    │  │  Critical Retry Logic   │  │  │
│  │ - Commands │  │  Settings       │  │  Watchdog Checker       │  │  │
│  │ - Audit Log│  │  Error Types    │  │                          │  │  │
│  └──────────┘  └──────────────────┘  └──────────────────────────┘  │  │
└─────────────────────────────────────────────────────────────────────┘  │
                                                                         │
                                    UDP (JSON)                           │
                                    Port 8080/8081                       │
                                                                         │
┌────────────────────────────────────────────────────────────────────┐   │
│                      Pi Controller (Edge)                          │◄──┘
│  ┌────────────────┐  ┌──────────────┐  ┌────────────────────────┐  │
│  │  Command       │  │  Telemetry   │  │  Hardware Abstraction  │  │
│  │  Receiver      │  │  Sender      │  │  (trait HardwareBackend│  │
│  │  (Task A)      │  │  (Task B)    │  │   + Real/Simulated)   │  │
│  │                │  │              │  │                        │  │
│  │ - Seq/TS check │  │ - 10Hz loop │  │ - Sensors (battery,    │  │
│  │ - HMAC verify  │  │ - Fault mask│  │   temp, GPS)           │  │
│  │ - State trans. │  │ - HMAC sign │  │ - Actuators (relays)   │  │
│  │ - ACK response │  │              │  │ - Interlock (E-stop)   │  │
│  └───────┬────────┘  └──────┬───────┘  └────────────────────────┘  │
│          │                  │                                       │
│          └────────┬─────────┘                                       │
│                   │                                                 │
│          ┌────────▼────────┐                                        │
│          │    Watchdog     │                                        │
│          │    (Task C)     │                                        │
│          │  1500ms timer   │                                        │
│          └─────────────────┘                                        │
└─────────────────────────────────────────────────────────────────────┘
```

## Component Responsibilities

### Desktop Application
- **SvelteKit Frontend**: Operator console UI with real-time telemetry display, command console, audit feed, and emergency stop controls
- **Tauri Backend**:
  - `AppState`: Thread-safe shared state (view layer for IPC, internal layer for secrets)
  - `PiClient`: UDP network client for sending commands and receiving telemetry/ACKs
  - `AuditWriter`: Persistent audit log to disk
  - `Settings`: Connection settings persistence
  - `Commands`: Tauri IPC command handlers

### Pi Controller
- **Command Receiver (Task A)**: Validates incoming commands (HMAC, auth, seq, timestamp), applies state transitions, returns ACKs
- **Telemetry Sender (Task B)**: Broadcasts system telemetry at 10Hz with HMAC signing
- **Watchdog (Task C)**: Monitors connection health, disarms on timeout, reacts to E-stop
- **Hardware Abstraction**: `HardwareBackend` trait with `RealHardware` (GPIO) and `SimulatedHardware` (test)

## Data Flow: Command Lifecycle

```
Operator Keypress  →  Frontend  →  Tauri IPC  →  send_operator_command()
                                                      │
                                                      ▼
                                              PiClient.send_to(UDP)
                                                      │
                                                      ▼
                                              Pi Controller receives
                                                      │
                                          ┌───────────┴───────────┐
                                          │  Validation Pipeline  │
                                          │  1. Protocol version  │
                                          │  2. HMAC signature    │
                                          │  3. Auth token        │
                                          │  4. Timestamp window  │
                                          │  5. Sequence number   │
                                          └───────────┬───────────┘
                                                      │
                                                      ▼
                                          apply_state_transition()
                                                      │
                                                      ▼
                                              ACK sent back to desktop
                                                      │
                                                      ▼
                                              Frontend receives event
```

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

## Fault Handling Decision Tree

```
Telemetry received?
  ├── No for 1500ms → Disconnect alarm → Mark system offline
  ├── Yes → Check fault_mask:
  │   ├── THERMAL_CRITICAL → Temperature alert
  │   ├── BATTERY_LOW → Low battery warning
  │   ├── GPIO_INTERLOCK_ERR → E-stop active
  │   ├── WATCHDOG_TIMEOUT → Watchdog tripped
  │   └── GPS_STALE → GPS signal lost
  └── System:
      ├── ARMED + watchdog timeout → FORCE SAFE
      └── E-stop active → FORCE EMERGENCY
```

## Security Model

- **HMAC-SHA256**: Every frame signed with shared secret. Key loaded once at startup via `OnceLock`.
- **Operator Token**: Validated on every command. Never sent over IPC to frontend.
- **Protocol Version**: Rejects cross-version frames.
- **Timestamp Replay Window**: 5 second max clock skew enforced.
- **Sequence Numbers**: Strict monotonically increasing (except E-stop which bypasses).
- **CRITICAL COMMAND RETRY**: Fire/E-stop retried 3 times at 200ms if no ACK.
