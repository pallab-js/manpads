<script lang="ts">
  import { onDestroy } from 'svelte';
  import { sendOperatorCommand } from '../tauri';

  let { isConnected = false } = $props<{
    isConnected: boolean;
  }>();

  let holdProgress = $state(0); // 0 to 100
  let isHolding = $state(false);
  let estopTriggered = $state(false);
  let intervalId: any = null;

  function startHolding() {
    if (!isConnected || estopTriggered) return;
    isHolding = true;
    holdProgress = 0;
    
    const startTime = Date.now();
    const duration = 1500; // 1.5s hold required

    intervalId = setInterval(() => {
      const elapsed = Date.now() - startTime;
      holdProgress = Math.min(100, (elapsed / duration) * 100);
      
      if (holdProgress >= 100) {
        triggerEstop();
      }
    }, 30);
  }

  function stopHolding() {
    isHolding = false;
    if (intervalId) {
      clearInterval(intervalId);
      intervalId = null;
    }
    if (!estopTriggered) {
      holdProgress = 0;
    }
  }

  async function triggerEstop() {
    stopHolding();
    estopTriggered = true;
    holdProgress = 100;
    try {
      await sendOperatorCommand('estop');
    } catch (e) {
      console.error('Failed to trigger ESTOP:', e);
    }
  }

  function resetEstop() {
    estopTriggered = false;
    holdProgress = 0;
  }

  onDestroy(() => {
    if (intervalId) clearInterval(intervalId);
  });
</script>

<div class="p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1 flex flex-col items-center justify-center space-y-md text-center">
  <div class="w-full text-left">
    <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute border-b border-hairline pb-xs">Emergency Operations</h3>
  </div>

  {#if !estopTriggered}
    <div class="relative flex flex-col items-center">
      <!-- Hold button -->
      <button
        onmousedown={startHolding}
        onmouseup={stopHolding}
        onmouseleave={stopHolding}
        ontouchstart={startHolding}
        ontouchend={stopHolding}
        disabled={!isConnected}
        class="w-32 h-32 rounded-full border border-status-error/40 flex flex-col items-center justify-center transition duration-150 active:scale-95 disabled:pointer-events-none select-none
          {isHolding 
            ? 'bg-status-error text-canvas border-status-error shadow-lvl2' 
            : 'bg-canvas-elevated text-status-error hover:bg-status-error/10 hover:border-status-error'
          }"
      >
        <span class="text-xs font-mono font-black tracking-wider uppercase">HOLD TO</span>
        <span class="text-body-md font-mono font-black tracking-widest uppercase mt-xxs">E-STOP</span>
        <span class="text-[10px] font-mono opacity-60 mt-xs">{isHolding ? 'HOLDING...' : '1.5 SECS'}</span>
      </button>

      <!-- Progress bar -->
      {#if isHolding}
        <div class="absolute -bottom-md w-full bg-canvas border border-hairline h-[6px] rounded-full overflow-hidden">
          <div class="h-full bg-status-error" style="width: {holdProgress}%"></div>
        </div>
      {/if}
    </div>
    <p class="text-micro font-mono text-ink-faint max-w-xs mt-sm leading-relaxed">
      WARNING: Activating emergency shutdown disarms all triggers and locks edge hardware into safety state.
    </p>
  {:else}
    <div class="w-full flex flex-col items-center space-y-sm">
      <div class="w-16 h-16 rounded-full bg-status-error flex items-center justify-center text-xl animate-bounce">
        🚨
      </div>
      <div>
        <div class="text-body-md font-mono font-black text-status-error uppercase tracking-widest">EMERGENCY SYSTEM LOCKDOWN</div>
        <div class="text-micro text-ink-mute mt-xxs">Estop command transmitted. Latching logic active.</div>
      </div>
      <button
        onclick={resetEstop}
        class="px-lg py-sm bg-canvas-elevated hover:bg-canvas-elevated/70 border border-hairline hover:border-ink-mute text-ink-mute hover:text-ink text-micro font-mono rounded-md uppercase tracking-wider transition duration-150"
      >
        Acknowledge & Reset Interlock
      </button>
    </div>
  {/if}
</div>
