<script lang="ts">
  import type { LiveEvent } from "../../types/livegame.js";

  let { events, gameTime }: { events: LiveEvent[]; gameTime: number } = $props();

  let recentObjectives = $derived(events.slice(-6).reverse());

  // Count objectives by type
  let dragonCount = $derived(events.filter((e) => e.event_name === "DragonKill").length);
  let baronCount = $derived(events.filter((e) => e.event_name === "BaronKill").length);
  let heraldCount = $derived(events.filter((e) => e.event_name === "HeraldKill").length);
  let turretCount = $derived(events.filter((e) => e.event_name === "TurretKilled").length);

  // SVG icons for each objective
  const icons: Record<string, { svg: string; color: string; bg: string; label: string }> = {
    DragonKill: {
      svg: "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 17.93c-3.95-.49-7-3.85-7-7.93 0-.62.08-1.21.21-1.79L9 15v1c0 1.1.9 2 2 2v1.93zm6.9-2.54c-.26-.81-1-1.39-1.9-1.39h-1v-3c0-.55-.45-1-1-1H8v-2h2c.55 0 1-.45 1-1V7h2c1.1 0 2-.9 2-2v-.41c2.93 1.19 5 4.06 5 7.41 0 2.08-.8 3.97-2.1 5.39z",
      color: "var(--accent-purple)",
      bg: "rgba(168, 85, 247, 0.15)",
      label: "Dragon",
    },
    BaronKill: {
      svg: "M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z",
      color: "var(--accent-gold)",
      bg: "rgba(234, 179, 8, 0.15)",
      label: "Baron",
    },
    HeraldKill: {
      svg: "M12 4.5C7 4.5 2.73 7.61 1 12c1.73 4.39 6 7.5 11 7.5s9.27-3.11 11-7.5c-1.73-4.39-6-7.5-11-7.5zM12 17c-2.76 0-5-2.24-5-5s2.24-5 5-5 5 2.24 5 5-2.24 5-5 5zm0-8c-1.66 0-3 1.34-3 3s1.34 3 3 3 3-1.34 3-3-1.34-3-3-3z",
      color: "var(--accent-blue)",
      bg: "rgba(59, 130, 246, 0.15)",
      label: "Herald",
    },
    TurretKilled: {
      svg: "M7 2v11h3v9l7-12h-4l4-8z",
      color: "var(--accent-red)",
      bg: "rgba(239, 68, 68, 0.15)",
      label: "Turret",
    },
    InhibKilled: {
      svg: "M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-2 10h-4v4h-2v-4H7v-2h4V7h2v4h4v2z",
      color: "var(--accent-red)",
      bg: "rgba(239, 68, 68, 0.15)",
      label: "Inhibitor",
    },
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
  <!-- Header with counters -->
  <div class="flex items-center justify-between border-b px-4 py-2.5" style="border-color: var(--border)">
    <span class="text-xs font-bold uppercase tracking-wide" style="color: var(--text-muted)">Objectives</span>
    <div class="flex items-center gap-3">
      {#if dragonCount > 0}
        <div class="flex items-center gap-1">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill={icons.DragonKill.color}>
            <path d={icons.DragonKill.svg} />
          </svg>
          <span class="text-xs font-bold" style="color: {icons.DragonKill.color}">{dragonCount}</span>
        </div>
      {/if}
      {#if baronCount > 0}
        <div class="flex items-center gap-1">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill={icons.BaronKill.color}>
            <path d={icons.BaronKill.svg} />
          </svg>
          <span class="text-xs font-bold" style="color: {icons.BaronKill.color}">{baronCount}</span>
        </div>
      {/if}
      {#if heraldCount > 0}
        <div class="flex items-center gap-1">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill={icons.HeraldKill.color}>
            <path d={icons.HeraldKill.svg} />
          </svg>
          <span class="text-xs font-bold" style="color: {icons.HeraldKill.color}">{heraldCount}</span>
        </div>
      {/if}
      {#if turretCount > 0}
        <div class="flex items-center gap-1">
          <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill={icons.TurretKilled.color}>
            <path d={icons.TurretKilled.svg} />
          </svg>
          <span class="text-xs font-bold" style="color: {icons.TurretKilled.color}">{turretCount}</span>
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
          {@const icon = icons[event.event_name]}
          <div class="flex items-center gap-3 rounded-lg px-3 py-2" style="background: {icon?.bg ?? 'var(--bg-tertiary)'}">
            <div
              class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg"
              style="background: {icon?.color ?? 'var(--text-muted)'}20"
            >
              <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill={icon?.color ?? 'var(--text-muted)'}>
                <path d={icon?.svg ?? "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2z"} />
              </svg>
            </div>
            <div class="flex-1 min-w-0">
              <span class="text-xs font-medium" style="color: {icon?.color ?? 'var(--text-secondary)'}">
                {icon?.label ?? event.event_name}
              </span>
              <span class="ml-2 text-[10px]" style="color: var(--text-muted)">
                {formatTime(event.event_time)}
              </span>
            </div>
            <span class="text-[10px]" style="color: var(--text-muted)">{timeAgo(event.event_time)}</span>
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
