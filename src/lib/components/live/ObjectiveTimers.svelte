<script lang="ts">
  import type { LiveEvent } from "../../types/livegame.js";

  let { events, gameTime }: { events: LiveEvent[]; gameTime: number } = $props();

  const CDN = "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-match-history/global/default";

  let recentObjectives = $derived(events.slice(-6).reverse());

  let dragonCount = $derived(events.filter((e) => e.event_name === "DragonKill").length);
  let baronCount = $derived(events.filter((e) => e.event_name === "BaronKill").length);
  let heraldCount = $derived(events.filter((e) => e.event_name === "HeraldKill").length);
  let turretCount = $derived(events.filter((e) => e.event_name === "TurretKilled").length);
  let inhibCount = $derived(events.filter((e) => e.event_name === "InhibKilled").length);

  const objInfo: Record<string, { img: string; color: string; bg: string; label: string }> = {
    DragonKill: { img: `${CDN}/dragon-100.png`, color: "var(--accent-purple)", bg: "rgba(168, 85, 247, 0.12)", label: "Dragon" },
    BaronKill: { img: `${CDN}/baron-100.png`, color: "var(--accent-gold)", bg: "rgba(234, 179, 8, 0.12)", label: "Baron Nashor" },
    HeraldKill: { img: `${CDN}/herald-100.png`, color: "var(--accent-blue)", bg: "rgba(59, 130, 246, 0.12)", label: "Rift Herald" },
    TurretKilled: { img: `${CDN}/tower-100.png`, color: "var(--accent-red)", bg: "rgba(239, 68, 68, 0.12)", label: "Turret" },
    InhibKilled: { img: `${CDN}/inhibitor-100.png`, color: "var(--accent-red)", bg: "rgba(239, 68, 68, 0.12)", label: "Inhibitor" },
  };

  function formatTime(seconds: number): string {
    const min = Math.floor(seconds / 60);
    const sec = Math.floor(seconds % 60);
    return `${min}:${sec.toString().padStart(2, "0")}`;
  }

  function timeAgo(eventTime: number): string {
    const diff = gameTime - eventTime;
    if (diff < 60) return `${Math.floor(diff)}s ago`;
    return `${Math.floor(diff / 60)}m ago`;
  }
</script>

<div class="rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
  <!-- Header with icon counters -->
  <div class="flex items-center justify-between border-b px-4 py-2.5" style="border-color: var(--border)">
    <span class="text-xs font-bold uppercase tracking-wide" style="color: var(--text-muted)">Objectives</span>
    <div class="flex items-center gap-3">
      {#if dragonCount > 0}
        <div class="flex items-center gap-1">
          <img src="{CDN}/dragon-100.png" alt="Dragon" class="h-4 w-4" />
          <span class="text-xs font-bold" style="color: var(--accent-purple)">{dragonCount}</span>
        </div>
      {/if}
      {#if baronCount > 0}
        <div class="flex items-center gap-1">
          <img src="{CDN}/baron-100.png" alt="Baron" class="h-4 w-4" />
          <span class="text-xs font-bold" style="color: var(--accent-gold)">{baronCount}</span>
        </div>
      {/if}
      {#if heraldCount > 0}
        <div class="flex items-center gap-1">
          <img src="{CDN}/herald-100.png" alt="Herald" class="h-4 w-4" />
          <span class="text-xs font-bold" style="color: var(--accent-blue)">{heraldCount}</span>
        </div>
      {/if}
      {#if turretCount > 0}
        <div class="flex items-center gap-1">
          <img src="{CDN}/tower-100.png" alt="Turret" class="h-4 w-4" />
          <span class="text-xs font-bold" style="color: var(--accent-red)">{turretCount}</span>
        </div>
      {/if}
      {#if inhibCount > 0}
        <div class="flex items-center gap-1">
          <img src="{CDN}/inhibitor-100.png" alt="Inhib" class="h-4 w-4" />
          <span class="text-xs font-bold" style="color: var(--accent-red)">{inhibCount}</span>
        </div>
      {/if}
    </div>
  </div>

  <!-- Timeline -->
  <div class="p-3">
    {#if recentObjectives.length === 0}
      <div class="flex h-20 items-center justify-center">
        <p class="text-xs" style="color: var(--text-muted)">No objectives taken yet</p>
      </div>
    {:else}
      <div class="flex flex-col gap-2">
        {#each recentObjectives as event}
          {@const info = objInfo[event.event_name]}
          <div class="flex items-center gap-3 rounded-lg px-3 py-2" style="background: {info?.bg ?? 'var(--bg-tertiary)'}">
            <div class="flex h-9 w-9 shrink-0 items-center justify-center rounded-lg" style="background: {info?.bg ?? 'var(--bg-primary)'}">
              {#if info?.img}
                <img src={info.img} alt={info.label} class="h-6 w-6" />
              {:else}
                <div class="h-3 w-3 rounded-full" style="background: var(--text-muted)"></div>
              {/if}
            </div>
            <div class="flex-1 min-w-0">
              <span class="text-xs font-semibold" style="color: {info?.color ?? 'var(--text-secondary)'}">
                {info?.label ?? event.event_name}
              </span>
              <span class="ml-2 font-mono text-[10px]" style="color: var(--text-muted)">
                {formatTime(event.event_time)}
              </span>
            </div>
            <span class="text-[10px] shrink-0" style="color: var(--text-muted)">{timeAgo(event.event_time)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
