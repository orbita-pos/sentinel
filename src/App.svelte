<script lang="ts">
  import "./app.css";
  // Initialize stores (registers Tauri event listeners)
  import "./lib/stores/connection.js";
  import "./lib/stores/champselect.js";
  import AppShell from "./lib/components/AppShell.svelte";
</script>

<AppShell />
