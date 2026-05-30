<script lang="ts">
  import { sendOperatorCommand } from '../tauri';
  import type { SystemState } from '../types';

  let { isConnected = false, systemState = 'off' } = $props<{
    isConnected: boolean;
    systemState: SystemState;
  }>();

  // Safety latch toggle to activate the FIRE button
  let safetyLatchReleased = $state(false);
  let isSending = $state(false);
  let commandError = $state('');

  // Reset the safety latch if the state falls back to Safe or Off
  $effect(() => {
    if (systemState !== 'armed') {
      safetyLatchReleased = false;
    }
  });

  async function handleCommand(action: 'arm' | 'disarm' | 'fire') {
    if (!isConnected && action !== 'disarm') return;
    
    // Safety check for FIRE
    if (action === 'fire' && (!safetyLatchReleased || systemState !== 'armed')) {
      commandError = 'CRITICAL: Safety Latch is active or system is not Armed.';
      return;
    }

    try {
      isSending = true;
      commandError = '';
      await sendOperatorCommand(action);
      
      if (action === 'fire') {
        safetyLatchReleased = false; // Relock after firing
      }
    } catch (e: any) {
      commandError = e?.message || `Failed to execute: ${action}`;
    } finally {
      isSending = false;
    }
  }
</script>

<div class="p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1 space-y-md">
  <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute border-b border-hairline pb-xs">Operator Command Console</h3>

  <!-- Authentication Token Status -->
  <div class="flex items-center justify-between p-sm bg-canvas-elevated border border-hairline rounded-md">
    <div class="text-micro font-mono text-ink-faint uppercase">Operator Session Key</div>
    <div class="text-xs font-code text-primary font-semibold">DEMO-OPERATOR-TOKEN-2026</div>
  </div>

  <!-- Main Arm / Disarm Actions -->
  <div class="grid grid-cols-2 gap-md">
    <!-- Arm System -->
    <button
      onclick={() => handleCommand('arm')}
      disabled={!isConnected || systemState === 'armed' || systemState === 'active' || isSending}
      class="py-md px-lg bg-canvas-elevated hover:bg-canvas-elevated/70 border border-status-warning/40 hover:border-status-warning disabled:border-hairline text-status-warning disabled:text-ink-faint rounded-md font-mono text-xs uppercase font-bold tracking-wider transition duration-150 disabled:pointer-events-none"
    >
      {systemState === 'armed' ? 'SYSTEM ARMED' : 'ARM SYSTEM'}
    </button>

    <!-- Disarm System -->
    <button
      onclick={() => handleCommand('disarm')}
      disabled={systemState === 'off' || systemState === 'safe' || isSending}
      class="py-md px-lg bg-canvas-elevated hover:bg-canvas-elevated/70 border border-status-success/40 hover:border-status-success disabled:border-hairline text-status-success disabled:text-ink-faint rounded-md font-mono text-xs uppercase font-bold tracking-wider transition duration-150 disabled:pointer-events-none"
    >
      DISARM SYSTEM
    </button>
  </div>

  <!-- Dual-Action FIRE Command Section -->
  <div class="p-md bg-canvas-elevated/50 border border-hairline rounded-md space-y-md">
    <div class="flex items-center justify-between">
      <div>
        <div class="text-xs font-bold text-ink uppercase tracking-wide">Primary Fire Mechanism</div>
        <div class="text-micro text-ink-mute2 mt-xxs">Requires System ARMED & manual release latch.</div>
      </div>
      
      <!-- Safety Latch Toggle Switch -->
      <label class="flex items-center cursor-pointer select-none">
        <span class="text-micro font-mono uppercase tracking-wider mr-sm {safetyLatchReleased ? 'text-status-warning' : 'text-ink-faint'}">
          {safetyLatchReleased ? 'LATCH RELEASED' : 'LATCH LOCKED'}
        </span>
        <div class="relative">
          <input
            type="checkbox"
            bind:checked={safetyLatchReleased}
            disabled={systemState !== 'armed'}
            class="sr-only"
          />
          <div class="w-10 h-6 bg-canvas border border-hairline rounded-full transition-colors duration-150 {safetyLatchReleased ? 'border-status-warning bg-status-warning/10' : ''}"></div>
          <div class="absolute left-xs top-xs w-md h-md bg-ink-faint rounded-full transition-transform duration-150 {safetyLatchReleased ? 'translate-x-4 bg-status-warning' : ''}"></div>
        </div>
      </label>
    </div>

    <!-- Active Fire Button -->
    <button
      onclick={() => handleCommand('fire')}
      disabled={!isConnected || systemState !== 'armed' || !safetyLatchReleased || isSending}
      class="w-full py-lg px-xl text-button-md font-mono uppercase tracking-widest rounded-md font-black border transition duration-200
        {!isConnected || systemState !== 'armed' || !safetyLatchReleased 
          ? 'bg-canvas border-hairline text-ink-faint cursor-not-allowed'
          : 'bg-status-error/15 border-status-error text-status-error hover:bg-status-error hover:text-canvas cursor-pointer shadow-lvl2'
        }"
    >
      ⚠️ SEND FIRE TRN
    </button>
  </div>

  <!-- Error display -->
  {#if commandError}
    <div class="p-sm bg-status-error/10 border border-status-error/30 text-status-error text-micro font-code rounded-md animate-pulse">
      {commandError}
    </div>
  {/if}
</div>
