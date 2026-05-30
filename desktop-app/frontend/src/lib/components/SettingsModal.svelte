<script lang="ts">
  import { updateSettings } from '../tauri';

  let {
    show = false,
    piIp = '127.0.0.1',
    piPort = 8080,
    localPort = 8081,
    onclose = (_saved: boolean) => {},
  } = $props<{
    show: boolean;
    piIp: string;
    piPort: number;
    localPort: number;
    onclose: (saved: boolean) => void;
  }>();

  let editPiIp = $state('');
  let editPiPort = $state(8080);
  let editLocalPort = $state(8081);
  let saveError = $state('');
  let saveSuccess = $state(false);

  $effect(() => {
    if (show) {
      editPiIp = piIp;
      editPiPort = piPort;
      editLocalPort = localPort;
    }
  });

  function validateIp(ip: string): boolean {
    const ipv4 = /^(\d{1,3}\.){3}\d{1,3}$/;
    const ipv6 = /^[0-9a-fA-F:]+$/;
    if (!ipv4.test(ip) && !ipv6.test(ip)) return false;
    if (ipv4.test(ip)) {
      return ip.split('.').every(octet => parseInt(octet) <= 255);
    }
    return true;
  }

  function validatePort(port: number): boolean {
    return Number.isInteger(port) && port >= 1024 && port <= 65535;
  }

  let ipError = $derived(editPiIp.length > 0 && !validateIp(editPiIp) ? 'Invalid IP address' : '');
  let piPortError = $derived(!validatePort(editPiPort) ? 'Port must be 1024-65535' : '');
  let localPortError = $derived(!validatePort(editLocalPort) ? 'Port must be 1024-65535' : '');
  let canSave = $derived(!ipError && !piPortError && !localPortError);

  async function handleSave() {
    if (!canSave) return;
    saveError = '';
    saveSuccess = false;
    try {
      await updateSettings(editPiIp, editPiPort, editLocalPort);
      saveSuccess = true;
      setTimeout(() => onclose(true), 800);
    } catch (e: unknown) {
      saveError = e instanceof Error ? e.message : 'Failed to save settings';
    }
  }

  function handleClose() {
    onclose(false);
  }
</script>

{#if show}
  <!-- svelte-ignore a11y_interactive_supports_focus -->
  <div
    role="dialog" aria-modal="true" aria-label="Connection settings"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/70 backdrop-blur-sm"
    onclick={handleClose}
    onkeydown={(e) => { if (e.key === 'Escape') handleClose(); }}
  >
    <div
      role="none"
      class="bg-canvas-soft border border-hairline rounded-xl shadow-lvl2 w-full max-w-[480px] mx-4"
      onclick={(e) => e.stopPropagation()}
    >
      <div class="flex items-center justify-between px-6 py-4 border-b border-hairline">
        <h2 class="text-sm font-mono uppercase tracking-[0.08em] text-ink-mute font-semibold">Connection Settings</h2>
        <button
          onclick={handleClose}
          class="text-xs font-mono text-ink-faint border border-hairline px-2 py-[2px] rounded hover:text-ink hover:border-ink-mute transition-colors"
        >
          ESC
        </button>
      </div>

      <div class="px-6 py-5 space-y-4">
        <div>
          <label for="pi-ip-input" class="text-xs font-mono uppercase tracking-wide text-ink-faint block mb-1.5">Pi IP Address</label>
          <input
            id="pi-ip-input"
            bind:value={editPiIp}
            class="w-full bg-canvas border border-hairline rounded-lg px-4 py-3 text-sm font-mono text-ink placeholder:text-ink-faint outline-none focus:border-primary/50 transition-colors"
            placeholder="127.0.0.1"
          />
          {#if ipError}
            <p class="text-[10px] font-code text-status-error mt-1">{ipError}</p>
          {/if}
        </div>

        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="pi-port-input" class="text-xs font-mono uppercase tracking-wide text-ink-faint block mb-1.5">Pi Port</label>
            <input
              id="pi-port-input"
              type="number"
              bind:value={editPiPort}
              class="w-full bg-canvas border border-hairline rounded-lg px-4 py-3 text-sm font-mono text-ink outline-none focus:border-primary/50 transition-colors"
            />
            {#if piPortError}
              <p class="text-[10px] font-code text-status-error mt-1">{piPortError}</p>
            {/if}
          </div>
          <div>
            <label for="local-port-input" class="text-xs font-mono uppercase tracking-wide text-ink-faint block mb-1.5">Local Port</label>
            <input
              id="local-port-input"
              type="number"
              bind:value={editLocalPort}
              class="w-full bg-canvas border border-hairline rounded-lg px-4 py-3 text-sm font-mono text-ink outline-none focus:border-primary/50 transition-colors"
            />
            {#if localPortError}
              <p class="text-[10px] font-code text-status-error mt-1">{localPortError}</p>
            {/if}
          </div>
        </div>
      </div>

      {#if saveError}
        <div class="mx-6 mb-2 px-4 py-2.5 bg-status-error/10 border border-status-error/30 text-status-error text-sm font-mono rounded-lg">
          {saveError}
        </div>
      {/if}

      {#if saveSuccess}
        <div class="mx-6 mb-2 px-4 py-2.5 bg-primary/10 border border-primary/30 text-primary text-sm font-mono rounded-lg">
          Settings saved. Restart connection to apply.
        </div>
      {/if}

      <div class="flex justify-end gap-3 px-6 py-4 border-t border-hairline">
        <button
          onclick={handleClose}
          class="px-5 py-2.5 text-sm font-mono text-ink-mute border border-hairline rounded-lg hover:text-ink hover:border-ink-mute transition-colors"
        >
          Cancel
        </button>
        <button
          onclick={handleSave}
          disabled={!canSave}
          class="px-5 py-2.5 text-sm font-mono font-bold text-primary-on bg-primary rounded-lg hover:opacity-90 transition-opacity disabled:opacity-40 disabled:cursor-not-allowed"
        >
          Save Settings
        </button>
      </div>
    </div>
  </div>
{/if}
