<script lang="ts">
  import type { GoldDiffPoint } from "../../types/livegame.js";

  let { history, currentDiff }: { history: GoldDiffPoint[]; currentDiff: number } = $props();

  const WIDTH = 500;
  const HEIGHT = 120;
  const PADDING = 4;

  let pathData = $derived(() => {
    if (history.length < 2) return "";

    const maxTime = history[history.length - 1].time;
    const minTime = history[0].time;
    const timeRange = maxTime - minTime || 1;

    // Find max absolute diff for Y scaling
    const maxAbsDiff = Math.max(
      Math.abs(Math.max(...history.map((p) => p.diff))),
      Math.abs(Math.min(...history.map((p) => p.diff))),
      500 // minimum scale
    );

    const scaleX = (t: number) =>
      PADDING + ((t - minTime) / timeRange) * (WIDTH - 2 * PADDING);
    const scaleY = (d: number) =>
      HEIGHT / 2 - (d / maxAbsDiff) * (HEIGHT / 2 - PADDING);

    let d = `M ${scaleX(history[0].time)} ${scaleY(history[0].diff)}`;
    for (let i = 1; i < history.length; i++) {
      d += ` L ${scaleX(history[i].time)} ${scaleY(history[i].diff)}`;
    }
    return d;
  });

  let diffColor = $derived(currentDiff >= 0 ? "var(--accent-blue)" : "var(--accent-red)");
  let diffLabel = $derived(() => {
    const sign = currentDiff >= 0 ? "+" : "";
    return `${sign}${(currentDiff / 1000).toFixed(1)}k`;
  });
</script>

<div class="rounded-lg border p-3" style="background: var(--bg-secondary); border-color: var(--border)">
  <div class="mb-2 flex items-center justify-between">
    <span class="text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)">Gold Advantage</span>
    <span class="text-lg font-bold" style="color: {diffColor}">{diffLabel()}</span>
  </div>
  <svg viewBox="0 0 {WIDTH} {HEIGHT}" class="w-full" style="height: 120px">
    <!-- Center line -->
    <line x1={PADDING} y1={HEIGHT / 2} x2={WIDTH - PADDING} y2={HEIGHT / 2}
      stroke="var(--border)" stroke-width="1" stroke-dasharray="4" />
    <!-- Graph line -->
    {#if pathData()}
      <path d={pathData()} fill="none" stroke={diffColor} stroke-width="2" />
    {/if}
  </svg>
</div>
