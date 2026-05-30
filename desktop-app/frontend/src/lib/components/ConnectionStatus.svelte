<script lang="ts">
  let { isConnected = false, latencyMs = 0, piIp = '127.0.0.1' } = $props<{
    isConnected: boolean;
    latencyMs: number;
    piIp: string;
  }>();
</script>

<div class="flex items-center justify-between p-lg bg-canvas-soft border border-hairline rounded-lg shadow-lvl1">
  <div class="flex items-center space-x-md">
    <!-- Status Ring -->
    <div class="relative flex h-sm w-sm">
      {#if isConnected}
        <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-status-success opacity-75"></span>
        <span class="relative inline-flex rounded-full h-sm w-sm bg-status-success"></span>
      {:else}
        <span class="relative inline-flex rounded-full h-sm w-sm bg-status-error"></span>
      {/if}
    </div>

    <div>
      <div class="text-xs font-mono uppercase tracking-wider text-ink-mute">Control Link</div>
      <div class="text-body-md font-bold text-ink">
        {isConnected ? 'ONLINE' : 'DISCONNECTED'}
      </div>
    </div>
  </div>

  <div class="flex space-x-xl text-right font-code text-micro text-ink-mute2">
    <div>
      <div class="uppercase tracking-wider text-ink-faint">Latency</div>
      <div class="text-xs font-semibold text-ink-mute">
        {isConnected ? `${latencyMs}ms` : '--'}
      </div>
    </div>
    <div>
      <div class="uppercase tracking-wider text-ink-faint">Node Target</div>
      <div class="text-xs font-semibold text-ink-mute">
        {piIp}
      </div>
    </div>
  </div>
</div>
