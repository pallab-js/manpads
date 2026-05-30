<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  let {
    show = false,
    onclose = () => {},
  } = $props<{
    show: boolean;
    onclose: () => void;
  }>();

  function onKeyDown(e: KeyboardEvent) {
    if (e.key === 'Escape' || e.key === '?') {
      onclose();
    }
  }

  onMount(() => document.addEventListener('keydown', onKeyDown));
  onDestroy(() => document.removeEventListener('keydown', onKeyDown));
</script>

{#if show}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    role="dialog" aria-modal="true" aria-label="Keyboard shortcuts"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    onclick={onclose}
    onkeydown={(e) => { if (e.key === 'Escape') onclose(); }}
  >
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
    <div
      class="bg-canvas-soft border border-hairline rounded-xl shadow-lvl2 w-full max-w-[420px] mx-4"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between px-6 py-4 border-b border-hairline">
        <h2 class="text-sm font-mono uppercase tracking-[0.08em] text-ink-mute font-semibold">Keyboard Shortcuts</h2>
        <button
          onclick={onclose}
          class="text-xs font-mono text-ink-faint border border-hairline px-2 py-[2px] rounded hover:text-ink hover:border-ink-mute transition-colors"
        >
          ESC
        </button>
      </div>

      <div class="px-6 py-5 space-y-3">
        {#each [
          { label: 'Emergency Stop', key: 'Hold Esc (1.5s)' },
          { label: 'Arm System', key: 'A' },
          { label: 'Disarm System', key: 'Hold D (1s)' },
          { label: 'Fire (when armed)', key: 'F' },
          { label: 'Toggle Settings', key: 'S' },
          { label: 'Show Shortcuts', key: '?' },
        ] as item}
          <div class="flex items-center justify-between">
            <span class="text-sm font-mono text-ink-mute">{item.label}</span>
            <kbd class="text-sm font-mono text-ink bg-canvas-elevated border border-hairline px-2 py-[2px] rounded-sm">{item.key}</kbd>
          </div>
        {/each}
      </div>

      <div class="px-6 pb-5 text-center text-xs font-mono text-ink-faint">
        Hold modifiers for safety-critical actions
      </div>
    </div>
  </div>
{/if}
