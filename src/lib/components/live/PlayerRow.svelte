<script lang="ts">
  import type { LivePlayerState } from "../../types/livegame.js";

  let { player }: { player: LivePlayerState } = $props();

  let kda = $derived(
    player.deaths === 0
      ? "Perfect"
      : ((player.kills + player.assists) / player.deaths).toFixed(1)
  );
</script>

<div
  class="flex items-center gap-2 rounded px-2 py-1.5 text-xs"
  style="background: {player.is_local_player ? 'var(--bg-tertiary)' : 'transparent'}"
>
  <!-- Champion + Level -->
  <div class="w-28 truncate font-medium" style="color: var(--text-primary)">
    <span class="mr-1 text-[10px]" style="color: var(--text-muted)">{player.level}</span>
    {player.champion}
    {#if player.is_local_player}
      <span class="ml-1 text-[9px] font-bold" style="color: var(--accent-blue)">YOU</span>
    {/if}
  </div>

  <!-- KDA -->
  <div class="w-20 text-center">
    <span style="color: var(--text-primary)">{player.kills}/{player.deaths}/{player.assists}</span>
    <span class="ml-1" style="color: var(--text-muted)">({kda})</span>
  </div>

  <!-- CS -->
  <div class="w-10 text-center" style="color: var(--text-secondary)">{player.cs}</div>

  <!-- Items -->
  <div class="flex flex-1 flex-wrap gap-0.5">
    {#each player.items as item}
      <span
        class="rounded px-1 py-0.5 text-[9px]"
        style="background: var(--bg-primary); color: var(--text-muted)"
        title={item.name}
      >
        {item.name.length > 8 ? item.name.slice(0, 8) + "..." : item.name}
      </span>
    {/each}
  </div>
</div>
