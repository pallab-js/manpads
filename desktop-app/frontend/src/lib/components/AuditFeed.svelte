<script lang="ts">
  import { onMount } from 'svelte';

  let { auditLog = [] } = $props<{
    auditLog: string[];
  }>();

  let logContainer: HTMLElement | null = $state(null);

  // Auto-scroll to the bottom of the console whenever new log entries are appended
  $effect(() => {
    if (auditLog.length && logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });

  onMount(() => {
    if (logContainer) {
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  });
</script>

<div class="p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1 flex flex-col space-y-md">
  <div class="flex items-center justify-between border-b border-hairline pb-xs">
    <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute">Audit & Command Feed</h3>
    <span class="px-sm py-[2px] bg-canvas-elevated border border-hairline rounded-sm text-[10px] font-code text-ink-faint uppercase">
      Append-Only
    </span>
  </div>

  <!-- Terminal log container -->
  <div
    bind:this={logContainer}
    class="flex-1 min-h-[140px] max-h-[220px] overflow-y-auto p-md bg-canvas border border-hairline rounded-md font-code text-[11px] text-ink-mute space-y-sm scrollbar-thin scrollbar-thumb-canvas-elevated"
  >
    {#if auditLog.length === 0}
      <div class="text-ink-faint italic font-mono uppercase">Console Feed Idle...</div>
    {:else}
      {#each auditLog as log}
        <!-- Formats timestamps and log lines with color highlights -->
        {@const match = log.match(/^\[(\d+)\] (.*)/)}
        {#if match}
          {@const timestamp = new Date(parseInt(match[1])).toLocaleTimeString()}
          {@const text = match[2]}
          <div class="flex space-x-md border-l-2 border-primary/20 pl-sm">
            <span class="text-ink-faint select-none shrink-0">{timestamp}</span>
            <span class="break-all {text.includes('ERROR') ? 'text-status-error font-bold' : text.includes('WARNING') ? 'text-status-warning' : 'text-ink-mute'}">
              {text}
            </span>
          </div>
        {:else}
          <div class="pl-sm">{log}</div>
        {/if}
      {/each}
    {/if}
  </div>
</div>
