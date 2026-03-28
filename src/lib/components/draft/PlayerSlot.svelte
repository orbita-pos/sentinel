<script lang="ts">
  import type { ChampSelectPlayer } from "../../types/champselect.js";
  import { championMap, getChampionName } from "../../stores/champions.js";

  let { player, side }: { player: ChampSelectPlayer; side: "ally" | "enemy" } = $props();

  const roleLabels: Record<string, string> = {
    TOP: "Top",
    JUNGLE: "Jungle",
    MIDDLE: "Mid",
    BOTTOM: "Bot",
    UTILITY: "Support",
  };

  let map = $derived($championMap);
  let champName = $derived(getChampionName(map, player.champion_id));
  let isHovering = $derived(player.champion_id > 0);
  let roleLabel = $derived(roleLabels[player.assigned_position] ?? (player.assigned_position || "?"));
  let borderColor = $derived(side === "ally" ? "var(--accent-blue)" : "var(--accent-red)");

  // First letter for the avatar
  let avatarLetter = $derived(champName ? champName[0] : "?");
</script>

<div
  class="flex items-center gap-3 rounded-lg border px-3 py-2"
  style="background: var(--bg-tertiary); border-color: {player.is_local_player ? borderColor : 'var(--border)'}; border-left: 3px solid {borderColor}"
>
  <!-- Champion avatar -->
  <div
    class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-sm font-bold"
    style="background: {isHovering ? 'var(--bg-primary)' : 'var(--bg-secondary)'}; color: {isHovering ? 'var(--text-primary)' : 'var(--text-muted)'}"
  >
    {avatarLetter}
  </div>

  <div class="min-w-0 flex-1">
    <div class="flex items-center gap-2">
      <span class="truncate text-sm font-medium" style="color: {isHovering ? 'var(--text-primary)' : 'var(--text-muted)'}">
        {isHovering ? champName : "Picking..."}
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
