# MANPADS TD Control Suite

A dark-themed technology demonstration control panel for remote Raspberry Pi edge integration. This project showcases a full-stack Rust + SvelteKit application with real-time telemetry, command-and-control over UDP, hardware safety interlocks, and a simulated edge controller for offline development.

## Architecture

| Component | Directory | Stack |
|-----------|-----------|-------|
| **Desktop App** | `desktop-app/` | Tauri v2, SvelteKit, TypeScript, TailwindCSS |
| **Pi Controller** | `pi-controller/` | Rust tokio, UDP, GPIO (rppal), HMAC auth |
| **Simulator** | `pi-controller/src/bin/pi-simulator.rs` | Standalone binary that emulates the edge controller locally |

### Desktop App (`desktop-app/`)
Tauri v2 shell wrapping a SvelteKit frontend. Communicates with the Pi controller over UDP using framed JSON messages. Features include:
- Real-time telemetry panel (10Hz stream) with history chart
- Operator command console with safety latch and hold-to-confirm
- Emergency stop with hold-to-activate
- Audit log with rate-limited event capture (capped at 100 entries)
- Persisted connection settings
- Keyboard shortcuts

### Pi Controller (`pi-controller/`)
Rust binary that runs on a Raspberry Pi (or as a simulator). Handles:
- UDP receiver loop for inbound operator commands
- HMAC-signed message authentication
- State machine: `Off → Safe → Armed → Active → Emergency`
- GPIO-based hardware interlocks and actuator control
- 10Hz telemetry broadcast back to the desktop app
- Watchdog: auto-disarms if command heartbeats are lost for 3s
- Connection heartbeat: marks disconnected after 1.5s without telemetry

## Quick Start

### Prerequisites
- [Rust](https://rustup.rs/) (stable toolchain)
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 10
- Raspberry Pi (optional — simulator works without one)

### Run the desktop app (with simulator)

```sh
# Terminal 1: Start the Pi simulator
cargo run --bin pi-simulator

# Terminal 2: Start the desktop app
pnpm -C desktop-app/frontend dev
```

The Tauri desktop window will launch and auto-connect to the simulator.

### Build for production

```sh
pnpm -C desktop-app/frontend build
cargo tauri build --manifest-path desktop-app/src-tauri/Cargo.toml
```

## Communication Protocol

Messages are JSON-framed over UDP with an HMAC-SHA256 signature field (`hmac`).

| Direction | Frame | Rate | Purpose |
|-----------|-------|------|---------|
| Desktop → Pi | `CommandPayload` | On-demand | Operator commands (arm, disarm, fire, estop) |
| Pi → Desktop | `AckFrame` | Per command | Acknowledgment with success/error |
| Pi → Desktop | `TelemetryFrame` | 10Hz | Battery voltage, temperature, GPS, system state, fault mask |

The HMAC secret is configurable via the `MANPADS_HMAC_SECRET` environment variable.

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `MANPADS_HMAC_SECRET` | `manpads-td-secret-key` | HMAC signing key for message authentication |
| `MANPADS_OPERATOR_TOKEN` | `DEMO-OPERATOR-TOKEN-2026` | Operator session token validated on each command |

## Development

### Rust checks
```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

### Frontend checks
```sh
pnpm -C desktop-app/frontend check
```

## License
MIT
