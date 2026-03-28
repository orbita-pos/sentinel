<script lang="ts">
  import type { LiveGameState } from "../../types/livegame.js";
  import { invoke } from "@tauri-apps/api/core";
  import { currentPatch } from "../../stores/champions.js";

  let { state }: { state: LiveGameState } = $props();
  let patch = $derived($currentPatch);

  // Backend intelligence response
  let intel: any = $state(null);
  let lastFetchTime = $state(0);
  let opggBuild: any = $state(null);
  let opggLoading = $state(false);

  // Fetch OP.GG build once per champion
  let lastChampFetched = $state("");
  $effect(() => {
    const me = state.my_team.find(p => p.is_local_player);
    if (!me || me.champion === lastChampFetched || opggLoading) return;
    lastChampFetched = me.champion;
    opggLoading = true;
    // Guess position from team composition or default to "all"
    invoke("get_opgg_build", { champion: me.champion, position: "all" })
      .then((result: any) => { opggBuild = result; })
      .catch((e: any) => console.warn("OP.GG build fetch:", e))
      .finally(() => { opggLoading = false; });
  });

  // Fetch intelligence from Rust backend every 5 seconds
  $effect(() => {
    const now = state.game_time;
    if (now - lastFetchTime < 5) return;
    lastFetchTime = now;

    const me = state.my_team.find(p => p.is_local_player);
    if (!me) return;

    invoke("get_item_intelligence", {
      myChampion: me.champion,
      myItems: me.items.map(i => i.item_id).filter(id => id > 0),
      myGold: state.active_player.current_gold,
      enemyChampions: state.enemy_team.map(p => p.champion),
      enemyItems: state.enemy_team.map(p => p.items.map(i => i.item_id).filter(id => id > 0)),
      enemyStats: state.enemy_team.map(p => [p.kills, p.deaths, p.items.reduce((s, i) => s + i.price, 0)]),
    }).then((result: any) => {
      intel = result;
    }).catch(e => console.error("Item intelligence error:", e));
  });

  let me = $derived(state.my_team.find(p => p.is_local_player));

  function champImg(n: string) { return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/champion/${n}.png`; }
  function itemImg(id: number) { return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${id}.png`; }
  function fmtGold(g: number) { return g >= 1000 ? (g/1000).toFixed(1)+"k" : g.toString(); }

  const threatColors: Record<string, string> = { HIGH: "var(--accent-red)", MED: "var(--accent-gold)", LOW: "var(--accent-green)" };
  const threatBg: Record<string, string> = { HIGH: "rgba(239,68,68,0.08)", MED: "rgba(234,179,8,0.06)", LOW: "rgba(34,197,94,0.06)" };
  const tagColors: Record<string, string> = { RUSH: "var(--accent-red)", CORE: "var(--accent-green)", BUY: "var(--accent-blue)", CONSIDER: "var(--text-muted)" };
</script>

