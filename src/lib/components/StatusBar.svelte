<script lang="ts">
  import ConnectionIndicator from "./ConnectionIndicator.svelte";
  import { gamePhase } from "../stores/gameflow.js";
  import { currentSummoner } from "../stores/connection.js";

  let phase = $derived($gamePhase);
  let summoner = $derived($currentSummoner);

  const phaseLabels: Record<string, string> = {
    None: "Idle",
    Lobby: "In Lobby",
    Matchmaking: "In Queue",
    ReadyCheck: "Ready Check",
    ChampSelect: "Champion Select",
    GameStart: "Game Starting",
    InProgress: "In Game",
    WaitingForStats: "Loading Stats",
    PreEndOfGame: "Game Ending",
    EndOfGame: "Post Game",
    Reconnect: "Reconnecting",
  };
</script>

<footer class="flex h-8 items-center justify-between border-t px-4" style="background: var(--bg-secondary); border-color: var(--border)">
  <div class="flex items-center gap-4">
    <ConnectionIndicator />
    {#if summoner}
      <span class="text-xs" style="color: var(--text-secondary)">
        {summoner.game_name}<span style="color: var(--text-muted)">#{summoner.tag_line}</span>
      </span>
    {/if}
  </div>

  <div class="flex items-center gap-4">
    {#if phase !== "None"}
      <span class="rounded px-2 py-0.5 text-xs font-medium" style="background: var(--bg-tertiary); color: var(--accent-blue)">
        {phaseLabels[phase] ?? phase}
      </span>
    {/if}
    <span class="text-xs" style="color: var(--text-muted)">v0.1.0</span>
  </div>
</footer>
