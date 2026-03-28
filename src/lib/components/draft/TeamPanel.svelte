<script lang="ts">
  import type { ChampSelectPlayer } from "../../types/champselect.js";
  import PlayerSlot from "./PlayerSlot.svelte";

  let { players, side, label }: {
    players: ChampSelectPlayer[];
    side: "ally" | "enemy";
    label: string;
  } = $props();
</script>

<div class="flex flex-col gap-2">
  <h3 class="text-xs font-semibold uppercase tracking-wide" style="color: {side === 'ally' ? 'var(--accent-blue)' : 'var(--accent-red)'}">
    {label}
  </h3>
  {#each players as player (player.cell_id)}
    <PlayerSlot {player} {side} />
  {:else}
    <div class="rounded-lg border px-3 py-4 text-center text-xs" style="background: var(--bg-tertiary); border-color: var(--border); color: var(--text-muted)">
      Waiting for players...
    </div>
  {/each}
</div>
