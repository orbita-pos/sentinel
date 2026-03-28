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
  let totalCS = $derived(players.reduce((s, p) => s + p.cs, 0));
</script>

<div class="rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
  <!-- Header -->
  <div class="flex items-center justify-between border-b px-4 py-2.5" style="border-color: var(--border)">
    <span class="text-xs font-bold uppercase tracking-wide" style="color: {color}">{label}</span>
    <div class="flex items-center gap-3">
      <span class="text-xs font-semibold" style="color: var(--text-primary)">
        <span style="color: var(--accent-green)">{totalKills}</span>
        <span style="color: var(--text-muted)">/</span>
        <span style="color: var(--accent-red)">{totalDeaths}</span>
        <span style="color: var(--text-muted)">/</span>
        {totalAssists}
      </span>
      <span class="text-[10px]" style="color: var(--text-muted)">{totalCS} CS</span>
    </div>
  </div>

  <!-- Column headers -->
  <div class="flex items-center gap-2 px-2 py-1 text-[9px] font-medium uppercase tracking-wide" style="color: var(--text-muted)">
    <div style="width: 36px"></div>
    <div class="w-20">Champ</div>
    <div class="w-20 text-center">KDA</div>
    <div class="w-10 text-center">CS</div>
    <div class="flex-1">Items</div>
  </div>

  <!-- Players -->
  <div class="space-y-0.5 px-1 pb-1">
    {#each players as player (player.name)}
      <PlayerRow {player} />
    {/each}
  </div>
</div>