<div class="space-y-4">
  <!-- ═══ YOUR BUILD ═══ -->
  {#if me}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="flex items-center gap-3 mb-3">
        <img src={champImg(me.champion)} alt={me.champion} class="h-12 w-12 rounded-xl" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
        <div class="flex-1">
          <div class="flex items-center gap-2">
            <span class="text-sm font-bold" style="color: var(--text-primary)">{me.champion}</span>
            {#if intel}
              <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="background: var(--bg-tertiary); color: var(--text-muted)">{intel.my_class?.toUpperCase()}</span>
              <span class="rounded px-1 py-0.5 text-[9px]" style="background: var(--bg-primary); color: {intel.my_damage_type === 'AP' ? 'var(--accent-blue)' : 'var(--accent-red)'}">{intel.my_damage_type}</span>
            {/if}
            <span class="text-xs" style="color: var(--text-muted)">Lv{me.level}</span>
          </div>
          <div class="text-lg font-black">
            <span style="color: var(--accent-green)">{me.kills}</span><span style="color: var(--text-muted)"> / </span><span style="color: var(--accent-red)">{me.deaths}</span><span style="color: var(--text-muted)"> / </span>{me.assists}
          </div>
        </div>
        <div class="text-right">
          <div class="text-lg font-bold" style="color: var(--accent-gold)">{Math.floor(state.active_player.current_gold)}g</div>
          <div class="text-[9px]" style="color: var(--text-muted)">{me.cs} CS | {me.ward_score.toFixed(0)} vision</div>
        </div>
      </div>
      <!-- Item slots -->
      <div class="flex gap-1.5">
        {#each Array(6) as _, i}
          {@const item = me.items[i]}
          <div class="h-10 w-10 overflow-hidden rounded-lg" style="background: var(--bg-primary); border: 1px solid var(--border)">
            {#if item}
              <img src={itemImg(item.item_id)} alt={item.name} class="h-full w-full object-cover" title="{item.name} ({item.price}g)" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
            {/if}
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- ═══ OP.GG OPTIMAL BUILD (from millions of matches) ═══ -->
  {#if opggBuild && opggBuild.core_items?.item_ids?.length > 0}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="mb-3 flex items-center justify-between">
        <div class="flex items-center gap-2">
          <span class="text-[10px] font-bold uppercase tracking-wide" style="color: var(--accent-blue)">OP.GG Optimal Build</span>
          {#if opggBuild.win_rate > 0}
            <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="background: var(--accent-green); color: white">
              {(opggBuild.win_rate * 100).toFixed(1)}% WR
            </span>
          {/if}
          {#if opggBuild.tier}
            <span class="rounded px-1 py-0.5 text-[9px] font-bold" style="background: var(--bg-tertiary); color: var(--accent-gold)">
              {opggBuild.tier}
            </span>
          {/if}
        </div>
        <span class="text-[9px]" style="color: var(--text-muted)">Data from millions of ranked games</span>
      </div>

      <!-- Core Build Path -->
      <div class="mb-3">
        <p class="mb-1.5 text-[9px] font-medium" style="color: var(--text-muted)">Core Build ({(opggBuild.core_items.win_rate * 100).toFixed(1)}% WR, {opggBuild.core_items.games?.toLocaleString()} games)</p>
        <div class="flex gap-1.5">
          {#each opggBuild.core_items.item_ids as id, i}
            {@const owned = me?.items.some(it => it.item_id === id)}
            <div class="relative">
              <div class="h-10 w-10 overflow-hidden rounded-lg" style="background: var(--bg-primary); {owned ? 'border: 2px solid var(--accent-green)' : 'border: 1px solid var(--border)'}; {owned ? 'opacity: 0.5' : ''}">
                <img src={itemImg(id)} alt={opggBuild.core_items.item_names?.[i] ?? ''} class="h-full w-full object-cover" title={opggBuild.core_items.item_names?.[i] ?? ''} onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              </div>
              {#if owned}
                <span class="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full text-[7px] font-bold text-white" style="background: var(--accent-green)">OK</span>
              {/if}
            </div>
          {/each}
          <div class="flex items-center text-[10px]" style="color: var(--text-muted)">
            <span class="mx-1">+</span>
          </div>
          <!-- Boots -->
          {#if opggBuild.boots?.item_ids?.length > 0}
            {#each opggBuild.boots.item_ids as bootId, i}
              {@const ownedBoot = me?.items.some(it => it.item_id === bootId)}
              <div class="relative">
                <div class="h-10 w-10 overflow-hidden rounded-lg" style="background: var(--bg-primary); {ownedBoot ? 'border: 2px solid var(--accent-green); opacity: 0.5' : 'border: 1px solid var(--border)'}">
                  <img src={itemImg(bootId)} alt={opggBuild.boots.item_names?.[i] ?? ''} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                </div>
                {#if ownedBoot}
                  <span class="absolute -top-1 -right-1 flex h-4 w-4 items-center justify-center rounded-full text-[7px] font-bold text-white" style="background: var(--accent-green)">OK</span>
                {/if}
              </div>
            {/each}
          {/if}
        </div>
      </div>

      <!-- Situational Items (4th-6th slot options) -->
      {#if opggBuild.situational_items?.length > 0}
        <div>
          <p class="mb-1.5 text-[9px] font-medium" style="color: var(--text-muted)">Situational (4th-6th item options)</p>
          <div class="flex flex-wrap gap-1.5">
            {#each opggBuild.situational_items.slice(0, 6) as sit}
              {#each sit.item_ids as sid, j}
                <div class="flex items-center gap-1 rounded px-1.5 py-1" style="background: var(--bg-tertiary)" title="{sit.item_names?.[j] ?? ''} ({(sit.win_rate * 100).toFixed(0)}% WR)">
                  <div class="h-6 w-6 overflow-hidden rounded">
                    <img src={itemImg(sid)} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                  </div>
                  <span class="text-[9px]" style="color: var(--text-secondary)">{sit.item_names?.[j]?.split(' ')[0] ?? ''}</span>
                </div>
              {/each}
            {/each}
          </div>
        </div>
      {/if}

      <!-- Runes -->
      {#if opggBuild.runes?.primary_tree}
        <div class="mt-3 flex items-center gap-2 border-t pt-2" style="border-color: var(--border)">
          <span class="text-[9px] font-medium" style="color: var(--text-muted)">Runes:</span>
          <span class="rounded px-1.5 py-0.5 text-[9px] font-medium" style="background: var(--bg-tertiary); color: var(--accent-purple)">{opggBuild.runes.primary_tree}</span>
          {#if opggBuild.runes.secondary_tree}
            <span class="text-[9px]" style="color: var(--text-muted)">+</span>
            <span class="rounded px-1.5 py-0.5 text-[9px]" style="background: var(--bg-tertiary); color: var(--text-secondary)">{opggBuild.runes.secondary_tree}</span>
          {/if}
          {#if opggBuild.runes.win_rate > 0}
            <span class="text-[9px]" style="color: var(--accent-green)">{(opggBuild.runes.win_rate * 100).toFixed(1)}% WR</span>
          {/if}
        </div>
      {/if}
    </div>
  {:else if opggLoading}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs" style="color: var(--text-muted)">Loading OP.GG build data...</p>
    </div>
  {/if}

  {#if intel}
    <!-- ═══ WARNINGS ═══ -->
    {#if intel.warnings?.length > 0}
      <div class="space-y-1.5">
        {#each intel.warnings as warn}
          <div class="flex items-center gap-2 rounded-lg px-3 py-2" style="background: rgba(234,179,8,0.08); border-left: 3px solid var(--accent-gold)">
            <span class="text-xs font-bold" style="color: var(--accent-gold)">WARNING</span>
            <span class="text-xs" style="color: var(--text-secondary)">{warn}</span>
          </div>
        {/each}
      </div>
    {/if}

    <!-- ═══ BUILD PATH (what you're building toward) ═══ -->
    {#if intel.build_path?.length > 0}
      <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="mb-3 text-[10px] font-bold uppercase tracking-wide" style="color: var(--accent-gold)">Building Toward</p>
        {#each intel.build_path as path}
          <div class="mb-2">
            <p class="text-[10px] mb-1" style="color: var(--text-muted)">
              <span class="font-medium" style="color: var(--text-secondary)">{path.component_name}</span> builds into:
            </p>
            <div class="flex flex-wrap gap-1.5">
              {#each path.builds_into.slice(0, 4) as target}
                <div class="flex items-center gap-1.5 rounded-lg px-2 py-1.5" style="background: var(--bg-tertiary); {target.can_afford ? 'border: 1px solid var(--accent-green)' : ''}">
                  <div class="h-7 w-7 overflow-hidden rounded" style="background: var(--bg-primary)">
                    <img src={itemImg(target.item_id)} alt={target.item_name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                  </div>
                  <div>
                    <div class="text-[10px] font-medium" style="color: var(--text-primary)">{target.item_name}</div>
                    <div class="text-[9px]" style="color: {target.can_afford ? 'var(--accent-green)' : 'var(--accent-gold)'}">
                      {target.can_afford ? 'CAN COMPLETE' : `${fmtGold(target.remaining_cost)} more`}
                    </div>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
    {/if}

    <!-- ═══ ENEMY DAMAGE ═══ -->
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="mb-2 text-[10px] font-bold uppercase tracking-wide" style="color: var(--text-muted)">Enemy Damage</p>
      <div class="flex items-center gap-3">
        <div class="h-4 flex-1 overflow-hidden rounded-full" style="background: var(--bg-primary)">
          <div class="flex h-full">
            <div class="h-full rounded-l-full transition-all" style="width: {intel.enemy_damage.ad_pct}%; background: linear-gradient(90deg, #ef4444, #f87171)"></div>
            <div class="h-full rounded-r-full transition-all" style="width: {intel.enemy_damage.ap_pct}%; background: linear-gradient(90deg, #3b82f6, #60a5fa)"></div>
          </div>
        </div>
        <div class="flex gap-3 text-xs font-bold shrink-0">
          <span style="color: var(--accent-red)">AD {intel.enemy_damage.ad_pct}%</span>
          <span style="color: var(--accent-blue)">AP {intel.enemy_damage.ap_pct}%</span>
        </div>
      </div>
    </div>

    <!-- ═══ RECOMMENDED ITEMS ═══ -->
    {#if intel.recommendations?.length > 0}
      <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="mb-3 text-[10px] font-bold uppercase tracking-wide" style="color: var(--accent-purple)">Recommended Items</p>
        <div class="space-y-2">
          {#each intel.recommendations as r}
            <div class="rounded-lg px-3 py-2.5" style="background: var(--bg-tertiary)">
              <div class="flex items-center gap-3">
                <div class="h-10 w-10 shrink-0 overflow-hidden rounded-lg" style="background: var(--bg-primary); border: 2px solid {tagColors[r.tag] ?? 'var(--border)'}">
                  <img src={itemImg(r.item_id)} alt={r.item_name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                </div>
                <div class="flex-1 min-w-0">
                  <div class="flex items-center gap-2">
                    <span class="text-sm font-bold" style="color: var(--text-primary)">{r.item_name}</span>
                    <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="color: {tagColors[r.tag] ?? 'var(--text-muted)'}">{r.tag}</span>
                    <span class="text-[10px] font-medium" style="color: var(--accent-gold)">{fmtGold(r.cost)}</span>
                    {#if r.can_afford}
                      <span class="rounded px-1 py-0.5 text-[8px] font-bold" style="background: var(--accent-green); color: white">CAN BUY</span>
                    {/if}
                  </div>
                  <p class="text-[10px] mt-0.5" style="color: var(--text-muted)">{r.reason}</p>
                </div>
              </div>
              <!-- Recipe -->
              {#if r.from_items?.length > 0}
                <div class="mt-2 flex items-center gap-1.5 pl-1">
                  <span class="text-[9px]" style="color: var(--text-muted)">Recipe:</span>
                  {#each r.from_items as comp}
                    <div class="flex items-center gap-1 rounded px-1.5 py-0.5" style="background: var(--bg-primary)">
                      <div class="h-4 w-4 overflow-hidden rounded">
                        <img src={itemImg(comp.id)} alt={comp.name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                      </div>
                      <span class="text-[9px]" style="color: {comp.owned ? 'var(--accent-green)' : 'var(--text-muted)'}">{comp.name}</span>
                      {#if comp.owned}
                        <span class="text-[8px] font-bold" style="color: var(--accent-green)">OK</span>
                      {:else}
                        <span class="text-[8px]" style="color: var(--accent-gold)">{comp.cost}g</span>
                      {/if}
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- ═══ ON NEXT BACK ═══ -->
    {#if intel.on_next_back?.length > 0}
      <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="mb-2 text-[10px] font-bold uppercase tracking-wide" style="color: var(--accent-gold)">On Your Next Back ({Math.floor(state.active_player.current_gold)}g)</p>
        <div class="flex flex-wrap gap-2">
          {#each intel.on_next_back as b}
            <div class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5" style="background: var(--bg-tertiary); {b.is_complete ? 'border: 1px solid var(--accent-green)' : ''}">
              <div class="h-7 w-7 overflow-hidden rounded" style="background: var(--bg-primary)">
                <img src={itemImg(b.item_id)} alt={b.item_name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              </div>
              <div>
                <div class="flex items-center gap-1">
                  <span class="text-[10px] font-medium" style="color: var(--text-primary)">{b.item_name}</span>
                  {#if b.is_complete}
                    <span class="text-[8px] font-bold" style="color: var(--accent-green)">COMPLETE</span>
                  {/if}
                </div>
                <div class="text-[9px]" style="color: var(--text-muted)">{b.context}</div>
              </div>
              <span class="text-[9px] font-medium shrink-0" style="color: var(--accent-gold)">{fmtGold(b.cost)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

    <!-- ═══ THREAT ASSESSMENT ═══ -->
    {#if intel.threats?.length > 0}
      <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
        <p class="mb-3 text-[10px] font-bold uppercase tracking-wide" style="color: var(--text-muted)">Threat Assessment</p>
        <div class="space-y-1.5">
          {#each intel.threats as t}
            <div class="flex items-center gap-2.5 rounded-lg px-3 py-2" style="background: {threatBg[t.threat_level] ?? 'var(--bg-tertiary)'}">
              <img src={champImg(t.champion)} alt={t.champion} class="h-9 w-9 rounded-lg" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-1.5">
                  <span class="text-xs font-bold" style="color: var(--text-primary)">{t.champion}</span>
                  <span class="rounded px-1 py-0.5 text-[8px] font-bold" style="color: {threatColors[t.threat_level] ?? 'var(--text-muted)'}">{t.threat_level}</span>
                  <span class="rounded px-1 py-0.5 text-[8px]" style="background: var(--bg-primary); color: {t.damage_type === 'AP' ? 'var(--accent-blue)' : 'var(--accent-red)'}">{t.damage_type}</span>
                  {#if t.has_healing}<span class="text-[8px]" style="color: var(--accent-green)">HEALS</span>{/if}
                </div>
                <div class="text-[10px]" style="color: var(--text-muted)">
                  {t.kills}/{t.deaths}
                  {#if t.is_weak} -- <span style="color: var(--accent-green)">not a threat, skip</span>{/if}
                  {#if t.is_fed} -- <span style="color: var(--accent-red)">build against this</span>{/if}
                </div>
              </div>
              <span class="text-[10px] font-medium shrink-0" style="color: var(--accent-gold)">{fmtGold(t.gold)}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  {:else}
    <div class="flex h-32 items-center justify-center rounded-xl border" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="text-xs" style="color: var(--text-muted)">Analyzing game state...</p>
    </div>
  {/if}
</div>
