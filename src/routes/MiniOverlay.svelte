<script lang="ts">
  import { liveGameState } from "../lib/stores/livegame.js";
  import { invoke } from "@tauri-apps/api/core";
  import { currentPatch } from "../lib/stores/champions.js";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let state = $derived($liveGameState);
  let patch = $derived($currentPatch);
  let opggBuild: any = $state(null);
  let intel: any = $state(null);
  let lastChamp = $state("");
  let lastFetch = $state(0);
  let isDragging = $state(false);

  // Fetch OP.GG build once per champion
  $effect(() => {
    const me = state?.my_team.find(p => p.is_local_player);
    if (!me || me.champion === lastChamp) return;
    lastChamp = me.champion;
    invoke("get_opgg_build", { champion: me.champion, position: "all" })
      .then((r: any) => opggBuild = r)
      .catch(() => {});
  });

  // Fetch dynamic intel every 5 seconds
  $effect(() => {
    if (!state || state.game_time - lastFetch < 5) return;
    lastFetch = state.game_time;
    const me = state.my_team.find(p => p.is_local_player);
    if (!me) return;
    invoke("get_item_intelligence", {
      myChampion: me.champion,
      myItems: me.items.map(i => i.item_id).filter(id => id > 0),
      myGold: state.active_player.current_gold,
      enemyChampions: state.enemy_team.map(p => p.champion),
      enemyItems: state.enemy_team.map(p => p.items.map(i => i.item_id).filter(id => id > 0)),
      enemyStats: state.enemy_team.map(p => [p.kills, p.deaths, p.items.reduce((s: number, i: any) => s + i.price, 0)]),
    }).then((r: any) => intel = r).catch(() => {});
  });

  let me = $derived(state?.my_team.find(p => p.is_local_player));

  // Timer
  let timer = $derived(() => {
    if (!state) return "0:00";
    const m = Math.floor(state.game_time / 60);
    const s = Math.floor(state.game_time % 60);
    return `${m}:${s.toString().padStart(2, "0")}`;
  });

  let blueKills = $derived(state?.my_team.reduce((s, p) => s + p.kills, 0) ?? 0);
  let redKills = $derived(state?.enemy_team.reduce((s, p) => s + p.kills, 0) ?? 0);

  function champImg(n: string) { return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/champion/${n}.png`; }
  function itemImg(id: number) { return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${id}.png`; }
  function fmtGold(g: number) { return g >= 1000 ? (g/1000).toFixed(1)+"k" : g.toString(); }

  // Draggable window
  async function startDrag(e: MouseEvent) {
    if (e.button !== 0) return;
    isDragging = true;
    try {
      await getCurrentWindow().startDragging();
    } catch {}
    isDragging = false;
  }

  async function closeMini() {
    await invoke("close_mini_overlay");
  }
</script>

<div
  class="flex h-screen w-screen flex-col overflow-hidden rounded-xl border"
  style="background: rgba(10, 14, 20, 0.95); border-color: rgba(42, 53, 68, 0.6); font-family: 'Inter', 'Segoe UI', system-ui, sans-serif; user-select: none"
