<script lang="ts">
  import Sidebar from "./Sidebar.svelte";
  import StatusBar from "./StatusBar.svelte";
  import { currentRoute } from "../stores/router.js";
  import Dashboard from "../../routes/Dashboard.svelte";
  import Settings from "../../routes/Settings.svelte";

  let route = $derived($currentRoute);
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
        <div class="flex h-full items-center justify-center">
          <div class="text-center">
            <p class="text-lg" style="color: var(--text-secondary)">Match History</p>
            <p class="text-sm" style="color: var(--text-muted)">Coming in Phase 3</p>
          </div>
        </div>
      {:else if route === "patterns"}
        <div class="flex h-full items-center justify-center">
          <div class="text-center">
            <p class="text-lg" style="color: var(--text-secondary)">Patterns</p>
            <p class="text-sm" style="color: var(--text-muted)">Coming in Phase 6</p>
          </div>
        </div>
      {:else if route === "improvement"}
        <div class="flex h-full items-center justify-center">
          <div class="text-center">
            <p class="text-lg" style="color: var(--text-secondary)">Improvement Tracker</p>
            <p class="text-sm" style="color: var(--text-muted)">Coming in Phase 6</p>
          </div>
        </div>
      {:else}
        <div class="flex h-full items-center justify-center">
          <p style="color: var(--text-muted)">Unknown route</p>
        </div>
      {/if}
    </main>
  </div>

  <StatusBar />
</div>
