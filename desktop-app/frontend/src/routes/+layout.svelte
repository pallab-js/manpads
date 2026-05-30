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
  {@render children()}
</div>
