<script lang="ts">
  import { onMount } from 'svelte';
  import './layout.css';

  let { children } = $props();

  onMount(() => {
    document.documentElement.classList.add('dark');
    document.documentElement.style.colorScheme = 'dark';
    const obs = new MutationObserver(() => {
      document.documentElement.classList.add('dark');
      document.documentElement.style.colorScheme = 'dark';
    });
    obs.observe(document.documentElement, { attributes: true, attributeFilter: ['class', 'style'] });
    return () => obs.disconnect();
  });
</script>

<div class="min-h-screen bg-canvas text-ink font-body antialiased selection:bg-primary/30 selection:text-ink">
  <div class="fixed bottom-xs right-xs text-micro text-ink-mute opacity-60 pointer-events-none select-none z-50">
    TECHNOLOGY DEMONSTRATION • NOT FOR DEPLOYMENT
  </div>
  {@render children()}
</div>
