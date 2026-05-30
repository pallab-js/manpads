# MANPADS TD Control Suite: Operator Manual

This manual provides the technical documentation, safety interlock guidelines, threat modeling, and simulation execution steps for the **MANPADS TD Control Suite** (Technology Demonstration).

---

## 1. System Architecture & Wiring Diagram
The control link establishes a high-frequency, low-latency UDP link between the laptop operator dashboard and the edge controller daemon.

```mermaid
graph TD
    subgraph Laptop (Tauri Dashboard)
        UI[SvelteKit Frontend] <-->|IPC Core Commands| BK[Tauri Rust Backend]
        BK <-->|UDP socket Client 8081| NET[UDP Listener]
    end

    subgraph Raspberry Pi (Edge Controller)
        DA[Rust Daemon 8080] <-->|Internal States| ACT[Actuators Relay]
        DA <-->|Sensors Polling| SEN[Sensor Simulator]
        DA <-->|GPIO Authority| INT[Safety Interlock]
    end

    NET <-->|WiFi / Ethernet Link| DA
```

### Physical Pin Assignments (Raspberry Pi target)
When compiling for Linux architectures, the edge daemon interacts with the physical GPIO subsystem:
- **GPIO 23 (Output)**: Driven HIGH to trigger active firing relays (Active status).
- **GPIO 26 (Input, Pull-up)**: Connected to physical Safety E-STOP switch. Grounding this pin immediately halts all actuator triggers.

---

## 2. Integrated Safety Watchdogs

### Laptop Link Watchdog
If the edge controller is in an `Armed` or `Active` state and fails to receive any valid command or network heartbeat from the Laptop dashboard for **3000ms**, the edge daemon automatically triggers an **auto-disarm fail-safe**, resetting the state to `Safe` and cutting power to all triggers.

### Physical/Software E-Stop
- Triggering the emergency lockdown halts all actuators immediately.
- Transition to the `Emergency` state acts as a hard latch. To resume operations, the operator must explicitly click **"Acknowledge & Reset Interlock"** on the dashboard, which sends a `Disarm` command resetting the safety flags.

---

## 3. Threat Model Summary

- **Link Spoofing / Replay Attacks**: Mitigated by sequence counter validation. The edge controller maintains a thread-safe tracker of the highest processed sequence number (`last_command_seq`) and discards any frame containing an older or equal sequence.
- **Payload Tampering**: Mitigated by our **HMAC signature stub**. Payloads are verified with a SHA256 deterministic hash generated using a pre-shared secret key `"manpads-td-secret-key"`. If the signature does not match, the packet is silently discarded.
- **Operator Session Unauthorized Access**: Mitigated by explicit verification of the session key `"DEMO-OPERATOR-TOKEN-2026"` at both the Tauri and Pi-daemon levels.

---

## 4. Technology Demonstration Launch Sequence

### Step 1: Start the Edge Simulator
Launch the simulated Pi controller on your local machine. It will listen on port `8080` and emit telemetry updates to `8081` at 10Hz, injecting network latency and packet drops:
```bash
./scripts/run-simulator.sh
```

### Step 2: Launch the Tauri Control Client
Run SvelteKit and Tauri in development mode:
```bash
pnpm --prefix desktop-app tauri dev
```

### Step 3: Run Verification Test
1. Click **ARM SYSTEM** on the dashboard. Verify that the console state changes to `ARMED` and logs report the execution sequence.
2. Toggle the **Primary Fire Mechanism Latch** to `LATCH RELEASED` (unlocking the safety).
3. Hold the **E-STOP** button for 1.5 seconds. Verify that the console triggers a red lockdown layout and locks out all further arm/fire controls.
