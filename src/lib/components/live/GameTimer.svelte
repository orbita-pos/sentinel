<script lang="ts">
  let { gameTime }: { gameTime: number } = $props();

  let formatted = $derived(() => {
    const min = Math.floor(gameTime / 60);
    const sec = Math.floor(gameTime % 60);
    return `${min}:${sec.toString().padStart(2, "0")}`;
  });
</script>

<div class="text-center">
  <span class="font-mono text-3xl font-bold" style="color: var(--text-primary)">{formatted()}</span>
</div>
