<script lang="ts">
  import type { TelemetryFrame, SystemState } from '../types';

  let { lastTelemetry = null } = $props<{
    lastTelemetry: TelemetryFrame | null;
  }>();

  // Helper to decode the fault mask bitfield
  function decodeFaults(mask: number): string[] {
    const list: string[] = [];
    if (mask & 1) list.push('WATCHDOG_TIMEOUT');
    if (mask & 2) list.push('GPIO_INTERLOCK_ERR');
    if (mask & 4) list.push('THERMAL_CRITICAL');
    if (mask & 8) list.push('ACTUATOR_FAULT');
    return list;
  }

  // Helper to style system states
  function getStateBadgeClass(state: SystemState): string {
    switch (state) {
      case 'off': return 'bg-ink-faint/20 text-ink-mute border-ink-faint';
      case 'safe': return 'bg-status-success/10 text-status-success border-status-success/30';
      case 'armed': return 'bg-status-warning/10 text-status-warning border-status-warning/30 animate-pulse';
      case 'active': return 'bg-status-error/15 text-status-error border-status-error/40 font-bold';
      case 'emergency': return 'bg-status-error text-canvas border-status-error font-extrabold animate-bounce';
      default: return 'bg-ink-faint/20 text-ink-mute border-ink-faint';
    }
  }
</script>

<div class="grid grid-cols-1 md:grid-cols-2 gap-lg p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1">
  <!-- Left Side: System Metrics -->
  <div class="space-y-md">
    <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute border-b border-hairline pb-xs">Edge Telemetry</h3>
    
    <div class="grid grid-cols-2 gap-md">
      <!-- Battery Status -->
      <div class="p-md bg-canvas-elevated border border-hairline rounded-md">
        <div class="text-micro font-mono text-ink-faint uppercase">Battery Pack</div>
        {#if lastTelemetry}
          <div class="text-body-md font-bold text-ink mt-xxs">{lastTelemetry.batteryVoltage.toFixed(1)} V</div>
          <!-- Battery bar visualization (nominal 10V-14V) -->
          {@const percent = Math.max(0, Math.min(100, ((lastTelemetry.batteryVoltage - 10) / 4) * 100))}
          <div class="w-full bg-canvas-soft h-[3px] rounded-full mt-sm overflow-hidden">
            <div class="h-full bg-primary" style="width: {percent}%"></div>
          </div>
        {:else}
          <div class="text-body-md font-bold text-ink-faint mt-xxs">--</div>
        {/if}
      </div>

      <!-- Temperature Status -->
      <div class="p-md bg-canvas-elevated border border-hairline rounded-md">
        <div class="text-micro font-mono text-ink-faint uppercase">Edge Temp</div>
        {#if lastTelemetry}
          {@const isHigh = lastTelemetry.temperature > 65}
          <div class="text-body-md font-bold mt-xxs {isHigh ? 'text-status-error' : 'text-ink'}">
            {lastTelemetry.temperature.toFixed(1)} °C
          </div>
          <div class="text-micro font-code mt-xs {isHigh ? 'text-status-error' : 'text-ink-mute2'}">
            {isHigh ? 'WARNING: OVERHEAT' : 'STABLE'}
          </div>
        {:else}
          <div class="text-body-md font-bold text-ink-faint mt-xxs">--</div>
        {/if}
      </div>
    </div>

    <!-- GPS Location Data -->
    <div class="p-md bg-canvas-elevated border border-hairline rounded-md">
      <div class="text-micro font-mono text-ink-faint uppercase">GPS Tracker (L1/L2)</div>
      {#if lastTelemetry}
        <div class="grid grid-cols-2 gap-sm mt-xs text-xs font-code text-ink-mute">
          <div><span class="text-ink-faint">LAT:</span> {lastTelemetry.gpsLatitude.toFixed(6)}</div>
          <div><span class="text-ink-faint">LNG:</span> {lastTelemetry.gpsLongitude.toFixed(6)}</div>
        </div>
      {:else}
        <div class="text-xs font-code text-ink-faint mt-xs">GPS LINK OFFLINE</div>
      {/if}
    </div>
  </div>

  <!-- Right Side: System State & Diagnostics -->
  <div class="space-y-md flex flex-col justify-between">
    <div>
      <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute border-b border-hairline pb-xs">Status & Safety</h3>
      
      <div class="flex items-center justify-between mt-md">
        <span class="text-xs font-mono text-ink-mute">Current State:</span>
        {#if lastTelemetry}
          <span class="px-sm py-xxs border rounded-sm text-micro font-code uppercase tracking-widest {getStateBadgeClass(lastTelemetry.systemState)}">
            {lastTelemetry.systemState}
          </span>
        {:else}
          <span class="px-sm py-xxs border border-hairline bg-canvas-elevated text-ink-faint text-micro font-code uppercase">
            LINK OFFLINE
          </span>
        {/if}
      </div>
    </div>

    <!-- Diagnostics Console -->
    <div class="p-md bg-canvas-elevated border border-hairline rounded-md flex-1 mt-md min-h-[90px]">
      <div class="text-micro font-mono text-ink-faint uppercase mb-xs">Diagnostic Alarms</div>
      {#if lastTelemetry}
        {@const faults = decodeFaults(lastTelemetry.faultMask)}
        {#if faults.length === 0}
          <div class="flex items-center space-x-xs text-xs text-status-success font-code">
            <span>🟢</span>
            <span>SYSTEMS NOMINAL • WATCHDOG ARMED</span>
          </div>
        {:else}
          <div class="space-y-xxs">
            {#each faults as fault}
              <div class="flex items-center space-x-xs text-xs text-status-error font-code font-bold">
                <span>⚠️</span>
                <span>{fault}</span>
              </div>
            {/each}
          </div>
        {/if}
      {:else}
        <div class="text-xs font-code text-ink-faint">DIAGNOSTICS PENDING LINK...</div>
      {/if}
    </div>
  </div>
</div>