>
  <!-- Title bar (draggable) -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="flex items-center justify-between px-3 py-1.5 cursor-grab"
    style="background: rgba(17, 24, 33, 0.9)"
    onmousedown={startDrag}
  >
    <div class="flex items-center gap-2">
      <img src="/logo-small.png" alt="" class="h-4 w-4 rounded" />
      <span class="text-[10px] font-bold" style="color: #94a3b8">SENTINEL</span>
    </div>
    <div class="flex items-center gap-2">
      {#if state}
        <span class="font-mono text-xs font-bold" style="color: #e2e8f0">{timer()}</span>
      {/if}
      <button onclick={closeMini} class="flex h-4 w-4 items-center justify-center rounded text-[10px]" style="color: #64748b" title="Close">x</button>
    </div>
  </div>

  {#if !state || !me}
    <div class="flex flex-1 items-center justify-center p-4">
      <p class="text-xs" style="color: #64748b">Waiting for game...</p>
    </div>
  {:else}
    <div class="flex-1 overflow-y-auto p-2 space-y-2" style="scrollbar-width: thin">
      <!-- Score -->
      <div class="flex items-center justify-center gap-3">
        <span class="text-lg font-black" style="color: #3b82f6">{blueKills}</span>
        <span class="text-xs" style="color: #64748b">vs</span>
        <span class="text-lg font-black" style="color: #ef4444">{redKills}</span>
        {#if state.team_gold_diff !== 0}
          <span class="rounded px-1.5 py-0.5 text-[10px] font-bold"
            style="color: {state.team_gold_diff >= 0 ? '#3b82f6' : '#ef4444'}; background: {state.team_gold_diff >= 0 ? 'rgba(59,130,246,0.15)' : 'rgba(239,68,68,0.15)'}">
            {state.team_gold_diff >= 0 ? '+' : ''}{fmtGold(state.team_gold_diff)}
          </span>
        {/if}
      </div>

      <!-- Your champion + KDA -->
      <div class="flex items-center gap-2 rounded-lg px-2 py-1.5" style="background: rgba(26,35,50,0.8)">
        <img src={champImg(me.champion)} alt="" class="h-8 w-8 rounded-lg" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
        <div class="flex-1">
          <div class="text-xs font-bold" style="color: #e2e8f0">
            <span style="color: #22c55e">{me.kills}</span><span style="color: #64748b">/</span><span style="color: #ef4444">{me.deaths}</span><span style="color: #64748b">/</span>{me.assists}
          </div>
          <div class="text-[9px]" style="color: #64748b">{me.cs} CS | Lv{me.level}</div>
        </div>
        <div class="text-right">
          <div class="text-sm font-bold" style="color: #eab308">{Math.floor(state.active_player.current_gold)}g</div>
        </div>
      </div>

      <!-- OP.GG Core Build -->
      {#if opggBuild?.core_items?.item_ids?.length > 0}
        <div class="rounded-lg px-2 py-1.5" style="background: rgba(26,35,50,0.8)">
          <p class="mb-1 text-[8px] font-bold uppercase" style="color: #3b82f6">Optimal Build
            {#if opggBuild.core_items.win_rate > 0}
              <span style="color: #22c55e">{(opggBuild.core_items.win_rate * 100).toFixed(0)}% WR</span>
            {/if}
          </p>
          <div class="flex gap-1">
            {#each opggBuild.core_items.item_ids as id}
              {@const owned = me.items.some(it => it.item_id === id)}
              <div class="h-7 w-7 overflow-hidden rounded" style="{owned ? 'opacity: 0.4; border: 1px solid #22c55e' : 'border: 1px solid #2a3544'}">
                <img src={itemImg(id)} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              </div>
            {/each}
            {#if opggBuild.boots?.item_ids?.length > 0}
              <span class="flex items-center text-[9px]" style="color: #64748b">+</span>
              {#each opggBuild.boots.item_ids as bid}
                <div class="h-7 w-7 overflow-hidden rounded" style="{me.items.some(it => it.item_id === bid) ? 'opacity: 0.4; border: 1px solid #22c55e' : 'border: 1px solid #2a3544'}">
                  <img src={itemImg(bid)} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {/if}

      <!-- Dynamic Recommendations -->
      {#if intel?.recommendations?.length > 0}
        <div class="rounded-lg px-2 py-1.5" style="background: rgba(26,35,50,0.8)">
          <p class="mb-1 text-[8px] font-bold uppercase" style="color: #a855f7">Buy Now</p>
          {#each intel.recommendations.slice(0, 3) as r}
            <div class="flex items-center gap-1.5 mb-1">
              <div class="h-6 w-6 overflow-hidden rounded" style="border: 1px solid {r.tag === 'RUSH' ? '#ef4444' : r.tag === 'CORE' ? '#22c55e' : '#3b82f6'}">
                <img src={itemImg(r.item_id)} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              </div>
              <div class="flex-1 min-w-0">
                <span class="text-[10px] font-medium" style="color: #e2e8f0">{r.item_name}</span>
                <span class="text-[8px] font-bold" style="color: {r.tag === 'RUSH' ? '#ef4444' : r.tag === 'CORE' ? '#22c55e' : '#3b82f6'}">{r.tag}</span>
              </div>
              <span class="text-[9px]" style="color: #eab308">{fmtGold(r.cost)}</span>
              {#if r.can_afford}
                <span class="text-[7px] font-bold" style="color: #22c55e">OK</span>
              {/if}
            </div>
          {/each}
        </div>
      {/if}

      <!-- Threats -->
      {#if intel?.threats?.length > 0}
        <div class="rounded-lg px-2 py-1.5" style="background: rgba(26,35,50,0.8)">
          <p class="mb-1 text-[8px] font-bold uppercase" style="color: #64748b">Threats</p>
          {#each intel.threats.filter((t: any) => t.threat_level === 'HIGH').slice(0, 3) as t}
            <div class="flex items-center gap-1.5 mb-0.5">
              <img src={champImg(t.champion)} alt="" class="h-5 w-5 rounded" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              <span class="text-[10px] font-medium" style="color: #e2e8f0">{t.champion}</span>
              <span class="text-[8px] font-bold" style="color: #ef4444">{t.kills}/{t.deaths}</span>
              <span class="rounded px-1 text-[7px]" style="background: rgba(10,14,20,0.6); color: {t.damage_type === 'AP' ? '#3b82f6' : '#ef4444'}">{t.damage_type}</span>
            </div>
          {/each}
          {#if intel.threats.filter((t: any) => t.threat_level === 'HIGH').length === 0}
            <p class="text-[9px]" style="color: #22c55e">No major threats</p>
          {/if}
        </div>
      {/if}

      <!-- Enemy Damage -->
      {#if intel?.enemy_damage}
        <div class="flex items-center gap-2 px-2">
          <div class="h-2 flex-1 overflow-hidden rounded-full" style="background: rgba(10,14,20,0.6)">
            <div class="flex h-full">
              <div class="h-full" style="width: {intel.enemy_damage.ad_pct}%; background: #ef4444"></div>
              <div class="h-full" style="width: {intel.enemy_damage.ap_pct}%; background: #3b82f6"></div>
            </div>
          </div>
          <span class="text-[8px] font-bold" style="color: #ef4444">AD{intel.enemy_damage.ad_pct}%</span>
          <span class="text-[8px] font-bold" style="color: #3b82f6">AP{intel.enemy_damage.ap_pct}%</span>
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  :global(body) {
    margin: 0;
    padding: 0;
    background: transparent;
    overflow: hidden;
  }
</style>
