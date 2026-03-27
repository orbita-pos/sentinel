<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import type { DbStats } from "../lib/types/index.js";

  let apiKey = $state("");
  let region = $state("la1");
  let dbStats: DbStats | null = $state(null);

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
  });
</script>

<div class="mx-auto max-w-2xl">
  <div class="mb-8">
    <h2 class="text-2xl font-bold" style="color: var(--text-primary)">Settings</h2>
    <p class="mt-1 text-sm" style="color: var(--text-secondary)">Configure Sentinel</p>
  </div>

  <!-- Riot API Key -->
  <div class="mb-6 rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
    <h3 class="text-sm font-semibold" style="color: var(--text-primary)">Riot API Key</h3>
    <p class="mt-1 text-xs" style="color: var(--text-muted)">
      Get your development key from developer.riotgames.com
    </p>
    <input
      type="password"
      bind:value={apiKey}
      placeholder="RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"
      class="mt-3 w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-1"
      style="background: var(--bg-primary); border-color: var(--border); color: var(--text-primary); --tw-ring-color: var(--accent-blue)"
    />
    <p class="mt-2 text-xs" style="color: var(--text-muted)">
      Development keys expire every 24 hours. Production key coming later.
    </p>
  </div>

  <!-- Region -->
  <div class="mb-6 rounded-xl border p-6" style="background: var(--bg-secondary); border-color: var(--border)">
    <h3 class="text-sm font-semibold" style="color: var(--text-primary)">Region</h3>
    <p class="mt-1 text-xs" style="color: var(--text-muted)">
      Select your League of Legends server region
    </p>
    <select
      bind:value={region}
      class="mt-3 w-full rounded-lg border px-3 py-2 text-sm outline-none focus:ring-1"
      style="background: var(--bg-primary); border-color: var(--border); color: var(--text-primary); --tw-ring-color: var(--accent-blue)"
    >
      {#each regions as r}
        <option value={r.value}>{r.label} ({r.value})</option>
      {/each}
    </select>
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
