<script lang="ts">
  import { sendOperatorCommand } from '../tauri';
  import type { SystemState } from '../types';

  let { isConnected = false, systemState = 'off' } = $props<{
    isConnected: boolean;
    systemState: SystemState;
  }>();

  let safetyLatchReleased = $state(false);
  let isSending = $state(false);
  let commandError = $state('');
  let pendingCommand = $state<'arm' | 'disarm' | 'fire' | 'estop' | null>(null);

  let disarmHoldProgress = $state(0);
  let isDisarmHolding = $state(false);
  let disarmIntervalId: ReturnType<typeof setInterval> | null = null;

  let cooldownRemaining = $state(0);
  let cooldownIntervalId: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    if (systemState !== 'armed') {
      safetyLatchReleased = false;
    }
  });

  function startDisarmHold() {
    if (!isConnected || isSending) return;
    isDisarmHolding = true;
    disarmHoldProgress = 0;

    const startTime = Date.now();
    const duration = 1000;

    disarmIntervalId = setInterval(() => {
      const elapsed = Date.now() - startTime;
      disarmHoldProgress = Math.min(100, (elapsed / duration) * 100);
      if (disarmHoldProgress >= 100) {
        executeDisarm();
      }
    }, 30);
  }

  function stopDisarmHold() {
    isDisarmHolding = false;
    if (disarmIntervalId !== null) {
      clearInterval(disarmIntervalId);
      disarmIntervalId = null;
    }
    disarmHoldProgress = 0;
  }

  async function executeDisarm() {
    stopDisarmHold();
    await handleCommand('disarm');
  }

  function startCooldown(seconds: number) {
    cooldownRemaining = seconds;
    cooldownIntervalId = setInterval(() => {
      cooldownRemaining--;
      if (cooldownRemaining <= 0) {
        if (cooldownIntervalId !== null) {
          clearInterval(cooldownIntervalId);
          cooldownIntervalId = null;
        }
      }
    }, 1000);
  }

  async function handleCommand(action: 'arm' | 'disarm' | 'fire') {
    if (!isConnected && action !== 'disarm') return;

    if (action === 'fire' && (!safetyLatchReleased || systemState !== 'armed')) {
      commandError = 'CRITICAL: Safety Latch is active or system is not Armed.';
      return;
    }

    if (cooldownRemaining > 0) {
      commandError = `Rate-limited. Wait ${cooldownRemaining}s before next command.`;
      return;
    }

    try {
      pendingCommand = action;
      isSending = true;
      commandError = '';
      await sendOperatorCommand(action);

      if (action === 'fire') {
        safetyLatchReleased = false;
      }

      startCooldown(1);
    } catch (e: unknown) {
      commandError = e instanceof Error ? e.message : `Failed to execute: ${action}`;
    } finally {
      isSending = false;
      pendingCommand = null;
    }
  }
</script>

<div class="p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1 space-y-md">
  <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute border-b border-hairline pb-xs">Operator Command Console</h3>

  {#if commandError}
    <div class="p-sm bg-status-error/10 border border-status-error/30 text-status-error text-micro font-code rounded-md animate-pulse">
      {commandError}
    </div>
  {/if}

  <div class="grid grid-cols-2 gap-md">
    <button
      onclick={() => handleCommand('arm')}
      disabled={!isConnected || systemState === 'armed' || systemState === 'active' || isSending || cooldownRemaining > 0}
      class="py-md px-lg bg-canvas-elevated hover:bg-canvas-elevated/70 border border-status-warning/40 hover:border-status-warning disabled:border-hairline text-status-warning disabled:text-ink-faint rounded-md font-mono text-xs uppercase font-bold tracking-wider transition duration-150 disabled:pointer-events-none"
    >
      {#if isSending && pendingCommand === 'arm'}
        <span class="animate-pulse">SENDING...</span>
      {:else if cooldownRemaining > 0}
        WAIT {cooldownRemaining}s
      {:else if systemState === 'armed'}
        SYSTEM ARMED
      {:else}
        ARM SYSTEM
      {/if}
    </button>

    <button
      onmousedown={startDisarmHold}
      onmouseup={stopDisarmHold}
      onmouseleave={stopDisarmHold}
      ontouchstart={startDisarmHold}
      ontouchend={stopDisarmHold}
      disabled={systemState === 'off' || systemState === 'safe' || isSending || cooldownRemaining > 0}
      class="relative py-md px-lg bg-canvas-elevated hover:bg-canvas-elevated/70 border border-status-success/40 hover:border-status-success disabled:border-hairline text-status-success disabled:text-ink-faint rounded-md font-mono text-xs uppercase font-bold tracking-wider transition duration-150 disabled:pointer-events-none select-none overflow-hidden"
    >
      {#if isDisarmHolding}
        <div class="absolute inset-0 bg-status-success/20" style="width: {disarmHoldProgress}%"></div>
      {/if}
      <span class="relative z-10">
        {#if cooldownRemaining > 0}
          WAIT {cooldownRemaining}s
        {:else if isDisarmHolding}
          HOLD TO CONFIRM
        {:else}
          DISARM SYSTEM
        {/if}
      </span>
    </button>
  </div>

  <div class="p-md bg-canvas-elevated/50 border border-hairline rounded-md space-y-md">
    <div class="flex items-center justify-between">
      <div>
        <div class="text-xs font-bold text-ink uppercase tracking-wide">Primary Fire Mechanism</div>
        <div class="text-micro text-ink-mute2 mt-xxs">Requires System ARMED & manual release latch.</div>
      </div>

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

    <button
      onclick={() => handleCommand('fire')}
      disabled={!isConnected || systemState !== 'armed' || !safetyLatchReleased || isSending || cooldownRemaining > 0}
      class="w-full py-lg px-xl text-sm font-mono uppercase tracking-widest rounded-md font-black border transition duration-200
        {!isConnected || systemState !== 'armed' || !safetyLatchReleased
          ? 'bg-canvas border-hairline text-ink-faint cursor-not-allowed'
          : 'bg-status-error/15 border-status-error text-status-error hover:bg-status-error hover:text-canvas cursor-pointer shadow-lvl2'
        }"
    >
      {#if isSending && pendingCommand === 'fire'}
        <span class="animate-pulse">SENDING FIRE...</span>
      {:else if cooldownRemaining > 0}
        WAIT {cooldownRemaining}s
      {:else}
        SEND FIRE TRN
      {/if}
    </button>
  </div>
</div>
