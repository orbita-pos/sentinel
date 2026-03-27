<script lang="ts">
  import type { MatchSummaryItem } from "../types/index.js";

  let { match: m }: { match: MatchSummaryItem } = $props();

  let kdaRatio = $derived(
    m.deaths === 0 ? "Perfect" : ((m.kills + m.assists) / m.deaths).toFixed(1)
  );
  let csPerMin = $derived(
    m.game_duration > 0
      ? (m.cs / (m.game_duration / 60)).toFixed(1)
      : "0"
  );
  let timeStr = $derived(() => {
    const min = Math.floor(m.game_duration / 60);
    const sec = m.game_duration % 60;
    return `${min}:${sec.toString().padStart(2, "0")}`;
  });
  let dateStr = $derived(
    new Date(m.game_creation).toLocaleDateString(undefined, { month: "short", day: "numeric" })
  );

  const roleIcons: Record<string, string> = {
    TOP: "TOP",
    JUNGLE: "JNG",
    MIDDLE: "MID",
    BOTTOM: "BOT",
    UTILITY: "SUP",
  };
</script>

<div
  class="flex items-center gap-4 rounded-lg border px-4 py-3 transition-colors"
  style="background: var(--bg-secondary); border-color: var(--border); border-left: 3px solid {m.win ? 'var(--accent-blue)' : 'var(--accent-red)'}"
>
  <!-- Champion + Result -->
  <div class="flex w-24 flex-col items-center gap-1">
    <span class="text-xs font-medium truncate w-full text-center" style="color: var(--text-primary)">{m.champion_name}</span>
    <span class="text-[10px] font-bold" style="color: {m.win ? 'var(--accent-blue)' : 'var(--accent-red)'}">
      {m.win ? "WIN" : "LOSS"}
    </span>
  </div>

  <!-- KDA -->
  <div class="flex w-28 flex-col items-center">
    <span class="text-sm font-semibold" style="color: var(--text-primary)">
      {m.kills}/{m.deaths}/{m.assists}
    </span>
    <span class="text-[10px]" style="color: var(--text-muted)">
      {kdaRatio} KDA
    </span>
  </div>

  <!-- CS -->
  <div class="flex w-16 flex-col items-center">
    <span class="text-sm" style="color: var(--text-primary)">{m.cs}</span>
    <span class="text-[10px]" style="color: var(--text-muted)">{csPerMin}/min</span>
  </div>

  <!-- Gold -->
  <div class="flex w-16 flex-col items-center">
    <span class="text-sm" style="color: var(--accent-gold)">{(m.gold / 1000).toFixed(1)}k</span>
    <span class="text-[10px]" style="color: var(--text-muted)">gold</span>
  </div>

  <!-- Role -->
  {#if m.role}
    <div class="w-10 text-center">
      <span class="rounded px-1.5 py-0.5 text-[10px] font-medium" style="background: var(--bg-tertiary); color: var(--text-secondary)">
        {roleIcons[m.role] ?? m.role}
      </span>
    </div>
  {/if}

  <!-- Spacer + meta -->
  <div class="ml-auto flex flex-col items-end">
    <span class="text-xs" style="color: var(--text-muted)">{timeStr()}</span>
    <span class="text-[10px]" style="color: var(--text-muted)">{dateStr}</span>
  </div>
</div>
