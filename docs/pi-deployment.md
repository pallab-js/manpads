# Pi Deployment Guide

## Prerequisites

- Raspberry Pi (3 or newer, running Raspberry Pi OS Lite 64-bit)
- Static IP or mDNS hostname configured
- Rust cross-compilation toolchain (or compile on-device)

## Installation

### 1. Transfer the binary

```bash
# From your development machine, copy the binary to the Pi
scp pi-controller pi@raspberrypi.local:/home/pi/manpads/
```

### 2. Configure environment

```bash
# On the Pi, create the env file
sudo mkdir -p /home/pi/manpads
sudo nano /home/pi/manpads/.env
```

Contents of `/home/pi/manpads/.env`:

```ini
MANPADS_HMAC_SECRET=<run: openssl rand -hex 32>
MANPADS_OPERATOR_TOKEN=<run: openssl rand -hex 16>
MANPADS_LISTEN_ADDR=0.0.0.0:8080
MANPADS_DESKTOP_ADDR=<desktop-ip>:8081
MANPADS_WATCHDOG_MS=1500
MANPADS_TELEMETRY_INTERVAL_MS=100
RUST_LOG=pi_controller=info,warn
```

```bash
# Secure the env file
sudo chmod 600 /home/pi/manpads/.env
sudo chown pi:pi /home/pi/manpads/.env
```

### 3. Install systemd service

```bash
# Copy the service file
sudo cp manpads-controller.service /etc/systemd/system/

# Reload systemd
sudo systemctl daemon-reload

# Enable and start
sudo systemctl enable manpads-controller
sudo systemctl start manpads-controller

# Check status
sudo systemctl status manpads-controller

# View logs
sudo journalctl -u manpads-controller -f
```

### 4. Firewall configuration

```bash
# Allow UDP on the control port
sudo ufw allow 8080/udp
```

## Cross-Compilation

Build for aarch64 Raspberry Pi from your development machine:

```bash
# Install cross-compiler
sudo apt-get install gcc-aarch64-linux-gnu

# Build
CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=aarch64-linux-gnu-gcc \
  cargo build --package pi-controller --target aarch64-unknown-linux-gnu --release
```

The binary will be at `target/aarch64-unknown-linux-gnu/release/pi-controller`.

## Updating

```bash
# Upload new binary
scp pi-controller pi@raspberrypi.local:/home/pi/manpads/

# Restart the service
ssh pi@raspberrypi.local "sudo systemctl restart manpads-controller"
```
