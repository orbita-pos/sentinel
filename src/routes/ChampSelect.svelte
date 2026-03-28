<script lang="ts">
  import { champSelectSession } from "../lib/stores/champselect.js";
  import TeamPanel from "../lib/components/draft/TeamPanel.svelte";
  import BanTracker from "../lib/components/draft/BanTracker.svelte";
  import DraftRecommendations from "../lib/components/draft/DraftRecommendations.svelte";

  let session = $derived($champSelectSession);

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
    <!-- Bans -->
    <div class="mb-4">
      <BanTracker bans={session.bans} />
    </div>

    <!-- Main layout: Teams + Recommendations -->
    <div class="grid grid-cols-[1fr_auto_1fr] gap-6">
      <!-- Ally team -->
      <TeamPanel
        players={session.my_team}
        side="ally"
        label="Your Team"
      />

      <!-- Recommendations (center) -->
      <div class="w-64">
        <DraftRecommendations />
      </div>

      <!-- Enemy team -->
      <TeamPanel
        players={session.their_team}
        side="enemy"
        label="Enemy Team"
      />
    </div>
  {/if}
</div>
