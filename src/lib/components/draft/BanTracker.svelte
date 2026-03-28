<script lang="ts">
  import { championMap, currentPatch, getChampionName, getChampionImageUrl } from "../../stores/champions.js";

  let { bans }: { bans: number[] } = $props();
  let map = $derived($championMap);
  let patch = $derived($currentPatch);
</script>

{#if bans.length > 0}
  <div class="rounded-lg border px-4 py-3" style="background: var(--bg-secondary); border-color: var(--border)">
    <p class="mb-2 text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)">
      Bans ({bans.length})
    </p>
    <div class="flex flex-wrap gap-2">
      {#each bans as ban}
        {@const img = getChampionImageUrl(map, ban, patch)}
        <div class="flex items-center gap-1.5 rounded-md px-2 py-1" style="background: var(--bg-tertiary)">
          {#if img}
            <img src={img} alt="" class="h-5 w-5 rounded" />
          {/if}
          <span class="text-xs font-medium" style="color: var(--accent-red)">
            {getChampionName(map, ban)}
          </span>
        </div>
      {/each}
    </div>
  </div>
{/if}
