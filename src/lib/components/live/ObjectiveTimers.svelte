<script lang="ts">
  import type { LiveEvent } from "../../types/livegame.js";

  let { events, gameTime }: { events: LiveEvent[]; gameTime: number } = $props();

  // Show last 5 objective events
  let recentObjectives = $derived(events.slice(-5).reverse());

  const eventIcons: Record<string, string> = {
    DragonKill: "D",
    BaronKill: "B",
    HeraldKill: "H",
    TurretKilled: "T",
    InhibKilled: "I",
  };

  const eventColors: Record<string, string> = {
    DragonKill: "var(--accent-purple)",
    BaronKill: "var(--accent-gold)",
    HeraldKill: "var(--accent-blue)",
    TurretKilled: "var(--accent-red)",
    InhibKilled: "var(--accent-red)",
  };
</script>

<div class="rounded-lg border p-3" style="background: var(--bg-secondary); border-color: var(--border)">
  <p class="mb-2 text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)">Objectives</p>
  {#if recentObjectives.length === 0}
    <p class="text-xs" style="color: var(--text-muted)">No objectives taken yet</p>
  {:else}
    <div class="flex flex-col gap-1.5">
      {#each recentObjectives as event}
        <div class="flex items-center gap-2 text-xs">
          <span
            class="flex h-5 w-5 items-center justify-center rounded text-[10px] font-bold text-white"
            style="background: {eventColors[event.event_name] ?? 'var(--text-muted)'}"
          >
            {eventIcons[event.event_name] ?? "?"}
          </span>
          <span style="color: var(--text-secondary)">{event.description}</span>
        </div>
      {/each}
    </div>
  {/if}
</div>
