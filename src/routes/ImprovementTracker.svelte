<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { currentSummoner } from "../lib/stores/connection.js";

  interface Metric {
    key: string;
    label: string;
    current_value: number;
    previous_value: number | null;
    trend_pct: number | null;
    unit: string;
  }
  interface Goal {
    id: number;
    name: string;
    description: string | null;
    metric_key: string;
    target_value: number | null;
    created_at: string;
  }

  let metrics: Metric[] = $state([]);
  let goals: Goal[] = $state([]);
  let loading = $state(false);
  let summoner = $derived($currentSummoner);

  // New goal form
  let newGoalName = $state("");
  let newGoalMetric = $state("win_rate");
  let newGoalTarget = $state("");

  async function loadData() {
    if (!summoner?.puuid) return;
    loading = true;
    try {
      const [metricsResult, goalsResult] = await Promise.all([
        invoke<{ metrics: Metric[] }>("get_weekly_metrics", { puuid: summoner.puuid }),
        invoke<Goal[]>("get_improvement_goals", { puuid: summoner.puuid }),
      ]);
      metrics = metricsResult.metrics;
      goals = goalsResult;
    } catch (e) {
      console.error("Failed to load improvement data:", e);
    }
    loading = false;
  }

  async function createGoal() {
    if (!summoner?.puuid || !newGoalName.trim()) return;
    try {
      await invoke("create_improvement_goal", {
        puuid: summoner.puuid,
        name: newGoalName.trim(),
        metricKey: newGoalMetric,
        targetValue: newGoalTarget ? parseFloat(newGoalTarget) : null,
      });
      newGoalName = "";
      newGoalTarget = "";
      await loadData();
    } catch (e) {
      console.error("Failed to create goal:", e);
    }
  }

  $effect(() => {
    if (summoner?.puuid) loadData();
  });

  function trendIcon(pct: number | null): string {
    if (pct === null) return "";
    if (pct > 2) return "+";
    if (pct < -2) return "-";
    return "=";
  }
  function trendColor(pct: number | null, higherIsBetter: boolean): string {
    if (pct === null) return "var(--text-muted)";
    const isPositive = higherIsBetter ? pct > 0 : pct < 0;
    if (Math.abs(pct) < 2) return "var(--text-muted)";
    return isPositive ? "var(--accent-green)" : "var(--accent-red)";
  }

  const higherBetter: Record<string, boolean> = {
    win_rate: true,
    cs_at_15: true,
    early_deaths: false,
    vision_per_min: true,
    kill_participation: true,
    lead_conversion: true,
  };
</script>

<div class="mx-auto max-w-3xl">
  <div class="mb-6">
    <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Improvement Tracker</h2>
    <p class="mt-1 text-sm" style="color: var(--text-secondary)">Track your performance metrics over time</p>
  </div>

  {#if !summoner?.puuid}
    <div class="flex h-48 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-sm" style="color: var(--text-muted)">Connect to League and fetch matches to see metrics</p>
    </div>
  {:else}
    <!-- Metrics Grid -->
    {#if metrics.length > 0}
      <div class="mb-6 grid grid-cols-3 gap-3">
        {#each metrics as m (m.key)}
          <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
            <p class="text-[10px] font-medium uppercase tracking-wide" style="color: var(--text-muted)">{m.label}</p>
            <div class="mt-1 flex items-end gap-2">
              <span class="text-xl font-bold" style="color: var(--text-primary)">
                {m.unit === '%' ? m.current_value.toFixed(1) : m.current_value.toFixed(m.current_value < 10 ? 1 : 0)}{m.unit === '%' ? '%' : ''}
              </span>
              {#if m.trend_pct !== null && Math.abs(m.trend_pct) >= 1}
                <span class="mb-0.5 text-xs font-medium" style="color: {trendColor(m.trend_pct, higherBetter[m.key] ?? true)}">
                  {m.trend_pct > 0 ? '+' : ''}{m.trend_pct.toFixed(1)}%
                </span>
              {/if}
            </div>
            {#if m.previous_value !== null}
              <p class="mt-1 text-[10px]" style="color: var(--text-muted)">
                prev: {m.previous_value.toFixed(1)}{m.unit === '%' ? '%' : ''}
              </p>
            {/if}
          </div>
        {/each}
      </div>
    {:else if !loading}
      <div class="mb-6 rounded-xl border p-6 text-center" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="text-sm" style="color: var(--text-muted)">Run pattern analysis on the Patterns page to generate metrics</p>
      </div>
    {/if}

    <!-- Goals -->
    <div class="mb-4">
      <h3 class="mb-3 text-sm font-semibold uppercase tracking-wide" style="color: var(--text-muted)">Improvement Goals</h3>

      {#if goals.length > 0}
        <div class="mb-4 space-y-2">
          {#each goals as goal (goal.id)}
            <div class="rounded-lg border px-4 py-3" style="background: var(--bg-secondary); border-color: var(--border)">
              <div class="flex items-center justify-between">
                <span class="text-sm font-medium" style="color: var(--text-primary)">{goal.name}</span>
                <span class="rounded px-2 py-0.5 text-[10px]" style="background: var(--bg-tertiary); color: var(--text-muted)">
                  {goal.metric_key}
                </span>
              </div>
              {#if goal.target_value}
                <p class="mt-1 text-xs" style="color: var(--text-secondary)">Target: {goal.target_value}</p>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <!-- Create goal form -->
      <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="mb-3 text-xs font-medium" style="color: var(--text-secondary)">Add a new goal</p>
        <div class="flex gap-2">
          <input
            bind:value={newGoalName}
            placeholder="Goal name"
            class="flex-1 rounded-lg border px-3 py-1.5 text-sm outline-none"
            style="background: var(--bg-primary); border-color: var(--border); color: var(--text-primary)"
          />
          <select
            bind:value={newGoalMetric}
            class="rounded-lg border px-2 py-1.5 text-sm outline-none"
            style="background: var(--bg-primary); border-color: var(--border); color: var(--text-primary)"
          >
            <option value="win_rate">Win Rate</option>
            <option value="cs_at_15">CS at 15</option>
            <option value="early_deaths">Early Deaths</option>
            <option value="vision_per_min">Vision/min</option>
            <option value="kill_participation">Kill Participation</option>
            <option value="lead_conversion">Lead Conversion</option>
          </select>
          <button
            onclick={createGoal}
            class="rounded-lg px-3 py-1.5 text-sm font-medium text-white"
            style="background: var(--accent-green)"
          >Add</button>
        </div>
      </div>
    </div>
  {/if}
</div>
