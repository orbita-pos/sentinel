<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { DbStats } from "../lib/types/index.js";

  let apiKey = $state("");
  let region = $state("la1");
  let dbStats: DbStats | null = $state(null);
  let saveStatus = $state("");
  let hasKey = $state(false);
  let showAdvanced = $state(false);

  const regions = [
    { value: "na1", label: "North America" },
    { value: "euw1", label: "EU West" },
    { value: "eun1", label: "EU Nordic & East" },
    { value: "kr", label: "Korea" },
    { value: "br1", label: "Brazil" },
    { value: "la1", label: "LAN" },
    { value: "la2", label: "LAS" },
    { value: "oc1", label: "Oceania" },
    { value: "tr1", label: "Turkey" },
    { value: "ru", label: "Russia" },
    { value: "jp1", label: "Japan" },
    { value: "ph2", label: "Philippines" },
    { value: "sg2", label: "Singapore" },
    { value: "th2", label: "Thailand" },
    { value: "tw2", label: "Taiwan" },
    { value: "vn2", label: "Vietnam" },
  ];

  $effect(() => {
    invoke<DbStats>("get_db_stats").then((s) => (dbStats = s));
    invoke<boolean>("has_api_key").then((v) => {
      hasKey = v;
      if (v) showAdvanced = true;
    });
  });

  async function saveApiKey() {
    if (!apiKey.trim()) return;
    try {
      saveStatus = "saving";
      await invoke("set_api_key", { key: apiKey.trim() });
      saveStatus = "saved";
      hasKey = true;
      setTimeout(() => (saveStatus = ""), 2000);
    } catch (e) {
      saveStatus = "error";
      console.error("Failed to save API key:", e);
    }
  }

  async function saveRegion() {
    try {
      await invoke("set_region", { region });
    } catch (e) {
      console.error("Failed to save region:", e);
    }
  }
</script>

<div class="mx-auto max-w-2xl">
  <div class="mb-8">
    <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Settings</h2>
    <p class="mt-1 text-sm" style="color: var(--text-secondary)">Configure Sentinel</p>
  </div>

  <!-- Region -->
  <div class="mb-6 rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
    <h3 class="text-sm font-semibold" style="color: var(--text-primary)">Region</h3>
    <p class="mt-1 text-xs" style="color: var(--text-muted)">
      Select your League of Legends server region
    </p>
    <select
      bind:value={region}
      onchange={saveRegion}
      class="mt-3 w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-1"
      style="background: var(--bg-primary); border-color: var(--border); color: var(--text-primary); --tw-ring-color: var(--accent-blue)"
    >
      {#each regions as r}
        <option value={r.value}>{r.label} ({r.value})</option>
      {/each}
    </select>
  </div>

  <!-- Advanced: Riot API Key (Optional) -->
  <div class="mb-6 rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
    <button
      onclick={() => (showAdvanced = !showAdvanced)}
      class="flex w-full items-center justify-between p-6 text-left"
    >
      <div>
        <h3 class="text-sm font-semibold" style="color: var(--text-primary)">
          Advanced: Riot API Key
          {#if hasKey}
            <span class="ml-2 rounded px-1.5 py-0.5 text-[10px] font-medium" style="background: var(--accent-green); color: white">Active</span>
          {:else}
            <span class="ml-2 rounded px-1.5 py-0.5 text-[10px] font-medium" style="background: var(--bg-tertiary); color: var(--text-muted)">Optional</span>
          {/if}
        </h3>
        <p class="mt-1 text-xs" style="color: var(--text-muted)">
          Unlocks detailed timeline analysis (CS timing, gold leads, death patterns)
        </p>
      </div>
      <span class="text-sm" style="color: var(--text-muted)">{showAdvanced ? "^" : "v"}</span>
    </button>

    {#if showAdvanced}
      <div class="border-t px-6 pb-6 pt-4" style="border-color: var(--border)">
        <div class="mb-3 rounded-lg p-3" style="background: var(--bg-tertiary)">
          <p class="text-xs" style="color: var(--text-secondary)">
            Without an API key, Sentinel works using your League client's local data.
            An API key unlocks minute-by-minute timeline analysis for deeper pattern detection.
          </p>
          <p class="mt-2 text-xs">
            <span style="color: var(--text-muted)">Get a free key at </span>
            <span style="color: var(--accent-blue)">developer.riotgames.com</span>
            <span style="color: var(--text-muted)"> (expires every 24h)</span>
          </p>
        </div>
        <div class="flex gap-2">
          <input
            type="password"
            bind:value={apiKey}
            placeholder="RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
            class="flex-1 rounded-lg border px-3 py-2 text-sm outline-none focus:ring-1"
            style="background: var(--bg-primary); border-color: var(--border); color: var(--text-primary); --tw-ring-color: var(--accent-blue)"
            onkeydown={(e) => { if (e.key === 'Enter') saveApiKey() }}
          />
          <button
            onclick={saveApiKey}
            class="rounded-lg px-4 py-2 text-sm font-medium text-white transition-colors"
            style="background: var(--accent-blue)"
            disabled={saveStatus === 'saving'}
          >
            {saveStatus === "saving" ? "..." : saveStatus === "saved" ? "Saved" : "Save"}
          </button>
        </div>
      </div>
    {/if}
  </div>

  <!-- Database Info -->
  {#if dbStats}
    <div class="rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
      <h3 class="text-sm font-semibold" style="color: var(--text-primary)">Data Storage</h3>
      <div class="mt-3 space-y-2 text-sm">
        <div class="flex justify-between">
          <span style="color: var(--text-secondary)">Database location</span>
          <span class="font-mono text-xs" style="color: var(--text-muted)">{dbStats.db_path}</span>
        </div>
        <div class="flex justify-between">
          <span style="color: var(--text-secondary)">Summoners stored</span>
          <span style="color: var(--text-primary)">{dbStats.summoners}</span>
        </div>
        <div class="flex justify-between">
          <span style="color: var(--text-secondary)">Settings entries</span>
          <span style="color: var(--text-primary)">{dbStats.app_state_entries}</span>
        </div>
      </div>
    </div>
  {/if}
</div>
