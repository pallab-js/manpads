# MANPADS TD Control Suite

An enterprise-grade, permanently dark-adapted technology demonstration (TD) control panel for remote Raspberry Pi edge integration.

## Architecture
- **Desktop App (`desktop-app`)**: Tauri v2, SvelteKit, TypeScript, TailwindCSS.
- **Pi Controller (`pi-controller`)**: Rust-based edge controller featuring GPIO safety interlocks, watchdogs, and status loops.
- **Simulation**: Virtualized local agent for local desktop integration without physical Raspberry Pi hardware.

## Specifications
- **Design System**: Supabaze Permanent Dark Theme.
- **IPC Protocol**: Spec-driven IPC and JSON message framing with HMAC validation stubs.
- **Constraints**: Optimized for low footprint (8GB RAM), throttled to 10Hz, zero atmospheric layout elements.

## License
MIT
