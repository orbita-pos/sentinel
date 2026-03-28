<script lang="ts">
  import "./app.css";
  // Initialize stores (registers Tauri event listeners)
  import "./lib/stores/connection.js";
  import "./lib/stores/champselect.js";
  import "./lib/stores/livegame.js";
  import "./lib/stores/champions.js";
  import AppShell from "./lib/components/AppShell.svelte";
  import MiniOverlay from "./routes/MiniOverlay.svelte";

  // Detect if this is the mini overlay window
  const isMiniOverlay = window.location.pathname === "/mini" ||
    window.location.href.includes("/mini");
</script>

{#if isMiniOverlay}
  <MiniOverlay />
{:else}
  <AppShell />
{/if}
