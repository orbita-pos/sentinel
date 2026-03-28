<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentSummoner } from "../lib/stores/connection.js";

  interface KeyMoment {
    timestamp: string;
    minute: number;
    description: string;
    gold_impact: number;
    category: string;
  }
  interface PatternMatch {
    pattern_id: string;
    pattern_description: string;
    evidence: string;
  }
  interface Analysis {
    match_id: string;
    outcome: string;
    duration: string;
    champion_name: string;
    kills: number;
    deaths: number;
    assists: number;
    cs: number;
    key_moments: KeyMoment[];
    pattern_matches: PatternMatch[];
    no_timeline?: boolean;
  }

  let { matchId = "" }: { matchId?: string } = $props();
  let analysis: Analysis | null = $state(null);
  let loading = $state(false);
  let error = $state("");
  let summoner = $derived($currentSummoner);

  async function loadAnalysis(id: string) {
    if (!id || !summoner?.puuid) return;
    loading = true;
    error = "";
    try {
      analysis = await invoke<Analysis>("get_post_game_analysis", {
        matchId: id,
        puuid: summoner.puuid,
      });
    } catch (e) {
      error = String(e);
    }
    loading = false;
  }

  $effect(() => {
    if (matchId && summoner?.puuid) loadAnalysis(matchId);
  });

  function goldColor(impact: number): string {
    if (impact > 500) return "var(--accent-green)";
    if (impact < -500) return "var(--accent-red)";
    return "var(--text-secondary)";
  }
</script>

<div class="mx-auto max-w-3xl">
  <div class="mb-6">
    <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Post-Game Analysis</h2>
    <p class="mt-1 text-sm" style="color: var(--text-secondary)">Key moments and pattern insights</p>
  </div>

  {#if !matchId}
    <div class="flex h-48 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-sm" style="color: var(--text-muted)">Select a match from Match History to view analysis</p>
    </div>
  {:else if loading}
    <p class="text-sm" style="color: var(--text-muted)">Analyzing...</p>
  {:else if error}
    <p class="text-sm" style="color: var(--accent-red)">{error}</p>
  {:else if analysis}
    <!-- Header -->
    <div class="mb-4 rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border); border-left: 4px solid {analysis.outcome === 'Victory' ? 'var(--accent-blue)' : 'var(--accent-red)'}">
      <div class="flex items-center justify-between">
        <div>
          <span class="text-lg font-bold" style="color: {analysis.outcome === 'Victory' ? 'var(--accent-blue)' : 'var(--accent-red)'}">
            {analysis.outcome}
          </span>
          <span class="ml-3 text-sm" style="color: var(--text-secondary)">{analysis.champion_name} - {analysis.duration}</span>
        </div>
        <span class="text-lg font-semibold" style="color: var(--text-primary)">
          {analysis.kills}/{analysis.deaths}/{analysis.assists}
        </span>
      </div>
    </div>

    <!-- No timeline banner -->
    {#if analysis.no_timeline}
      <div class="mb-4 rounded-lg border px-4 py-3" style="background: var(--bg-tertiary); border-color: var(--accent-blue); border-left: 3px solid var(--accent-blue)">
        <p class="text-sm font-medium" style="color: var(--text-primary)">Basic stats only</p>
        <p class="mt-1 text-xs" style="color: var(--text-secondary)">
          Key moments and gold swing analysis require timeline data.
          This is available when Sentinel is running during the game,
          or with an API key for imported matches.
        </p>
      </div>
    {/if}

    <!-- Key Moments -->
    {#if analysis.key_moments.length > 0}
    <div class="mb-4">
      <h3 class="mb-3 text-sm font-semibold uppercase tracking-wide" style="color: var(--text-muted)">
        Key Moments ({analysis.key_moments.length})
      </h3>
      <div class="space-y-2">
        {#each analysis.key_moments as moment}
          <div class="rounded-lg border px-4 py-3" style="background: var(--bg-secondary); border-color: var(--border)">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-3">
                <span class="rounded px-2 py-0.5 font-mono text-xs font-bold" style="background: var(--bg-tertiary); color: var(--text-secondary)">
                  {moment.timestamp}
                </span>
                <span class="rounded px-1.5 py-0.5 text-[10px] font-medium" style="background: var(--bg-primary); color: {moment.category === 'Death' ? 'var(--accent-red)' : moment.category === 'GoldGained' ? 'var(--accent-green)' : 'var(--accent-gold)'}">
                  {moment.category}
                </span>
              </div>
              <span class="text-sm font-bold" style="color: {goldColor(moment.gold_impact)}">
                {moment.gold_impact > 0 ? '+' : ''}{moment.gold_impact}g
              </span>
            </div>
            <p class="mt-1.5 text-sm" style="color: var(--text-secondary)">{moment.description}</p>
          </div>
        {/each}
      </div>
    </div>
    {/if}

    <!-- Pattern Matches -->
    {#if analysis.pattern_matches.length > 0}
      <div>
        <h3 class="mb-3 text-sm font-semibold uppercase tracking-wide" style="color: var(--accent-purple)">
          Pattern Matches ({analysis.pattern_matches.length})
        </h3>
        <div class="space-y-2">
          {#each analysis.pattern_matches as pm}
            <div class="rounded-lg border-l-4 px-4 py-3" style="background: var(--bg-secondary); border-color: var(--accent-purple)">
              <p class="text-xs font-medium" style="color: var(--accent-purple)">{pm.pattern_id.replace('_', ' ').toUpperCase()}</p>
              <p class="mt-1 text-sm" style="color: var(--text-secondary)">{pm.evidence}</p>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {/if}
</div>
