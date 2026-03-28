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

  // Track last match for post-game analysis
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
</script>

<div class="flex h-screen w-screen flex-col">
  <div class="flex flex-1 overflow-hidden">
    <Sidebar />

    <!-- Main content area -->
    <main class="flex-1 overflow-y-auto p-6" style="background: var(--bg-primary)">
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

  <StatusBar />
</div>
