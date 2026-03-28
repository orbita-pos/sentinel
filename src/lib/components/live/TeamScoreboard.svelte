<script lang="ts">
  import type { LivePlayerState } from "../../types/livegame.js";
  import PlayerRow from "./PlayerRow.svelte";

  let { players, label, color }: {
    players: LivePlayerState[];
    label: string;
    color: string;
  } = $props();

  let totalKills = $derived(players.reduce((s, p) => s + p.kills, 0));
  let totalDeaths = $derived(players.reduce((s, p) => s + p.deaths, 0));
  let totalAssists = $derived(players.reduce((s, p) => s + p.assists, 0));
</script>

<div class="rounded-lg border" style="background: var(--bg-secondary); border-color: var(--border)">
  <div class="flex items-center justify-between border-b px-3 py-2" style="border-color: var(--border)">
    <span class="text-xs font-semibold uppercase tracking-wide" style="color: {color}">{label}</span>
    <span class="text-xs font-medium" style="color: var(--text-secondary)">
      {totalKills}/{totalDeaths}/{totalAssists}
    </span>
  </div>
  <!-- Header -->
  <div class="flex items-center gap-2 px-2 py-1 text-[10px] font-medium" style="color: var(--text-muted)">
    <div class="w-28">Champion</div>
    <div class="w-20 text-center">KDA</div>
    <div class="w-10 text-center">CS</div>
    <div class="flex-1">Items</div>
  </div>
  <!-- Players -->
  {#each players as player (player.name)}
    <PlayerRow {player} />
  {/each}
</div>
