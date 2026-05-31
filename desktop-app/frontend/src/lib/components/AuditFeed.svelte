<script lang="ts">
  import { onMount } from 'svelte';
  import { exportAuditLog } from '../tauri';

  let { auditLog = [] } = $props<{
    auditLog: string[];
  }>();

  let logContainer: HTMLElement | null = $state(null);
  let exportStatus = $state('');

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

  async function handleExport() {
    exportStatus = 'EXPORTING...';
    try {
      const path = await exportAuditLog();
      exportStatus = `SAVED: ${path}`;
      setTimeout(() => { exportStatus = ''; }, 3000);
    } catch (e: unknown) {
      exportStatus = `ERROR: ${e instanceof Error ? e.message : 'Export failed'}`;
      setTimeout(() => { exportStatus = ''; }, 4000);
    }
  }
</script>

<div class="p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1 flex flex-col space-y-md flex-1">
  <div class="flex items-center justify-between border-b border-hairline pb-xs">
    <h3 class="text-xs font-mono uppercase tracking-wider text-ink-mute">Audit & Command Feed</h3>
    <div class="flex items-center space-x-sm">
      {#if exportStatus}
        <span class="text-[10px] font-code text-ink-faint">{exportStatus}</span>
      {/if}
      <button
        onclick={handleExport}
        disabled={auditLog.length === 0}
        class="px-sm py-[2px] bg-canvas-elevated hover:bg-canvas-elevated/70 border border-hairline hover:border-ink-mute rounded-sm text-[10px] font-code text-ink-faint hover:text-ink uppercase transition disabled:opacity-40 disabled:pointer-events-none"
      >
        Export
      </button>
      <span class="px-sm py-[2px] bg-canvas-elevated border border-hairline rounded-sm text-[10px] font-code text-ink-faint uppercase">
        Append-Only
      </span>
    </div>
  </div>

  <div
    bind:this={logContainer}
    class="flex-1 min-h-[100px] overflow-y-auto p-md bg-canvas border border-hairline rounded-md font-code text-[11px] text-ink-mute space-y-sm scrollbar-thin scrollbar-thumb-canvas-elevated"
  >
    {#if auditLog.length === 0}
      <div class="text-ink-faint italic font-mono uppercase">Console Feed Idle...</div>
    {:else}
      {#each auditLog as log}
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
