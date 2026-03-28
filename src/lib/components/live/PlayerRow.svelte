<script lang="ts">
  import type { LivePlayerState } from "../../types/livegame.js";
  import { currentPatch } from "../../stores/champions.js";

  let { player }: { player: LivePlayerState } = $props();
  let patch = $derived($currentPatch);

  // Game Client API returns champion name as the Data Dragon key (e.g., "MissFortune")
  let champImg = $derived(
    `https://ddragon.leagueoflegends.com/cdn/${patch}/img/champion/${player.champion}.png`
  );

  let kda = $derived(
    player.deaths === 0
      ? "Perfect"
      : ((player.kills + player.assists) / player.deaths).toFixed(1)
  );
  let kdaColor = $derived(() => {
    const ratio = player.deaths === 0 ? 99 : (player.kills + player.assists) / player.deaths;
    if (ratio >= 5) return "var(--accent-gold)";
    if (ratio >= 3) return "var(--accent-green)";
    if (ratio >= 2) return "var(--text-primary)";
    return "var(--text-secondary)";
  });
</script>

<div
  class="flex items-center gap-2 rounded-lg px-2 py-1.5"
  style="background: {player.is_local_player ? 'var(--bg-tertiary)' : 'transparent'}"
>
  <!-- Champion Image + Level -->
  <div class="relative shrink-0">
    <div class="h-9 w-9 overflow-hidden rounded-lg" style="background: var(--bg-primary)">
      <img
        src={champImg}
        alt={player.champion}
        class="h-full w-full object-cover"
        onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'}
      />
    </div>
    <span
      class="absolute -bottom-0.5 -right-0.5 flex h-4 w-4 items-center justify-center rounded text-[8px] font-bold text-white"
      style="background: var(--bg-primary); border: 1px solid var(--border)"
    >
      {player.level}
    </span>
  </div>

  <!-- Name -->
  <div class="w-20 min-w-0">
    <div class="flex items-center gap-1">
      <span class="truncate text-xs font-medium" style="color: var(--text-primary)">
        {player.champion}
      </span>
    </div>
    {#if player.is_local_player}
      <span class="text-[9px] font-bold" style="color: var(--accent-blue)">YOU</span>
    {:else}
      <span class="truncate text-[9px]" style="color: var(--text-muted)">{player.name}</span>
    {/if}
  </div>

  <!-- KDA -->
  <div class="w-20 text-center">
    <div class="text-xs font-bold" style="color: var(--text-primary)">
      <span style="color: var(--accent-green)">{player.kills}</span>
      <span style="color: var(--text-muted)">/</span>
      <span style="color: var(--accent-red)">{player.deaths}</span>
      <span style="color: var(--text-muted)">/</span>
      <span>{player.assists}</span>
    </div>
    <div class="text-[9px]" style="color: {kdaColor()}">{kda} KDA</div>
  </div>

  <!-- CS -->
  <div class="w-10 text-center">
    <div class="text-xs font-medium" style="color: var(--text-primary)">{player.cs}</div>
    <div class="text-[8px]" style="color: var(--text-muted)">CS</div>
  </div>

  <!-- Items -->
  <div class="flex flex-1 flex-wrap gap-0.5">
    {#each player.items as item}
      {@const itemImg = `https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${item.item_id}.png`}
      <div class="h-6 w-6 overflow-hidden rounded" style="background: var(--bg-primary)" title={item.name}>
        <img
          src={itemImg}
          alt={item.name}
          class="h-full w-full object-cover"
          onerror={(e) => {
            const el = e.currentTarget as HTMLImageElement;
            el.style.display = 'none';
          }}
        />
      </div>
    {/each}
    {#if player.items.length === 0}
      <span class="text-[9px]" style="color: var(--text-muted)">No items</span>
    {/if}
  </div>
</div>
