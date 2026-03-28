<script lang="ts">
  import { liveGameState } from "../lib/stores/livegame.js";
  import GoldDiffGraph from "../lib/components/live/GoldDiffGraph.svelte";
  import TeamScoreboard from "../lib/components/live/TeamScoreboard.svelte";
  import PowerSpikeAlert from "../lib/components/live/PowerSpikeAlert.svelte";
  import ObjectiveTimers from "../lib/components/live/ObjectiveTimers.svelte";
  import PlayerTab from "../lib/components/live/PlayerTab.svelte";
  import { invoke } from "@tauri-apps/api/core";

  let state = $derived($liveGameState);
  let activeTab = $state("overview");

  function setTab(tab: string) {
    activeTab = tab;
  }

  let mobileUrl = $state("");
  let showMobileQR = $state(false);

  function openMiniOverlay() {
    invoke("open_mini_overlay").catch(e => console.error("Failed to open overlay:", e));
  }

  function togglePhoneCompanion() {
    if (showMobileQR) {
      showMobileQR = false;
      return;
    }
    // Start server on first click (opt-in, not auto-start)
    invoke<{ url: string }>("start_mobile_server")
      .then(r => { mobileUrl = r.url; showMobileQR = true; })
      .catch(e => console.error("Failed to start mobile server:", e));
  }

  // Community Dragon CDN for objective icons
  const CDN = "https://raw.communitydragon.org/latest/plugins/rcp-fe-lol-match-history/global/default";

  // All values from backend (pre-computed per tick in Rust)
  let blueKills = $derived(state?.my_team_kills ?? 0);
  let redKills = $derived(state?.enemy_team_kills ?? 0);
  let blueGold = $derived(state?.my_team_gold ?? 0);
  let redGold = $derived(state?.enemy_team_gold ?? 0);
  let goldDiff = $derived(state?.team_gold_diff ?? 0);
  let goldDiffStr = $derived(() => {
    const diff = Math.abs(goldDiff);
    const sign = goldDiff >= 0 ? "+" : "-";
    return `${sign}${(diff / 1000).toFixed(1)}k`;
  });

  // Game timer
  let timerStr = $derived(() => {
    if (!state) return "0:00";
    const min = Math.floor(state.game_time / 60);
    const sec = Math.floor(state.game_time % 60);
    return `${min}:${sec.toString().padStart(2, "0")}`;
  });

  // Objective counts from backend
  let dragonCount = $derived(state?.dragon_count ?? 0);
  let baronCount = $derived(state?.baron_count ?? 0);
  let heraldCount = $derived(state?.herald_count ?? 0);
  let turretCount = $derived(state?.turret_count ?? 0);

  // Event icons for recent events
  const eventImg: Record<string, string> = {
    DragonKill: `${CDN}/dragon-100.png`,
    BaronKill: `${CDN}/baron-100.png`,
    HeraldKill: `${CDN}/herald-100.png`,
    TurretKilled: `${CDN}/tower-100.png`,
    InhibKilled: `${CDN}/inhibitor-100.png`,
  };
</script>

<PowerSpikeAlert />

