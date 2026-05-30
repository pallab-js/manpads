# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-30

### Added
- Domain error types (`ControllerError`, `AppError`) with `thiserror` across both crates
- `ControllerConfig` struct with env-var-based configuration loading
- `ControllerDaemon` struct encapsulating all tasks (command receiver, telemetry sender, watchdog)
- `HardwareBackend` trait with `RealHardware` and `SimulatedHardware` implementations
- Protocol version field (`protocolVersion: u8`) in all frame types
- `fault_flags` constants module in Rust and TypeScript
- Timestamp replay-window validation (5 second max clock skew)
- Critical command retry for Fire/Estop (3 retries at 200ms intervals)
- `command-ack-timeout` event emitted to frontend when retries exhausted
- `CancellationToken`-based graceful shutdown for all tokio tasks
- Structured JSON logging with `tracing-subscriber` `json` feature
- Persistent audit log writing to disk via `AuditWriter`
- Configurable watchdog timeout (default 1500ms, via `MANPADS_WATCHDOG_MS`)
- CBOR transport feature flag (`cbor-transport`)
- Frontend Fire confirmation modal requiring explicit click
- Frontend IP/port validation in settings modal
- Frontend command in-flight state with disabled buttons
- Proptest-based property tests for state machine
- Frontend Vitest unit tests for `FaultFlags` and command action parsing
- CI/CD pipeline with `rust-check`, `frontend-check`, `cross-compile-pi`, `build-desktop`, and `security-audit` jobs
- `.env.example` with all documented variables
- `scripts/manpads-controller.service` systemd unit for Pi deployment
- `docs/architecture.md` with system overview
- `docs/pi-deployment.md` with deployment instructions

### Changed
- `MANPADS_HMAC_SECRET` and `MANPADS_OPERATOR_TOKEN` are now required at runtime — no fallback defaults
- HMAC key is loaded once at startup via `OnceLock` instead of per-frame env var lookup
- `software_estop` field is now private; accessed only via `activate_software_estop()` / `clear_software_estop()`
- `AppState` split into `AppStateInternal` (secrets) and `AppStateView` (IPC-safe public view)
- `pi_ip` in `AppStateView` stores bare IP address (no port suffix)
- `std::sync::Mutex` replaced with `tokio::sync::RwLock` in async contexts
- `main.rs` reduced to ~30 lines; all logic moved to `ControllerDaemon` and `config.rs`
- `pi-simulator.rs` now uses `ControllerDaemon<SimulatedHardware>` — no duplicated logic
- All logs use structured fields instead of string interpolation
- Workspace version unified to `0.2.0`
- Test coverage requirement: minimum 80% line coverage (CI gate)

### Security
- No hardcoded secrets in compiled binaries
- `operator_token` is never serialized over IPC to frontend
- Timestamp replay window prevents replay attacks
- Protocol version field prevents cross-version injection

### Removed
- Removed `// TD-ONLY` comments from production code
- Removed `DEMO-OPERATOR-TOKEN-2026` and `manpads-td-secret-key` defaults
- Removed `operatorToken` from frontend `AppStateData` type
