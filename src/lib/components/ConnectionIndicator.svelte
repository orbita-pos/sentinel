<script lang="ts">
  import { connectionStatus } from "../stores/connection.js";

  const statusConfig = {
    disconnected: { color: "bg-red-500", label: "Disconnected" },
    connecting: { color: "bg-yellow-500", label: "Connecting..." },
    connected: { color: "bg-green-500", label: "Connected" },
  };

  let status = $derived($connectionStatus);
  let config = $derived(statusConfig[status]);
</script>

<div class="flex items-center gap-2">
  <div class="relative flex h-2.5 w-2.5">
    {#if status === "connecting"}
      <span class="absolute inline-flex h-full w-full animate-ping rounded-full {config.color} opacity-75"></span>
    {/if}
    <span class="relative inline-flex h-2.5 w-2.5 rounded-full {config.color}"></span>
  </div>
  <span class="text-xs" style="color: var(--text-secondary)">{config.label}</span>
</div>