<div class="mx-auto max-w-5xl">
  {#if !state}
    <div class="flex h-64 flex-col items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <img src="/logo.png" alt="Sentinel" class="mb-4 h-16 w-16 rounded-xl opacity-40" />
      <p class="text-lg font-semibold" style="color: var(--text-secondary)">Waiting for game to start...</p>
      <p class="mt-2 text-xs" style="color: var(--text-muted)">Sentinel will auto-navigate here when the game starts</p>
    </div>
  {:else}
    <!-- ═══ ESPORTS SCOREBAR ═══ -->
    <div class="mb-4 rounded-xl border overflow-hidden" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="flex items-center justify-between px-4 py-3">
        <!-- Blue side -->
        <div class="flex items-center gap-4">
          <div class="flex items-center gap-2">
            <span class="text-xs font-bold uppercase" style="color: var(--accent-blue)">Blue</span>
            <span class="text-2xl font-black" style="color: var(--text-primary)">{blueKills}</span>
          </div>
          <div class="flex flex-col items-end">
            <span class="text-sm font-bold" style="color: var(--accent-gold)">{(blueGold / 1000).toFixed(1)}k</span>
            <span class="text-[9px]" style="color: var(--text-muted)">gold</span>
          </div>
          <!-- Blue objectives -->
          <div class="flex items-center gap-1.5">
            {#each Array(dragonCount) as _}
              <img src="{CDN}/dragon-100.png" alt="Dragon" class="h-5 w-5" />
            {/each}
            {#each Array(baronCount) as _}
              <img src="{CDN}/baron-100.png" alt="Baron" class="h-5 w-5" />
            {/each}
            {#each Array(heraldCount) as _}
              <img src="{CDN}/herald-100.png" alt="Herald" class="h-5 w-5" />
            {/each}
          </div>
        </div>

        <!-- Center: Timer + Gold Diff -->
        <div class="flex flex-col items-center">
          <span class="font-mono text-xl font-bold" style="color: var(--text-primary)">{timerStr()}</span>
          <div class="flex items-center gap-1.5">
            {#if goldDiff !== 0}
              <span
                class="rounded px-2 py-0.5 text-xs font-bold"
                style="background: {goldDiff >= 0 ? 'rgba(59, 130, 246, 0.2)' : 'rgba(239, 68, 68, 0.2)'}; color: {goldDiff >= 0 ? 'var(--accent-blue)' : 'var(--accent-red)'}"
              >
                {goldDiffStr()}
              </span>
            {:else}
              <span class="text-xs" style="color: var(--text-muted)">Even</span>
            {/if}
          </div>
        </div>

        <!-- Red side -->
        <div class="flex items-center gap-4">
          <!-- Red objectives (reversed) -->
          <div class="flex items-center gap-1.5">
            <!-- Enemy objectives would need separate tracking -- for now show turrets -->
            {#each Array(turretCount) as _}
              <img src="{CDN}/tower-100.png" alt="Turret" class="h-5 w-5 opacity-60" />
            {/each}
          </div>
          <div class="flex flex-col items-start">
            <span class="text-sm font-bold" style="color: var(--accent-gold)">{(redGold / 1000).toFixed(1)}k</span>
            <span class="text-[9px]" style="color: var(--text-muted)">gold</span>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-2xl font-black" style="color: var(--text-primary)">{redKills}</span>
            <span class="text-xs font-bold uppercase" style="color: var(--accent-red)">Red</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Tab Buttons -->
    <div class="mb-4 flex items-center gap-2">
      <button
        onclick={() => setTab("overview")}
        class="rounded-lg px-4 py-2 text-sm font-medium transition-colors"
        style="background: {activeTab === 'overview' ? 'var(--accent-blue)' : 'var(--bg-tertiary)'}; color: {activeTab === 'overview' ? 'white' : 'var(--text-secondary)'}"
      >Overview</button>
      <button
        onclick={() => setTab("player")}
        class="rounded-lg px-4 py-2 text-sm font-medium transition-colors"
        style="background: {activeTab === 'player' ? 'var(--accent-purple)' : 'var(--bg-tertiary)'}; color: {activeTab === 'player' ? 'white' : 'var(--text-secondary)'}"
      >Player Advice</button>

      <div class="ml-auto flex items-center gap-2">
        <button
          onclick={openMiniOverlay}
          class="flex items-center gap-1.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors"
          style="background: var(--bg-tertiary); color: var(--text-secondary)"
          title="Open floating mini window (stays on top of League)"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M4 8V4m0 0h4M4 4l5 5m11-1V4m0 0h-4m4 0l-5 5M4 16v4m0 0h4m-4 0l5-5m11 5v-4m0 4h-4m4 0l-5-5" />
          </svg>
          Mini Overlay
        </button>
        <button
          onclick={togglePhoneCompanion}
          class="flex items-center gap-1.5 rounded-lg px-3 py-2 text-sm font-medium transition-colors"
          style="background: {showMobileQR ? 'var(--accent-green)' : 'var(--bg-tertiary)'}; color: {showMobileQR ? 'white' : 'var(--text-secondary)'}"
          title="Open on your phone"
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M12 18h.01M8 21h8a2 2 0 002-2V5a2 2 0 00-2-2H8a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          Phone
        </button>
      </div>
    </div>

    <!-- Phone companion URL -->
    {#if showMobileQR && mobileUrl}
      <div class="mb-4 rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--accent-green)">
        <div class="flex items-center justify-between">
          <div>
            <p class="text-sm font-semibold" style="color: var(--text-primary)">Phone Companion</p>
            <p class="mt-1 text-xs" style="color: var(--text-secondary)">Open this URL on your phone (same WiFi network):</p>
            <p class="mt-2 select-text rounded-lg px-3 py-2 font-mono text-lg font-bold" style="background: var(--bg-primary); color: var(--accent-green); user-select: text">{mobileUrl}</p>
            <p class="mt-2 text-[10px]" style="color: var(--text-muted)">Phone and PC must be on the same WiFi. Updates every 2 seconds.</p>
          </div>
        </div>
      </div>
    {/if}

    {#if activeTab === "overview"}
      <!-- Gold diff graph -->
      <div class="mb-4">
        <GoldDiffGraph history={state.gold_diff_history} currentDiff={state.team_gold_diff} />
      </div>

      <!-- Scoreboards -->
      <div class="mb-4 grid grid-cols-2 gap-4">
        <TeamScoreboard players={state.my_team} label="Your Team" color="var(--accent-blue)" />
        <TeamScoreboard players={state.enemy_team} label="Enemy Team" color="var(--accent-red)" />
      </div>

      <!-- Bottom: Objectives + Events -->
      <div class="grid grid-cols-2 gap-4">
        <ObjectiveTimers events={state.objective_events} gameTime={state.game_time} />

        <!-- Recent Events -->
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
                  {@const objImg = eventImg[event.event_name]}
                  <div class="flex items-center gap-2.5 rounded-lg px-2.5 py-1.5" style="background: var(--bg-tertiary)">
                    {#if objImg}
                      <img src={objImg} alt={event.event_name} class="h-6 w-6 shrink-0" />
                    {:else if isKill}
                      <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded" style="background: var(--bg-primary)">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="none" stroke="var(--accent-red)" stroke-width="2.5">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </div>
                    {:else if isMulti || isAce}
                      <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded" style="background: {isAce ? 'var(--accent-gold)' : 'var(--accent-purple)'}">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-3.5 w-3.5" viewBox="0 0 24 24" fill="white">
                          <path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z" />
                        </svg>
                      </div>
                    {:else}
                      <div class="flex h-6 w-6 shrink-0 items-center justify-center rounded" style="background: var(--bg-primary)">
                        <div class="h-2 w-2 rounded-full" style="background: var(--text-muted)"></div>
                      </div>
                    {/if}
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
    {:else}
      <!-- Player Advice Tab -->
      {#if state}
        <PlayerTab gameState={state} />
      {:else}
        <p class="text-sm" style="color: var(--text-muted)">Loading player data...</p>
      {/if}
    {/if}
  {/if}
</div>
