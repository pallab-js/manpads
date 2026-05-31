<script lang="ts">
  import { onMount } from 'svelte';
  import { getAppState, onTelemetryUpdate, onConnectionStatusChange, onAckUpdate, sendOperatorCommand } from '$lib/tauri';
  import type { TelemetryFrame } from '$lib/types';
  import { playAlertConnect, playAlertDisconnect, playAlertEstop, playAlertFault, playAlertCommandSent } from '$lib/audio';

  import ConnectionStatus from '$lib/components/ConnectionStatus.svelte';
  import TelemetryPanel from '$lib/components/TelemetryPanel.svelte';
  import CommandConsole from '$lib/components/CommandConsole.svelte';
  import EmergencyStop from '$lib/components/EmergencyStop.svelte';
  import AuditFeed from '$lib/components/AuditFeed.svelte';
  import SettingsModal from '$lib/components/SettingsModal.svelte';
  import ShortcutHud from '$lib/components/ShortcutHud.svelte';
  import FireConfirmModal from '$lib/components/FireConfirmModal.svelte';

  let isConnected = $state(false);
  let piIp = $state('127.0.0.1');
  let latencyMs = $state(0);
  let lastTelemetry = $state<TelemetryFrame | null>(null);
  let telemetryHistory = $state<TelemetryFrame[]>([]);
  let auditLog = $state<string[]>([]);
  let isLoading = $state(true);
  let showSettings = $state(false);
  let showShortcuts = $state(false);
  let showFireConfirm = $state(false);

  let piIpRaw = $state('127.0.0.1');
  let piPort = $state(8080);
  let localPort = $state(8081);

  let systemState = $derived(lastTelemetry ? lastTelemetry.systemState : 'off');
  let wasConnected = $state(false);

  function appendLocalAuditEvent(msg: string) {
    const timestamp = Date.now();
    auditLog = [...auditLog, `[${timestamp}] ${msg}`];
    if (auditLog.length > 100) {
      auditLog = auditLog.slice(-100);
    }
  }

  function refreshAuditLog() {
    getAppState().then((state) => {
      auditLog = state.auditLog;
      latencyMs = state.latencyMs;
      piIpRaw = state.piIp;
      piPort = state.piPort || 8080;
      localPort = state.localPort || 8081;
    }).catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      appendLocalAuditEvent(`ERROR: ${msg}`);
      console.error('Failed to get app state:', err);
    });
  }

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === '?' && !e.repeat) {
      showShortcuts = true;
    }
    if ((e.key === 's' || e.key === 'S') && !e.repeat && !(e.target instanceof HTMLInputElement)) {
      showSettings = true;
    }
    if ((e.key === 'a' || e.key === 'A') && !e.repeat && !(e.target instanceof HTMLInputElement)) {
      sendOperatorCommand('arm').catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        appendLocalAuditEvent(`ERROR: ${msg}`);
        console.error('Arm command failed:', err);
      });
    }
    if ((e.key === 'd' || e.key === 'D') && !e.repeat && !(e.target instanceof HTMLInputElement)) {
      sendOperatorCommand('disarm').catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        appendLocalAuditEvent(`ERROR: ${msg}`);
        console.error('Disarm command failed:', err);
      });
    }
    if ((e.key === 'f' || e.key === 'F') && !e.repeat && !(e.target instanceof HTMLInputElement)) {
      showFireConfirm = true;
    }
  }

  onMount(() => {
    document.addEventListener('keydown', onKeyDown);

    getAppState().then((state) => {
      isConnected = state.isConnected;
      piIp = state.piIp;
      latencyMs = state.latencyMs;
      lastTelemetry = state.lastTelemetry;
      auditLog = state.auditLog;
      piIpRaw = state.piIp;
      piPort = state.piPort || 8080;
      localPort = state.localPort || 8081;
      isLoading = false;
    }).catch((err) => {
      console.warn('IPC failed. Running in browser simulation mode:', err);
      isLoading = false;
    });

    let unsubTelemetry: () => void;
    let unsubConn: () => void;
    let unsubAck: () => void;

    onTelemetryUpdate((frame) => {
      lastTelemetry = frame;
      telemetryHistory = [...telemetryHistory.slice(-300), frame];
      if (!isConnected) playAlertConnect();
      isConnected = true;
      refreshAuditLog();

      if (frame.faultMask) playAlertFault();
    }).then((un) => unsubTelemetry = un);

    onConnectionStatusChange((status) => {
      if (wasConnected && !status) playAlertDisconnect();
      wasConnected = status;
      isConnected = status;
      if (!status) {
        latencyMs = 0;
        lastTelemetry = null;
      }
      refreshAuditLog();
    }).then((un) => unsubConn = un);

    onAckUpdate((ack) => {
      refreshAuditLog();
      if (ack.success) playAlertCommandSent();
    }).then((un) => unsubAck = un);

    const interval = setInterval(() => {
      getAppState().then((state) => {
        auditLog = state.auditLog;
        piIp = state.piIp;
        if (!lastTelemetry) {
          isConnected = state.isConnected;
          latencyMs = state.latencyMs;
        }
      }).catch((err: unknown) => {
        const msg = err instanceof Error ? err.message : String(err);
        console.error('Poll failed:', msg);
      });
    }, 1000);

    return () => {
      document.removeEventListener('keydown', onKeyDown);
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
      <div class="text-ink-faint">|</div>
      <button
        onclick={() => showShortcuts = true}
        class="text-ink-faint hover:text-ink transition px-xs"
        title="Keyboard shortcuts (?)"
      >
        [?]
      </button>
      <button
        onclick={() => showSettings = true}
        class="text-ink-faint hover:text-ink transition px-xs"
        title="Settings (S)"
      >
        ⚙
      </button>
    </div>
  </header>

  {#if isLoading}
    <div class="flex-1 flex items-center justify-center">
      <div class="flex flex-col items-center space-y-md">
        <div class="w-sm h-sm border-2 border-primary border-t-transparent rounded-full animate-spin"></div>
        <div class="text-micro font-mono text-ink-faint uppercase tracking-wider">Initializing Control Link...</div>
      </div>
    </div>
  {:else}
    <div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-lg auto-rows-1fr overflow-hidden">
      <div class="lg:col-span-7 flex flex-col space-y-lg overflow-y-auto">
        <ConnectionStatus {isConnected} {latencyMs} {piIp} />
        <TelemetryPanel {lastTelemetry} {telemetryHistory} />
        <AuditFeed {auditLog} />
      </div>

      <div class="lg:col-span-5 flex flex-col space-y-lg overflow-y-auto">
        <CommandConsole {isConnected} {systemState} />
        <EmergencyStop {isConnected} />
      </div>
    </div>
  {/if}
</main>

<SettingsModal
  show={showSettings}
  piIp={piIpRaw}
  {piPort}
  {localPort}
  onclose={(saved: boolean) => { showSettings = false; if (saved) refreshAuditLog(); }}
/>

<FireConfirmModal
  show={showFireConfirm}
  onconfirm={() => {
    showFireConfirm = false;
    sendOperatorCommand('fire').catch((err: unknown) => {
      const msg = err instanceof Error ? err.message : String(err);
      appendLocalAuditEvent(`ERROR: ${msg}`);
      console.error('Fire command failed:', err);
    });
  }}
  oncancel={() => { showFireConfirm = false; }}
/>

<ShortcutHud show={showShortcuts} onclose={() => showShortcuts = false} />
