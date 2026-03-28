<script lang="ts">
  import type { MatchSummaryItem } from "../types/index.js";
  import { championMap, getChampionImageUrl, currentPatch } from "../stores/champions.js";
  import type { ChampionInfo } from "../stores/champions.js";

  let { match: m, onclick }: { match: MatchSummaryItem; onclick?: () => void } = $props();

  let map = $derived($championMap);
  let patch = $derived($currentPatch);

  // Resolve champion name -- DB may store "Champion123" from LCU, resolve to real name
  let champInfo: ChampionInfo | undefined = $derived(map[m.champion_id]);
  let champName = $derived(champInfo?.name ?? m.champion_name);
  let champImg = $derived(getChampionImageUrl(map, m.champion_id, patch));

  let kda = $derived(
    m.deaths === 0 ? "Perfect" : ((m.kills + m.assists) / m.deaths).toFixed(1)
  );
  let kdaColor = $derived(() => {
    const ratio = m.deaths === 0 ? 99 : (m.kills + m.assists) / m.deaths;
    if (ratio >= 5) return "var(--accent-gold)";
    if (ratio >= 3) return "var(--accent-green)";
    if (ratio >= 2) return "var(--text-primary)";
    return "var(--text-secondary)";
  });
  let csPerMin = $derived(
    m.game_duration > 0 ? (m.cs / (m.game_duration / 60)).toFixed(1) : "0"
  );
  let timeStr = $derived(() => {
    const min = Math.floor(m.game_duration / 60);
    const sec = m.game_duration % 60;
    return `${min}:${sec.toString().padStart(2, "0")}`;
  });
  let dateStr = $derived(
    new Date(m.game_creation).toLocaleDateString(undefined, { month: "short", day: "numeric" })
  );
  let timeAgo = $derived(() => {
    const diff = Date.now() - m.game_creation;
    const hours = Math.floor(diff / 3600000);
    if (hours < 1) return "just now";
    if (hours < 24) return `${hours}h ago`;
    const days = Math.floor(hours / 24);
    if (days < 7) return `${days}d ago`;
    return dateStr;
  });

  const roleLabels: Record<string, string> = {
    TOP: "Top", JUNGLE: "Jungle", MIDDLE: "Mid", BOTTOM: "Bot", UTILITY: "Support",
  };
  let roleLabel = $derived(m.role ? (roleLabels[m.role] ?? m.role) : "");

  const queueNames: Record<number, string> = {
    420: "Ranked Solo", 440: "Ranked Flex", 400: "Normal Draft",
    430: "Normal Blind", 450: "ARAM", 900: "URF", 1700: "Arena",
    830: "Co-op Intro", 840: "Co-op Beginner", 850: "Co-op Intermediate",
  };
  let queueName = $derived(queueNames[m.queue_id] ?? m.game_mode);
</script>

<button
  {onclick}
  class="group flex w-full items-center gap-4 rounded-xl border p-3 text-left transition-all"
  style="background: var(--bg-secondary); border-color: var(--border); border-left: 4px solid {m.win ? 'var(--accent-blue)' : 'var(--accent-red)'}"
  onmouseenter={(e) => (e.currentTarget as HTMLElement).style.background = 'var(--bg-hover)'}
  onmouseleave={(e) => (e.currentTarget as HTMLElement).style.background = 'var(--bg-secondary)'}
>
  <!-- Champion image -->
  <div class="relative shrink-0">
    <div class="h-14 w-14 overflow-hidden rounded-xl" style="background: var(--bg-tertiary)">
      {#if champImg}
        <img src={champImg} alt={champName} class="h-full w-full object-cover" />
      {:else}
        <div class="flex h-full w-full items-center justify-center text-lg font-bold" style="color: var(--text-muted)">
          {champName[0] ?? "?"}
        </div>
      {/if}
    </div>
    <!-- Win/Loss badge -->
    <div
      class="absolute -bottom-1 -right-1 rounded px-1.5 py-0.5 text-[9px] font-bold text-white"
      style="background: {m.win ? 'var(--accent-blue)' : 'var(--accent-red)'}"
    >
      {m.win ? "W" : "L"}
    </div>
  </div>

  <!-- Champion + Queue info -->
  <div class="flex min-w-0 flex-col gap-0.5" style="width: 110px">
    <span class="truncate text-sm font-semibold" style="color: var(--text-primary)">{champName}</span>
    <span class="text-[10px]" style="color: var(--text-muted)">{queueName}</span>
    {#if roleLabel}
      <span class="text-[10px]" style="color: var(--text-secondary)">{roleLabel}</span>
    {/if}
  </div>

  <!-- KDA -->
  <div class="flex flex-col items-center" style="width: 90px">
    <span class="text-sm font-bold" style="color: var(--text-primary)">
      {m.kills}<span style="color: var(--text-muted)">/</span>{m.deaths}<span style="color: var(--text-muted)">/</span>{m.assists}
    </span>
    <span class="text-[10px] font-medium" style="color: {kdaColor()}">
      {kda} KDA
    </span>
  </div>

  <!-- CS + Vision -->
  <div class="flex flex-col items-center" style="width: 70px">
    <span class="text-sm" style="color: var(--text-primary)">{m.cs} <span class="text-[10px]" style="color: var(--text-muted)">CS</span></span>
    <span class="text-[10px]" style="color: var(--text-muted)">{csPerMin}/min</span>
  </div>

  <!-- Gold -->
  <div class="flex flex-col items-center" style="width: 60px">
    <span class="text-sm font-medium" style="color: var(--accent-gold)">{(m.gold / 1000).toFixed(1)}k</span>
    <span class="text-[10px]" style="color: var(--text-muted)">gold</span>
  </div>

  <!-- Vision -->
  <div class="flex flex-col items-center" style="width: 50px">
    <span class="text-sm" style="color: var(--text-secondary)">{m.vision_score}</span>
    <span class="text-[10px]" style="color: var(--text-muted)">vision</span>
  </div>

  <!-- Time + Date -->
  <div class="ml-auto flex flex-col items-end gap-0.5">
    <span class="text-xs font-mono" style="color: var(--text-secondary)">{timeStr()}</span>
    <span class="text-[10px]" style="color: var(--text-muted)">{timeAgo()}</span>
  </div>

  <!-- Arrow hint -->
  <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 shrink-0 opacity-0 transition-opacity group-hover:opacity-100" fill="none" viewBox="0 0 24 24" stroke="var(--text-muted)" stroke-width="2">
    <path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
  </svg>
</button>
