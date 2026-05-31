<script lang="ts">
  import { onDestroy, onMount } from 'svelte';
  import { sendOperatorCommand } from '../tauri';

  let { isConnected = false } = $props<{
    isConnected: boolean;
  }>();

  let holdProgress = $state(0);
  let isHolding = $state(false);
  let estopTriggered = $state(false);
  let intervalId: ReturnType<typeof setInterval> | null = null;
  let escPressed = $state(false);

  function startHolding() {
    if (!isConnected || estopTriggered) return;
    isHolding = true;
    holdProgress = 0;

    const startTime = Date.now();
    const duration = 1500;

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
    if (intervalId !== null) {
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

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' && !estopTriggered && isConnected && !escPressed) {
      escPressed = true;
      startHolding();
    }
  }

  function onKeyUp(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      escPressed = false;
      stopHolding();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', onKeyDown);
    document.addEventListener('keyup', onKeyUp);
  });

  onDestroy(() => {
    document.removeEventListener('keydown', onKeyDown);
    document.removeEventListener('keyup', onKeyUp);
    if (intervalId !== null) clearInterval(intervalId);
  });
</script>

<div class="p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1 flex flex-col items-center justify-center space-y-md text-center flex-1">
  <div class="w-full text-left">
    <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute border-b border-hairline pb-xs">Emergency Operations</h3>
  </div>

  {#if !estopTriggered}
    <div class="relative flex flex-col items-center">
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

      {#if isHolding}
        <div class="absolute -bottom-md w-full bg-canvas border border-hairline h-[6px] rounded-full overflow-hidden">
          <div class="h-full bg-status-error" style="width: {holdProgress}%"></div>
        </div>
      {/if}

      <div class="text-[10px] font-code text-ink-faint mt-lg">[hold or press <kbd class="px-xs py-[1px] bg-canvas-elevated border border-hairline rounded-sm text-ink-mute">Esc</kbd> for 1.5s]</div>
    </div>
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
