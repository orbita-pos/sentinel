<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentSummoner } from "../lib/stores/connection.js";
  import { championMap, getChampionName, getChampionImageUrl, currentPatch } from "../lib/stores/champions.js";

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
    champion_id?: number;
    kills: number;
    deaths: number;
    assists: number;
    cs: number;
    gold?: number;
    vision_score?: number;
    key_moments: KeyMoment[];
    pattern_matches: PatternMatch[];
    no_timeline?: boolean;
  }

  let { matchId = "" }: { matchId?: string } = $props();
  let analysis: Analysis | null = $state(null);
  let loading = $state(false);
  let error = $state("");
  let summoner = $derived($currentSummoner);
  let map = $derived($championMap);
  let patch = $derived($currentPatch);

  // Also load match stats from history for extra data
  let matchStats: any = $state(null);
  let opggBuild: any = $state(null);

  async function loadAnalysis(id: string) {
    if (!id || !summoner?.puuid) return;
    loading = true;
    error = "";
    try {
      analysis = await invoke<Analysis>("get_post_game_analysis", {
        matchId: id,
        puuid: summoner.puuid,
      });
      // Load match stats for extra info (gold, cs, vision)
      const history = await invoke<{ matches: any[] }>("get_match_history", {
        puuid: summoner.puuid, count: 100, offset: 0,
      });
      matchStats = history.matches.find((m: any) => m.match_id === id) ?? null;

      // Fetch OP.GG optimal build for comparison
      if (matchStats?.champion_id) {
        const info = map[matchStats.champion_id];
        if (info?.key) {
          invoke("get_opgg_build", { champion: info.key, position: "all" })
            .then((r: any) => { opggBuild = r; })
            .catch(() => {});
        }
      }
    } catch (e) {
      error = String(e);
    }
    loading = false;
  }

  $effect(() => {
    if (matchId && summoner?.puuid) loadAnalysis(matchId);
  });

  // Resolve champion name from map if needed
  let champName = $derived(() => {
    if (!analysis) return "";
    // Try resolving from champion map if name looks like "Champion123"
    if (analysis.champion_name.startsWith("Champion") && matchStats?.champion_id) {
      return getChampionName(map, matchStats.champion_id);
    }
    if (matchStats?.champion_name && !matchStats.champion_name.startsWith("Champion")) {
      return matchStats.champion_name;
    }
    return analysis.champion_name;
  });

  let champImg = $derived(() => {
    const id = matchStats?.champion_id ?? analysis?.champion_id ?? 0;
    return getChampionImageUrl(map, id, patch);
  });

  let kda = $derived(() => {
    if (!analysis) return "0";
    return analysis.deaths === 0 ? "Perfect" : ((analysis.kills + analysis.assists) / analysis.deaths).toFixed(1);
  });

  let kdaColor = $derived(() => {
    if (!analysis) return "var(--text-primary)";
    const ratio = analysis.deaths === 0 ? 99 : (analysis.kills + analysis.assists) / analysis.deaths;
    if (ratio >= 5) return "var(--accent-gold)";
    if (ratio >= 3) return "var(--accent-green)";
    return "var(--text-primary)";
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
    <!-- Header Card -->
    <div class="mb-4 rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border); border-left: 4px solid {analysis.outcome === 'Victory' ? 'var(--accent-blue)' : 'var(--accent-red)'}">
      <div class="flex items-center gap-4">
        <!-- Champion Image -->
        <div class="h-16 w-16 shrink-0 overflow-hidden rounded-xl" style="background: var(--bg-tertiary)">
          {#if champImg()}
            <img src={champImg()} alt={champName()} class="h-full w-full object-cover" />
          {:else}
            <div class="flex h-full w-full items-center justify-center text-xl font-bold" style="color: var(--text-muted)">
              {champName()[0] ?? "?"}
            </div>
          {/if}
        </div>

        <!-- Match Info -->
        <div class="flex-1">
          <div class="flex items-center gap-3">
            <span class="text-xl font-bold" style="color: {analysis.outcome === 'Victory' ? 'var(--accent-blue)' : 'var(--accent-red)'}">
              {analysis.outcome}
            </span>
            <span class="text-sm" style="color: var(--text-secondary)">{champName()}</span>
            <span class="text-xs" style="color: var(--text-muted)">{analysis.duration}</span>
          </div>
          <!-- KDA large -->
          <div class="mt-1 flex items-baseline gap-2">
            <span class="text-2xl font-bold" style="color: var(--text-primary)">
              {analysis.kills}<span style="color: var(--text-muted)"> / </span>{analysis.deaths}<span style="color: var(--text-muted)"> / </span>{analysis.assists}
            </span>
            <span class="text-sm font-medium" style="color: {kdaColor()}">
              {kda()} KDA
            </span>
          </div>
        </div>
      </div>

      <!-- Stats Row -->
      {#if matchStats}
        <div class="mt-4 flex gap-4 border-t pt-3" style="border-color: var(--border)">
          <div class="flex flex-col items-center">
            <span class="text-sm font-semibold" style="color: var(--text-primary)">{matchStats.cs}</span>
            <span class="text-[10px]" style="color: var(--text-muted)">CS ({(matchStats.cs / (matchStats.game_duration / 60)).toFixed(1)}/min)</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-sm font-semibold" style="color: var(--accent-gold)">{(matchStats.gold / 1000).toFixed(1)}k</span>
            <span class="text-[10px]" style="color: var(--text-muted)">Gold</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-sm font-semibold" style="color: var(--text-secondary)">{matchStats.vision_score}</span>
            <span class="text-[10px]" style="color: var(--text-muted)">Vision</span>
          </div>
          <div class="flex flex-col items-center">
            <span class="text-sm font-semibold" style="color: var(--text-secondary)">{matchStats.role ?? "?"}</span>
            <span class="text-[10px]" style="color: var(--text-muted)">Role</span>
          </div>
          <div class="ml-auto flex flex-col items-end">
            <span class="text-xs" style="color: var(--text-muted)">
              {new Date(matchStats.game_creation).toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" })}
            </span>
          </div>
        </div>
      {/if}
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

    <!-- ═══ BUILD COMPARISON (OP.GG) ═══ -->
    {#if opggBuild?.core_items?.item_ids?.length > 0}
      <div class="mb-4 rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
        <div class="mb-3 flex items-center justify-between">
          <h3 class="text-sm font-semibold uppercase tracking-wide" style="color: var(--accent-blue)">Build Comparison</h3>
          {#if opggBuild.core_items.win_rate > 0}
            <span class="rounded px-2 py-0.5 text-[10px] font-bold" style="background: var(--accent-green); color: white">
              Optimal: {(opggBuild.core_items.win_rate * 100).toFixed(0)}% WR
            </span>
          {/if}
        </div>

        <!-- OP.GG Optimal Build -->
        <div class="mb-3">
          <p class="mb-1.5 text-[10px] font-medium" style="color: var(--text-muted)">
            OP.GG Optimal ({opggBuild.core_items.games?.toLocaleString()} games)
          </p>
          <div class="flex gap-2">
            {#each opggBuild.core_items.item_ids as id, i}
              <div class="flex flex-col items-center gap-1">
                <div class="h-10 w-10 overflow-hidden rounded-lg" style="border: 2px solid var(--accent-blue)">
                  <img src={`https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${id}.png`} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                </div>
                <span class="text-[8px] text-center" style="color: var(--text-muted); max-width: 48px">{opggBuild.core_items.item_names?.[i]?.split(' ')[0] ?? ''}</span>
              </div>
            {/each}
            {#if opggBuild.boots?.item_ids?.length > 0}
              <div class="flex items-center text-[10px]" style="color: var(--text-muted)">+</div>
              {#each opggBuild.boots.item_ids as bid, i}
                <div class="flex flex-col items-center gap-1">
                  <div class="h-10 w-10 overflow-hidden rounded-lg" style="border: 1px solid var(--border)">
                    <img src={`https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${bid}.png`} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                  </div>
                  <span class="text-[8px] text-center" style="color: var(--text-muted); max-width: 48px">{opggBuild.boots.item_names?.[i]?.split(' ')[0] ?? ''}</span>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- Insight -->
        <div class="rounded-lg px-3 py-2" style="background: var(--bg-tertiary)">
          <p class="text-xs" style="color: var(--text-secondary)">
            {#if analysis.outcome === "Victory"}
              You won this game. The optimal build above has a {(opggBuild.core_items.win_rate * 100).toFixed(0)}% WR across {opggBuild.core_items.games?.toLocaleString()} games. Compare your items to see if you could optimize further.
            {:else}
              The OP.GG optimal build has a {(opggBuild.core_items.win_rate * 100).toFixed(0)}% WR. Consider following this build path in future games on {champName()}.
            {/if}
          </p>
        </div>

        <!-- Runes used vs optimal -->
        {#if opggBuild.runes?.primary_tree}
          <div class="mt-3 flex items-center gap-2 border-t pt-2" style="border-color: var(--border)">
            <span class="text-[9px] font-medium" style="color: var(--text-muted)">Optimal Runes:</span>
            <span class="rounded px-1.5 py-0.5 text-[9px] font-medium" style="background: var(--bg-tertiary); color: var(--accent-purple)">{opggBuild.runes.primary_tree}</span>
            {#if opggBuild.runes.secondary_tree}
              <span class="text-[9px]" style="color: var(--text-muted)">+</span>
              <span class="rounded px-1.5 py-0.5 text-[9px]" style="background: var(--bg-tertiary); color: var(--text-secondary)">{opggBuild.runes.secondary_tree}</span>
            {/if}
            {#if opggBuild.runes.win_rate > 0}
              <span class="text-[9px]" style="color: var(--accent-green)">{(opggBuild.runes.win_rate * 100).toFixed(0)}% WR</span>
            {/if}
          </div>
        {/if}
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
