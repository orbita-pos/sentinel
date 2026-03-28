<script lang="ts">
  import { liveGameState } from "../lib/stores/livegame.js";
  import GameTimer from "../lib/components/live/GameTimer.svelte";
  import GoldDiffGraph from "../lib/components/live/GoldDiffGraph.svelte";
  import TeamScoreboard from "../lib/components/live/TeamScoreboard.svelte";
  import PowerSpikeAlert from "../lib/components/live/PowerSpikeAlert.svelte";
  import ObjectiveTimers from "../lib/components/live/ObjectiveTimers.svelte";

  let state = $derived($liveGameState);
</script>

<PowerSpikeAlert />

<div class="mx-auto max-w-5xl">
  <!-- Header -->
  <div class="mb-4 flex items-center justify-between">
    <div>
      <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Live Game</h2>
      <p class="mt-1 text-sm" style="color: var(--text-secondary)">
        {#if state}
          {state.game_mode} - Real-time companion
        {:else}
          Waiting for game to start...
        {/if}
      </p>
    </div>
    {#if state}
      <GameTimer gameTime={state.game_time} />
    {/if}
  </div>

  {#if !state}
    <div class="flex h-64 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="text-center">
        <p class="text-sm" style="color: var(--text-secondary)">Game not yet detected</p>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">Sentinel will auto-navigate here when the game starts and poll at 1Hz</p>
      </div>
    </div>
  {:else}
    <!-- Gold diff graph -->
    <div class="mb-4">
      <GoldDiffGraph history={state.gold_diff_history} currentDiff={state.team_gold_diff} />
    </div>

    <!-- Scoreboards side by side -->
    <div class="mb-4 grid grid-cols-2 gap-4">
      <TeamScoreboard
        players={state.my_team}
        label="Your Team"
        color="var(--accent-blue)"
      />
      <TeamScoreboard
        players={state.enemy_team}
        label="Enemy Team"
        color="var(--accent-red)"
      />
    </div>

    <!-- Bottom row: objectives + recent events -->
    <div class="grid grid-cols-2 gap-4">
      <ObjectiveTimers events={state.objective_events} gameTime={state.game_time} />

      <!-- Recent events -->
      <div class="rounded-lg border p-3" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="mb-2 text-xs font-semibold uppercase tracking-wide" style="color: var(--text-muted)">Recent Events</p>
        {#if state.recent_events.length === 0}
          <p class="text-xs" style="color: var(--text-muted)">No notable events yet</p>
        {:else}
          <div class="flex flex-col gap-1 max-h-32 overflow-y-auto">
            {#each state.recent_events.slice(-8).reverse() as event}
              <p class="text-xs" style="color: var(--text-secondary)">{event.description}</p>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>
