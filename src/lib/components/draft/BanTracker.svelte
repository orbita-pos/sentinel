<script lang="ts">
  import { championMap, getChampionName } from "../../stores/champions.js";

  let { bans }: { bans: number[] } = $props();
  let map = $derived($championMap);
</script>

{#if bans.length > 0}
  <div class="rounded-lg border px-4 py-3" style="background: var(--bg-secondary); border-color: var(--border)">
    <p class="mb-2 text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)">
      Bans ({bans.length})
    </p>
    <div class="flex flex-wrap gap-2">
      {#each bans as ban}
        <span class="rounded-md px-2 py-1 text-xs font-medium" style="background: var(--bg-tertiary); color: var(--accent-red)">
          {getChampionName(map, ban)}
        </span>
      {/each}
    </div>
  </div>
{/if}
