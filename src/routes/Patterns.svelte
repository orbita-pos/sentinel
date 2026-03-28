<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentSummoner } from "../lib/stores/connection.js";

  interface Pattern {
    id: string;
    category: string;
    description: string;
    confidence: number;
    sample_size: number;
    impact_wr_change: number | null;
    impact_games_pct: number | null;
    trend: string;
    first_detected: string;
    last_updated: string;
  }

  let patterns: Pattern[] = $state([]);
  let loading = $state(false);
  let analysisResult = $state("");
  let summoner = $derived($currentSummoner);

  async function loadPatterns() {
    if (!summoner?.puuid) return;
    loading = true;
    try {
      patterns = await invoke<Pattern[]>("get_detected_patterns", { puuid: summoner.puuid });
    } catch (e) {
      console.error("Failed to load patterns:", e);
    }
    loading = false;
  }

  async function runAnalysis() {
    if (!summoner?.puuid) return;
    analysisResult = "Running...";
    try {
      const result = await invoke<{ features_extracted: number; patterns_detected: number }>(
        "run_pattern_analysis",
        { puuid: summoner.puuid }
      );
      analysisResult = `Extracted ${result.features_extracted} features, detected ${result.patterns_detected} patterns`;
      await loadPatterns();
    } catch (e) {
      analysisResult = `Error: ${e}`;
    }
  }

  $effect(() => {
    if (summoner?.puuid) loadPatterns();
  });

  const categoryColors: Record<string, string> = {
    GoldManagement: "var(--accent-gold)",
    DeathTiming: "var(--accent-red)",
    CsEfficiency: "var(--accent-blue)",
    VisionControl: "var(--accent-purple)",
  };

  function confidenceBar(c: number): string {
    if (c >= 0.8) return "var(--accent-green)";
    if (c >= 0.5) return "var(--accent-gold)";
    return "var(--text-muted)";
  }
</script>

<div class="mx-auto max-w-3xl">
  <div class="mb-6 flex items-center justify-between">
    <div>
      <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Patterns</h2>
      <p class="mt-1 text-sm" style="color: var(--text-secondary)">
        {patterns.length > 0 ? `${patterns.length} behavioral patterns detected` : "Behavioral pattern analysis"}
      </p>
    </div>
    <button
      onclick={runAnalysis}
      class="rounded-lg px-4 py-2 text-sm font-medium text-white"
      style="background: var(--accent-purple)"
      disabled={!summoner?.puuid || analysisResult === 'Running...'}
    >
      {analysisResult === "Running..." ? "Analyzing..." : "Run Analysis"}
    </button>
  </div>

  {#if analysisResult && analysisResult !== "Running..."}
    <div class="mb-4 rounded-lg border px-4 py-2 text-xs" style="background: var(--bg-tertiary); border-color: var(--border); color: var(--text-muted)">
      {analysisResult}
    </div>
  {/if}

  {#if !summoner?.puuid}
    <div class="flex h-48 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-sm" style="color: var(--text-muted)">Connect to League and fetch matches first</p>
    </div>
  {:else if patterns.length === 0 && !loading}
    <div class="flex h-48 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="text-center">
        <p class="text-sm" style="color: var(--text-secondary)">No patterns detected yet</p>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">Fetch 10+ matches and click "Run Analysis"</p>
      </div>
    </div>
  {:else}
    <div class="space-y-3">
      {#each patterns as pattern (pattern.id)}
        <div class="rounded-xl border p-5" style="background: var(--bg-secondary); border-color: var(--border)">
          <div class="mb-2 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="rounded px-2 py-0.5 text-[10px] font-bold" style="background: var(--bg-primary); color: {categoryColors[pattern.category] ?? 'var(--text-muted)'}">
                {pattern.category}
              </span>
              <span class="text-xs" style="color: var(--text-muted)">{pattern.sample_size} games</span>
            </div>
            <div class="flex items-center gap-2">
              <!-- Confidence bar -->
              <div class="h-1.5 w-16 rounded-full" style="background: var(--bg-primary)">
                <div class="h-full rounded-full" style="width: {Math.round(pattern.confidence * 100)}%; background: {confidenceBar(pattern.confidence)}"></div>
              </div>
              <span class="text-[10px]" style="color: var(--text-muted)">{Math.round(pattern.confidence * 100)}%</span>
            </div>
          </div>
          <p class="text-sm leading-relaxed" style="color: var(--text-primary)">{pattern.description}</p>
          {#if pattern.impact_wr_change}
            <p class="mt-2 text-xs" style="color: {pattern.impact_wr_change < 0 ? 'var(--accent-red)' : 'var(--accent-green)'}">
              Impact: {pattern.impact_wr_change > 0 ? '+' : ''}{(pattern.impact_wr_change * 100).toFixed(1)}% WR
            </p>
          {/if}
        </div>
      {/each}
    </div>
  {/if}
</div>
