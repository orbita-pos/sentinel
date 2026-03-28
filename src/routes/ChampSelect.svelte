<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { champSelectSession } from "../lib/stores/champselect.js";
  import { championMap, getChampionName, getChampionImageUrl, currentPatch } from "../lib/stores/champions.js";
  import TeamPanel from "../lib/components/draft/TeamPanel.svelte";
  import BanTracker from "../lib/components/draft/BanTracker.svelte";
  import DraftRecommendations from "../lib/components/draft/DraftRecommendations.svelte";

  let session = $derived($champSelectSession);
  let map = $derived($championMap);
  let patch = $derived($currentPatch);

  // Rune import state
  let importStatus = $state("");
  let importedChamp = $state("");

  let phaseLabel = $derived(() => {
    if (!session) return "Waiting...";
    switch (session.phase) {
      case "PLANNING": return "Planning Phase";
      case "BAN_PICK": return "Ban & Pick";
      case "FINALIZATION": return "Finalization";
      default: return session.phase;
    }
  });

  let timerStr = $derived(() => {
    if (!session || session.timer_remaining <= 0) return "";
    return `${Math.ceil(session.timer_remaining)}s`;
  });

  // Get local player's locked champion
  let myChampId = $derived(() => {
    if (!session) return 0;
    const me = session.my_team.find(p => p.is_local_player);
    return me?.champion_id ?? 0;
  });

  let myChampName = $derived(() => {
    const id = myChampId();
    if (id <= 0) return "";
    return getChampionName(map, id);
  });

  let myChampKey = $derived(() => {
    const id = myChampId();
    if (id <= 0) return "";
    return map[id]?.key ?? "";
  });

  let myChampImg = $derived(() => {
    const id = myChampId();
    return getChampionImageUrl(map, id, patch);
  });

  // Auto-import runes when champion is locked
  async function importRunes() {
    const key = myChampKey();
    if (!key || key === importedChamp) return;

    importStatus = "importing";
    try {
      const result = await invoke<{ success: boolean; page_name: string; primary_tree: string; secondary_tree: string; win_rate: number }>(
        "import_runes",
        { champion: key, position: "all" }
      );
      importStatus = `Imported: ${result.primary_tree} + ${result.secondary_tree} (${(result.win_rate * 100).toFixed(0)}% WR)`;
      importedChamp = key;
    } catch (e) {
      importStatus = `Failed: ${e}`;
    }
    setTimeout(() => {
      if (importStatus.startsWith("Imported") || importStatus.startsWith("Failed")) {
        importStatus = importedChamp ? "ready" : "";
      }
    }, 5000);
  }
</script>

<div class="mx-auto max-w-5xl">
  <!-- Header -->
  <div class="mb-6 flex items-center justify-between">
    <div>
      <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Champion Select</h2>
      <p class="mt-1 text-sm" style="color: var(--text-secondary)">
        {#if session}
          {phaseLabel()}
          {#if timerStr()}
            <span class="ml-2 font-mono" style="color: var(--accent-gold)">{timerStr()}</span>
          {/if}
        {:else}
          Waiting for champion select...
        {/if}
      </p>
    </div>
  </div>

  {#if !session}
    <div class="flex h-64 flex-col items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <img src="/logo.png" alt="Sentinel" class="mb-4 h-16 w-16 rounded-xl opacity-40" />
      <p class="text-sm" style="color: var(--text-secondary)">Enter champion select to see draft data</p>
      <p class="mt-1 text-xs" style="color: var(--text-muted)">Sentinel will auto-navigate here when you enter champ select</p>
    </div>
  {:else}
    <!-- Rune Import Bar (when champion is selected) -->
    {#if myChampId() > 0}
      <div class="mb-4 rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
        <div class="flex items-center gap-4">
          {#if myChampImg()}
            <img src={myChampImg()} alt={myChampName()} class="h-12 w-12 rounded-xl" />
          {/if}
          <div class="flex-1">
            <span class="text-sm font-bold" style="color: var(--text-primary)">{myChampName()}</span>
            {#if importStatus === "importing"}
              <p class="text-xs" style="color: var(--accent-blue)">Importing optimal runes...</p>
            {:else if importStatus.startsWith("Imported")}
              <p class="text-xs" style="color: var(--accent-green)">{importStatus}</p>
            {:else if importStatus.startsWith("Failed")}
              <p class="text-xs" style="color: var(--accent-red)">{importStatus}</p>
            {:else if importedChamp === myChampKey()}
              <p class="text-xs" style="color: var(--accent-green)">Runes imported</p>
            {/if}
          </div>
          <button
            onclick={importRunes}
            class="flex items-center gap-2 rounded-lg px-4 py-2.5 text-sm font-medium text-white transition-colors disabled:opacity-50"
            style="background: var(--accent-purple)"
            disabled={importStatus === "importing" || importedChamp === myChampKey()}
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
              <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
            </svg>
            {importedChamp === myChampKey() ? "Runes Imported" : "Import Runes (OP.GG)"}
          </button>
        </div>
      </div>
    {/if}

    <!-- Bans -->
    <div class="mb-4">
      <BanTracker bans={session.bans} />
    </div>

    <!-- Main layout: Teams + Recommendations -->
    <div class="grid grid-cols-[1fr_auto_1fr] gap-6">
      <TeamPanel players={session.my_team} side="ally" label="Your Team" />

      <div class="w-64">
        <DraftRecommendations />
      </div>

      <TeamPanel players={session.their_team} side="enemy" label="Enemy Team" />
    </div>
  {/if}
</div>
