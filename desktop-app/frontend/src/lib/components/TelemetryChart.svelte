<script lang="ts">
  import { onMount } from 'svelte';

  let {
    data = [] as number[],
    color = '#3ecf8e',
    height = 40,
    min = 0,
    max = 100,
    label = '',
  } = $props<{
    data: number[];
    color?: string;
    height?: number;
    min?: number;
    max?: number;
    label?: string;
  }>();

  let canvas: HTMLCanvasElement | null = $state(null);
  let prevData = '';

  $effect(() => {
    const key = data.join(',');
    if (key === prevData || !canvas) return;
    prevData = key;
    draw();
  });

  function draw() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    const w = canvas.offsetWidth * dpr;
    const h = height * dpr;
    canvas.width = w;
    canvas.height = h;
    ctx.scale(dpr, dpr);

    const cw = canvas.offsetWidth;
    const ch = height;

    ctx.clearRect(0, 0, cw, ch);

    if (data.length < 2) return;

    const range = max - min || 1;
    const stepX = cw / (data.length - 1);

    ctx.beginPath();
    ctx.strokeStyle = color;
    ctx.lineWidth = 1.5;
    ctx.lineJoin = 'round';
    ctx.lineCap = 'round';

    for (let i = 0; i < data.length; i++) {
      const x = i * stepX;
      const y = ch - ((data[i] - min) / range) * (ch - 4) - 2;
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    }
    ctx.stroke();

    ctx.fillStyle = color + '20';
    ctx.lineTo(cw, ch);
    ctx.lineTo(0, ch);
    ctx.closePath();
    ctx.fill();
  }

  onMount(() => {
    requestAnimationFrame(draw);
  });
</script>

<div class="relative">
  {#if label}
    <div class="text-[10px] font-code text-ink-faint uppercase mb-xs">{label}</div>
  {/if}
  <canvas bind:this={canvas} class="w-full rounded-sm" style="height: {height}px"></canvas>
</div>
