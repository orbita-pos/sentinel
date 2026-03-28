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
      <div class="rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
        <div class="border-b px-4 py-2.5" style="border-color: var(--border)">
          <span class="text-xs font-bold uppercase tracking-wide" style="color: var(--text-muted)">Recent Events</span>
        </div>
        <div class="p-3">
          {#if state.recent_events.length === 0}
            <div class="flex h-20 items-center justify-center">
              <p class="text-xs" style="color: var(--text-muted)">No notable events yet</p>
            </div>
          {:else}
            <div class="flex flex-col gap-1.5 max-h-48 overflow-y-auto">
              {#each state.recent_events.slice(-10).reverse() as event}
                {@const isKill = event.event_name === "ChampionKill"}
                {@const isMulti = event.event_name === "Multikill"}
                {@const isAce = event.event_name === "Ace"}
                {@const isObjective = ["DragonKill", "BaronKill", "HeraldKill", "TurretKilled", "InhibKilled"].includes(event.event_name)}
                <div class="flex items-center gap-2 rounded-lg px-2.5 py-1.5" style="background: var(--bg-tertiary)">
                  <!-- Event icon -->
                  <div
                    class="flex h-6 w-6 shrink-0 items-center justify-center rounded"
                    style="background: {isAce ? 'var(--accent-gold)' : isMulti ? 'var(--accent-purple)' : isKill ? 'var(--bg-primary)' : isObjective ? 'var(--accent-blue)' : 'var(--bg-primary)'}; opacity: {isKill ? 0.8 : 1}"
                  >
                    {#if isKill}
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="var(--accent-red)" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    {:else if isMulti || isAce}
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="white">
                        <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                      </svg>
                    {:else}
                      <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="var(--text-muted)">
                        <circle cx="12" cy="12" r="4" />
                      </svg>
                    {/if}
                  </div>
                  <span
                    class="flex-1 text-xs"
                    style="color: {isAce ? 'var(--accent-gold)' : isMulti ? 'var(--accent-purple)' : 'var(--text-secondary)'}"
                  >
                    {event.description}
                  </span>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
