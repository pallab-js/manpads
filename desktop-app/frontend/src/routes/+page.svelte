<script lang="ts">
  import { onMount } from 'svelte';
  import { getAppState, onTelemetryUpdate, onConnectionStatusChange, onAckUpdate } from '$lib/tauri';
  import type { TelemetryFrame } from '$lib/types';
  
  import ConnectionStatus from '$lib/components/ConnectionStatus.svelte';
  import TelemetryPanel from '$lib/components/TelemetryPanel.svelte';
  import CommandConsole from '$lib/components/CommandConsole.svelte';
  import EmergencyStop from '$lib/components/EmergencyStop.svelte';
  import AuditFeed from '$lib/components/AuditFeed.svelte';

  // Svelte 5 Runes for reactive state
  let isConnected = $state(false);
  let piIp = $state('127.0.0.1:8080');
  let latencyMs = $state(0);
  let lastTelemetry = $state<TelemetryFrame | null>(null);
  let auditLog = $state<string[]>([]);

  // Derived system state
  let systemState = $derived(lastTelemetry ? lastTelemetry.systemState : 'off');

  onMount(() => {
    // Initial State Fetch
    getAppState().then((state) => {
      isConnected = state.isConnected;
      piIp = state.piIp;
      latencyMs = state.latencyMs;
      lastTelemetry = state.lastTelemetry;
      auditLog = state.auditLog;
    }).catch((err) => {
      console.warn('IPC failed. Running in browser simulation mode:', err);
    });

    let unsubTelemetry: () => void;
    let unsubConn: () => void;
    let unsubAck: () => void;

    // Real-Time Event Listeners
    onTelemetryUpdate((frame) => {
      lastTelemetry = frame;
      isConnected = true;
    }).then((un) => unsubTelemetry = un);

    onConnectionStatusChange((status) => {
      isConnected = status;
      if (!status) {
        latencyMs = 0;
        lastTelemetry = null;
      }
    }).then((un) => unsubConn = un);

    onAckUpdate((ack) => {
      getAppState().then((state) => {
        auditLog = state.auditLog;
        latencyMs = state.latencyMs;
      });
    }).then((un) => unsubAck = un);

    // Sync state logs at 1Hz
    const interval = setInterval(() => {
      getAppState().then((state) => {
        auditLog = state.auditLog;
        piIp = state.piIp;
        if (!lastTelemetry) {
          isConnected = state.isConnected;
          latencyMs = state.latencyMs;
        }
      }).catch(() => {});
    }, 1000);

    return () => {
      if (unsubTelemetry) unsubTelemetry();
      if (unsubConn) unsubConn();
      if (unsubAck) unsubAck();
      clearInterval(interval);
    };
  });
</script>

<svelte:head>
  <title>MANPADS TD Control Suite</title>
</svelte:head>

<main class="min-h-screen bg-canvas text-ink flex flex-col p-lg md:p-xl space-y-lg select-none">
  <!-- Header Bar -->
  <header class="flex flex-col md:flex-row items-start md:items-center justify-between border-b border-hairline pb-md space-y-sm md:space-y-0">
    <div>
      <h1 class="text-xs font-mono uppercase tracking-widest text-primary font-bold">Tactical Control Terminal</h1>
      <div class="text-display-xl font-display font-extrabold tracking-tight text-ink mt-xxs">
        MANPADS <span class="text-ink-mute">TD-SUITE</span>
      </div>
    </div>
    
    <div class="flex items-center space-x-md font-mono text-micro text-ink-mute bg-canvas-soft border border-hairline px-md py-sm rounded-md shadow-lvl1">
      <div class="flex items-center space-x-xs">
        <span class="inline-block w-xs h-xs rounded-full bg-primary"></span>
        <span>SYS STATE: <span class="text-ink font-bold uppercase">{systemState}</span></span>
      </div>
      <div class="text-ink-faint">|</div>
      <div>DOM LOCK: <span class="text-status-success font-bold">ACTIVE</span></div>
    </div>
  </header>

  <!-- Two Column Layout Grid -->
  <div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-lg overflow-hidden">
    <!-- Left Column: Metrics & Logs (7 / 12) -->
    <div class="lg:col-span-7 flex flex-col space-y-lg justify-start">
      <!-- Connection Status Card -->
      <ConnectionStatus {isConnected} {latencyMs} {piIp} />
      
      <!-- Real-time Telemetry Panel -->
      <TelemetryPanel {lastTelemetry} />
      
      <!-- Event Audit log -->
      <AuditFeed {auditLog} />
    </div>

    <!-- Right Column: Operator Controls (5 / 12) -->
    <div class="lg:col-span-5 flex flex-col space-y-lg">
      <!-- Critical Fire & Safety command console -->
      <CommandConsole {isConnected} {systemState} />
      
      <!-- Emergency lockdown latch -->
      <EmergencyStop {isConnected} />
    </div>
  </div>
</main>
