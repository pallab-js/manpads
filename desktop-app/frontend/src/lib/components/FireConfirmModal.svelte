<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  let {
    show = false,
    onconfirm = () => {},
    oncancel = () => {},
  } = $props<{
    show: boolean;
    onconfirm: () => void;
    oncancel: () => void;
  }>();

  let countdown = $state(3);
  let intervalId: ReturnType<typeof setInterval> | null = null;

  $effect(() => {
    if (show) {
      countdown = 3;
      intervalId = setInterval(() => {
        countdown--;
        if (countdown <= 0) {
          if (intervalId) clearInterval(intervalId);
          oncancel();
        }
      }, 1000);
    } else {
      if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
      }
    }
  });

  function handleConfirm() {
    if (intervalId) clearInterval(intervalId);
    onconfirm();
  }

  function handleCancel() {
    if (intervalId) clearInterval(intervalId);
    oncancel();
  }
</script>

{#if show}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    role="dialog" aria-modal="true" aria-label="Fire confirmation"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    onclick={handleCancel}
    onkeydown={(e) => { if (e.key === 'Escape') handleCancel(); }}
  >
    <div
      role="none"
      class="bg-canvas-soft border-2 border-status-error rounded-xl shadow-lvl3 w-full max-w-md mx-4 text-center"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="px-6 py-8 space-y-4">
        <div class="text-6xl">🔥</div>
        <h2 class="text-xl font-mono font-black uppercase tracking-widest text-status-error">FIRE CONFIRMATION</h2>
        <p class="text-sm font-mono text-ink-mute">
          You are about to send the FIRE command. This action is irreversible.
        </p>
        <p class="text-xs font-mono text-ink-faint">
          Auto-cancelling in {countdown}s
        </p>
      </div>

      <div class="flex justify-center gap-4 px-6 pb-6">
        <button
          onclick={handleCancel}
          class="px-6 py-3 text-sm font-mono text-ink-mute border border-hairline rounded-lg hover:text-ink hover:border-ink-mute transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleConfirm}
          class="px-6 py-3 text-sm font-mono font-black uppercase tracking-widest bg-status-error text-canvas rounded-lg hover:opacity-90 transition-opacity"
        >
          CONFIRM FIRE
        </button>
      </div>
    </div>
  </div>
{/if}
