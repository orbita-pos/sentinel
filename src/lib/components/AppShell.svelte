<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import StatusBar from "./StatusBar.svelte";
  import { currentRoute } from "../stores/router.js";
  import { gamePhase } from "../stores/gameflow.js";
  import Dashboard from "../../routes/Dashboard.svelte";
  import Settings from "../../routes/Settings.svelte";
  import MatchHistory from "../../routes/MatchHistory.svelte";
  import ChampSelect from "../../routes/ChampSelect.svelte";
  import LiveGame from "../../routes/LiveGame.svelte";
  import Patterns from "../../routes/Patterns.svelte";
  import ImprovementTracker from "../../routes/ImprovementTracker.svelte";
  import PostGame from "../../routes/PostGame.svelte";

  let route = $derived($currentRoute);
  let phase = $derived($gamePhase);

  // Track previous phase to detect transitions
  let prevPhase = $state("None");
  let postGameMatchId = $state("");

  // Auto-navigate based on game phase transitions
  $effect(() => {
    if (phase !== prevPhase) {
      if (phase === "ChampSelect") {
        currentRoute.set("champ-select");
      } else if (phase === "InProgress" || phase === "GameStart") {
        currentRoute.set("live-game");
      } else if (phase === "EndOfGame") {
        currentRoute.set("dashboard");
      } else if (
        (prevPhase === "ChampSelect" || prevPhase === "InProgress") &&
        phase === "None"
      ) {
        currentRoute.set("dashboard");
      }
      prevPhase = phase;
    }
  });

  // Detect when user navigated away from an active game phase
  let activePhaseRoute = $derived(
    phase === "ChampSelect" ? "champ-select"
    : (phase === "InProgress" || phase === "GameStart") ? "live-game"
    : null
  );
  let showReturnBanner = $derived(activePhaseRoute !== null && route !== activePhaseRoute);

  let bannerLabel = $derived(
    phase === "ChampSelect" ? "Champion Select"
    : (phase === "InProgress" || phase === "GameStart") ? "Live Game"
    : ""
  );

  function returnToActivePhase() {
    if (activePhaseRoute) {
      currentRoute.set(activePhaseRoute as any);
    }
  }
</script>

<div class="flex h-screen w-screen flex-col">
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />

    <!-- Main content area -->
    <div class="flex flex-1 flex-col overflow-hidden" style="background: var(--bg-primary)">
      <!-- Return banner when navigated away from active game phase -->
      {#if showReturnBanner}
        <button
          onclick={returnToActivePhase}
          class="flex items-center justify-center gap-2 border-b px-4 py-2 text-sm font-medium transition-colors"
          style="background: var(--accent-blue); color: white; border-color: transparent"
          onmouseenter={(e) => (e.currentTarget as HTMLElement).style.opacity = '0.9'}
          onmouseleave={(e) => (e.currentTarget as HTMLElement).style.opacity = '1'}
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
            <path stroke-linecap="round" stroke-linejoin="round" d="M11 15l-3-3m0 0l3-3m-3 3h8M3 12a9 9 0 1118 0 9 9 0 01-18 0z" />
          </svg>
          Return to {bannerLabel}
        </button>
      {/if}

      <main class="flex-1 overflow-y-auto p-6">
        {#if route === "dashboard"}
          <Dashboard />
        {:else if route === "settings"}
          <Settings />
        {:else if route === "match-history"}
          <MatchHistory />
        {:else if route === "champ-select"}
          <ChampSelect />
        {:else if route === "live-game"}
          <LiveGame />
        {:else if route === "patterns"}
          <Patterns />
        {:else if route === "improvement"}
          <ImprovementTracker />
        {:else if route === "post-game"}
          <PostGame matchId={postGameMatchId} />
        {:else}
          <div class="flex h-full items-center justify-center">
            <p style="color: var(--text-muted)">Unknown route</p>
          </div>
        {/if}
      </main>
    </div>
  </div>

  <StatusBar />
</div>
