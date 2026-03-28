<script lang="ts">
  import { powerSpikeAlerts } from "../../stores/livegame.js";

  let alerts = $derived($powerSpikeAlerts);
</script>

{#if alerts.length > 0}
  <div class="fixed right-4 top-4 z-50 flex flex-col gap-2" style="max-width: 320px">
    {#each alerts as spike}
      <div
        class="animate-in slide-in-from-right rounded-lg border px-4 py-3 shadow-lg"
        style="background: var(--bg-secondary); border-color: var(--accent-gold)"
      >
        <div class="flex items-center gap-2">
          <span class="text-xs font-bold" style="color: var(--accent-gold)">POWER SPIKE</span>
        </div>
        <p class="mt-1 text-sm" style="color: var(--text-primary)">{spike.description}</p>
      </div>
    {/each}
  </div>
{/if}
