<script lang="ts">
  import type { ChampSelectPlayer } from "../../types/champselect.js";

  let { player, side }: { player: ChampSelectPlayer; side: "ally" | "enemy" } = $props();

  const roleLabels: Record<string, string> = {
    TOP: "Top",
    JUNGLE: "Jungle",
    MIDDLE: "Mid",
    BOTTOM: "Bot",
    UTILITY: "Support",
  };

  let champLabel = $derived(
    player.champion_id > 0 ? `Champion ${player.champion_id}` : "Picking..."
  );
  let roleLabel = $derived(roleLabels[player.assigned_position] ?? (player.assigned_position || "?"));
  let borderColor = $derived(side === "ally" ? "var(--accent-blue)" : "var(--accent-red)");
</script>

<div
  class="flex items-center gap-3 rounded-lg border px-3 py-2"
  style="background: var(--bg-tertiary); border-color: {player.is_local_player ? borderColor : 'var(--border)'}; border-left: 3px solid {borderColor}"
>
  <!-- Champion icon placeholder -->
  <div
    class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-sm font-bold"
    style="background: var(--bg-primary); color: {player.champion_id > 0 ? 'var(--text-primary)' : 'var(--text-muted)'}"
  >
    {#if player.champion_id > 0}
      {player.champion_id}
    {:else}
      ?
    {/if}
  </div>

  <div class="min-w-0 flex-1">
    <div class="flex items-center gap-2">
      <span class="truncate text-sm font-medium" style="color: var(--text-primary)">
        {champLabel}
      </span>
      {#if player.is_local_player}
        <span class="rounded px-1 py-0.5 text-[9px] font-bold" style="background: var(--accent-blue); color: white">
          YOU
        </span>
      {/if}
    </div>
    <div class="flex items-center gap-2 text-xs" style="color: var(--text-muted)">
      <span>{roleLabel}</span>
      {#if player.rank}
        <span style="color: var(--accent-gold)">{player.rank}</span>
      {/if}
    </div>
  </div>
</div>
