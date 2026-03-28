<script lang="ts">
  import type { LiveGameState, LivePlayerState, LiveItem } from "../../types/livegame.js";
  import { currentPatch } from "../../stores/champions.js";

  let { state }: { state: LiveGameState } = $props();
  let patch = $derived($currentPatch);

  // ── Champion damage type database ──
  // Primary damage type for common champions (AP/AD/Mixed/True)
  const CHAMP_DAMAGE: Record<string, "AP" | "AD" | "Mixed"> = {
    // AP
    Ahri: "AP", Akali: "AP", Anivia: "AP", Annie: "AP", AurelionSol: "AP", Azir: "AP",
    Brand: "AP", Cassiopeia: "AP", Diana: "AP", Ekko: "AP", Elise: "AP", Evelynn: "AP",
    Fiddlesticks: "AP", Fizz: "AP", Hwei: "AP", Karma: "AP", Karthus: "AP", Kassadin: "AP",
    Katarina: "Mixed", Kennen: "AP", Leblanc: "AP", Lillia: "AP", Lissandra: "AP", Lux: "AP",
    Malzahar: "AP", Morgana: "AP", Neeko: "AP", Nidalee: "AP", Orianna: "AP", Rumble: "AP",
    Ryze: "AP", Seraphine: "AP", Shaco: "Mixed", Singed: "AP", Sona: "AP", Soraka: "AP",
    Swain: "AP", Syndra: "AP", Taliyah: "AP", Teemo: "AP", TwistedFate: "AP", Veigar: "AP",
    VelKoz: "AP", Vex: "AP", Viktor: "AP", Vladimir: "AP", Xerath: "AP", Ziggs: "AP",
    Zilean: "AP", Zoe: "AP", Zyra: "AP", Sylas: "AP", Nami: "AP", Janna: "AP",
    Lulu: "AP", Yuumi: "AP", Milio: "AP", Heimerdinger: "AP", Smolder: "AP",
    Naafiri: "AD", Aurora: "AP",
    // AD
    Aatrox: "AD", Aphelios: "AD", Ashe: "AD", Caitlyn: "AD", Camille: "AD", Darius: "AD",
    Draven: "AD", Ezreal: "Mixed", Fiora: "AD", Gangplank: "AD", Garen: "AD", Graves: "AD",
    Hecarim: "AD", Illaoi: "AD", Irelia: "AD", JarvanIV: "AD", Jax: "Mixed", Jayce: "AD",
    Jhin: "AD", Jinx: "AD", Kaisa: "Mixed", Kayn: "AD", Khazix: "AD", KogMaw: "Mixed",
    LeeSin: "AD", Lucian: "AD", MasterYi: "AD", MissFortune: "AD", Nasus: "AD",
    Nocturne: "AD", Olaf: "AD", Pantheon: "AD", Pyke: "AD", Qiyana: "AD", Quinn: "AD",
    Rakan: "AP", RekSai: "AD", Renekton: "AD", Rengar: "AD", Riven: "AD", Samira: "AD",
    Senna: "AD", Sett: "AD", Sivir: "AD", Talon: "AD", Tristana: "AD", Trundle: "AD",
    Tryndamere: "AD", Twitch: "AD", Urgot: "AD", Varus: "Mixed", Vayne: "AD", Vi: "AD",
    Viego: "AD", Wukong: "AD", Xayah: "AD", XinZhao: "AD", Yasuo: "AD", Yone: "AD",
    Zed: "AD", Zeri: "AD", Nilah: "AD", Belveth: "AD", KSante: "AD",
    // Mixed / Tanks
    Amumu: "AP", Blitzcrank: "AP", Braum: "AD", ChoGath: "AP", DrMundo: "AD",
    Galio: "AP", Gnar: "AD", Gragas: "AP", Leona: "AP", Malphite: "AP", Maokai: "AP",
    Nautilus: "AP", Nunu: "AP", Ornn: "AP", Poppy: "AD", Rammus: "AP", Sejuani: "AP",
    Shen: "AP", Skarner: "AD", TahmKench: "AP", Taric: "AP", Thresh: "AP",
    Volibear: "Mixed", Warwick: "Mixed", Yorick: "AD", Zac: "AP",
    Mordekaiser: "AP", Gwen: "AP", BelVeth: "AD",
  };

  // ── AP item IDs (major AP items) ──
  const AP_ITEM_IDS = new Set([
    3089, 3157, 3165, 3116, 3003, 3504, 3907, 3102, 4645, 6655, 6656,
    6657, 3118, 3152, 2502, 4628, 4629, 6653, 4005,
  ]);
  // ── AD / Lethality item IDs ──
  const AD_ITEM_IDS = new Set([
    3031, 3072, 3036, 3033, 3074, 3071, 3004, 3142, 6676, 6694, 6695,
    3814, 6693, 3179, 6696, 3161, 6697, 6698, 6699, 3508, 3046, 6671, 6672,
  ]);
  // ── Armor items ──
  const ARMOR_ITEMS = [
    { id: 3143, name: "Randuin's Omen", reason: "vs crit/AD heavy" },
    { id: 3075, name: "Thornmail", reason: "vs auto-attackers + healing" },
    { id: 3110, name: "Frozen Heart", reason: "vs attack speed" },
    { id: 3047, name: "Plated Steelcaps", reason: "boots vs AD" },
    { id: 3742, name: "Dead Man's Plate", reason: "armor + mobility" },
  ];
  // ── MR items ──
  const MR_ITEMS = [
    { id: 3102, name: "Banshee's Veil", reason: "spell shield vs burst AP" },
    { id: 3111, name: "Mercury's Treads", reason: "boots vs AP/CC" },
    { id: 3065, name: "Spirit Visage", reason: "MR + healing amp" },
    { id: 4401, name: "Force of Nature", reason: "vs sustained AP damage" },
    { id: 3194, name: "Kaenic Rookern", reason: "magic damage shield" },
  ];
  // ── Anti-heal ──
  const ANTIHEAL_ITEMS = [
    { id: 3165, name: "Morellonomicon", reason: "AP anti-heal" },
    { id: 3033, name: "Mortal Reminder", reason: "AD anti-heal" },
    { id: 3075, name: "Thornmail", reason: "tank anti-heal" },
  ];

  // ── Healing champions ──
  const HEALERS = new Set([
    "Aatrox", "DrMundo", "Fiora", "Irelia", "Kayn", "Olaf", "Soraka",
    "Sylas", "Vladimir", "Warwick", "Yuumi", "Swain", "Illaoi", "Nasus",
  ]);

  // ── Detect actual damage type from items ──
  function detectDamageType(champ: string, items: LiveItem[]): "AP" | "AD" | "Mixed" {
    const base = CHAMP_DAMAGE[champ] ?? "AD";
    let apItems = 0;
    let adItems = 0;
    for (const item of items) {
      if (AP_ITEM_IDS.has(item.item_id)) apItems++;
      if (AD_ITEM_IDS.has(item.item_id)) adItems++;
    }
    // Override if building opposite of expected
    if (base === "AP" && adItems > apItems && adItems >= 2) return "AD";
    if (base === "AD" && apItems > adItems && apItems >= 2) return "AP";
    return base;
  }

  // ── Threat level ──
  interface ThreatInfo {
    player: LivePlayerState;
    threat: "HIGH" | "MEDIUM" | "LOW";
    damageType: "AP" | "AD" | "Mixed";
    isFed: boolean;
    isWeak: boolean;
    hasHealing: boolean;
    goldEstimate: number;
    note: string;
  }

  let threats = $derived(() => {
    return state.enemy_team.map((p): ThreatInfo => {
      const goldEstimate = p.items.reduce((s, i) => s + i.price, 0);
      const kda = p.deaths === 0 ? p.kills + p.assists : (p.kills + p.assists) / p.deaths;
      const isFed = p.kills >= 5 || kda >= 4 || goldEstimate > 10000;
      const isWeak = p.deaths >= 5 && p.kills <= 1;
      const damageType = detectDamageType(p.champion, p.items);
      const hasHealing = HEALERS.has(p.champion);

      let threat: "HIGH" | "MEDIUM" | "LOW" = "MEDIUM";
      let note = "";

      if (isFed) {
        threat = "HIGH";
        note = `${p.kills}/${p.deaths} - Build against this threat`;
      } else if (isWeak) {
        threat = "LOW";
        note = `${p.kills}/${p.deaths} - Not a priority`;
      } else {
        note = `${p.kills}/${p.deaths}`;
      }

      return { player: p, threat, damageType, isFed, isWeak, hasHealing, goldEstimate, note };
    }).sort((a, b) => {
      const order = { HIGH: 0, MEDIUM: 1, LOW: 2 };
      return order[a.threat] - order[b.threat];
    });
  });

  // ── Team damage breakdown ──
  let teamDamageBreakdown = $derived(() => {
    let ap = 0, ad = 0;
    for (const t of threats()) {
      if (t.damageType === "AP") ap++;
      else if (t.damageType === "AD") ad++;
      else { ap += 0.5; ad += 0.5; }
    }
    const total = ap + ad || 1;
    return { ap, ad, apPct: Math.round((ap / total) * 100), adPct: Math.round((ad / total) * 100) };
  });

  // ── Build suggestions ──
  interface BuildSuggestion {
    item_name: string;
    item_id: number;
    reason: string;
    priority: "RUSH" | "CONSIDER" | "SITUATIONAL";
  }

  let buildSuggestions = $derived((): BuildSuggestion[] => {
    const suggestions: BuildSuggestion[] = [];
    const myItems = new Set(state.my_team.find(p => p.is_local_player)?.items.map(i => i.item_id) ?? []);
    const dmg = teamDamageBreakdown();
    const fedThreats = threats().filter(t => t.isFed);
    const anyHealing = threats().some(t => t.hasHealing && !t.isWeak);

    // Check if already has anti-heal
    const hasAntiheal = ANTIHEAL_ITEMS.some(i => myItems.has(i.id));

    // Priority 1: Build against fed threats
    for (const t of fedThreats) {
      if (t.damageType === "AP" || t.damageType === "Mixed") {
        const mrItem = MR_ITEMS.find(i => !myItems.has(i.id));
        if (mrItem) {
          suggestions.push({
            item_name: mrItem.name, item_id: mrItem.id,
            reason: `${t.player.champion} is ${t.player.kills}/${t.player.deaths} (${t.damageType})`,
            priority: "RUSH",
          });
          break;
        }
      }
      if (t.damageType === "AD" || t.damageType === "Mixed") {
        const armorItem = ARMOR_ITEMS.find(i => !myItems.has(i.id));
        if (armorItem) {
          suggestions.push({
            item_name: armorItem.name, item_id: armorItem.id,
            reason: `${t.player.champion} is ${t.player.kills}/${t.player.deaths} (${t.damageType})`,
            priority: "RUSH",
          });
          break;
        }
      }
    }

    // Priority 2: Team damage composition
    if (dmg.apPct >= 60) {
      const mrItem = MR_ITEMS.find(i => !myItems.has(i.id) && !suggestions.some(s => s.item_id === i.id));
      if (mrItem) suggestions.push({ ...mrItem, item_id: mrItem.id, item_name: mrItem.name, reason: `Enemy team is ${dmg.apPct}% AP damage`, priority: "CONSIDER" });
    }
    if (dmg.adPct >= 60) {
      const armorItem = ARMOR_ITEMS.find(i => !myItems.has(i.id) && !suggestions.some(s => s.item_id === i.id));
      if (armorItem) suggestions.push({ ...armorItem, item_id: armorItem.id, item_name: armorItem.name, reason: `Enemy team is ${dmg.adPct}% AD damage`, priority: "CONSIDER" });
    }

    // Priority 3: Anti-heal
    if (anyHealing && !hasAntiheal) {
      const healChamps = threats().filter(t => t.hasHealing && !t.isWeak).map(t => t.player.champion);
      const ahItem = ANTIHEAL_ITEMS.find(i => !myItems.has(i.id));
      if (ahItem) suggestions.push({ item_name: ahItem.name, item_id: ahItem.id, reason: `Anti-heal for ${healChamps.join(", ")}`, priority: "CONSIDER" });
    }

    return suggestions.slice(0, 4);
  });

  // ── Helpers ──
  function champImg(name: string): string {
    return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/champion/${name}.png`;
  }
  function itemImg(id: number): string {
    return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${id}.png`;
  }

  const threatColors = { HIGH: "var(--accent-red)", MEDIUM: "var(--accent-gold)", LOW: "var(--accent-green)" };
  const threatBg = { HIGH: "rgba(239,68,68,0.1)", MEDIUM: "rgba(234,179,8,0.08)", LOW: "rgba(34,197,94,0.08)" };
  const priorityColors = { RUSH: "var(--accent-red)", CONSIDER: "var(--accent-blue)", SITUATIONAL: "var(--text-muted)" };
</script>

<div class="space-y-4">
  <!-- ── Your Stats ── -->
  {#if state.my_team.find(p => p.is_local_player)}
  {@const me = state.my_team.find(p => p.is_local_player)!}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <div class="flex items-center gap-3">
        <img src={champImg(me.champion)} alt={me.champion} class="h-12 w-12 rounded-xl" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'} />
        <div>
          <div class="text-sm font-bold" style="color: var(--text-primary)">{me.champion} <span class="text-xs font-normal" style="color: var(--text-muted)">Lv{me.level}</span></div>
          <div class="text-lg font-black">
            <span style="color: var(--accent-green)">{me.kills}</span><span style="color: var(--text-muted)"> / </span><span style="color: var(--accent-red)">{me.deaths}</span><span style="color: var(--text-muted)"> / </span>{me.assists}
          </div>
        </div>
        <div class="ml-auto text-right">
          <div class="text-sm font-medium" style="color: var(--accent-gold)">{(me.items.reduce((s, i) => s + i.price, 0) / 1000).toFixed(1)}k gold</div>
          <div class="text-xs" style="color: var(--text-muted)">{me.cs} CS | {me.ward_score.toFixed(0)} vision</div>
        </div>
      </div>
    </div>
  {/if}

  <!-- ── Enemy Damage Breakdown ── -->
  <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
    <p class="mb-3 text-xs font-bold uppercase tracking-wide" style="color: var(--text-muted)">Enemy Damage Composition</p>
    <div class="flex items-center gap-3">
      <div class="h-3 flex-1 overflow-hidden rounded-full" style="background: var(--bg-primary)">
        <div class="flex h-full">
          <div class="h-full transition-all" style="width: {teamDamageBreakdown().adPct}%; background: var(--accent-red)"></div>
          <div class="h-full transition-all" style="width: {teamDamageBreakdown().apPct}%; background: var(--accent-blue)"></div>
        </div>
      </div>
      <div class="flex gap-3 text-xs font-bold">
        <span style="color: var(--accent-red)">AD {teamDamageBreakdown().adPct}%</span>
        <span style="color: var(--accent-blue)">AP {teamDamageBreakdown().apPct}%</span>
      </div>
    </div>
  </div>

  <!-- ── Build Suggestions ── -->
  {#if buildSuggestions().length > 0}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="mb-3 text-xs font-bold uppercase tracking-wide" style="color: var(--accent-purple)">Build Advice</p>
      <div class="space-y-2">
        {#each buildSuggestions() as sug}
          <div class="flex items-center gap-3 rounded-lg px-3 py-2" style="background: var(--bg-tertiary)">
            <div class="h-8 w-8 shrink-0 overflow-hidden rounded" style="background: var(--bg-primary)">
              <img src={itemImg(sug.item_id)} alt={sug.item_name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'} />
            </div>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold" style="color: var(--text-primary)">{sug.item_name}</span>
                <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="color: {priorityColors[sug.priority]}">{sug.priority}</span>
              </div>
              <p class="text-[10px]" style="color: var(--text-muted)">{sug.reason}</p>
            </div>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- ── Threat Assessment ── -->
  <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
    <p class="mb-3 text-xs font-bold uppercase tracking-wide" style="color: var(--text-muted)">Threat Assessment</p>
    <div class="space-y-2">
      {#each threats() as t}
        <div class="flex items-center gap-3 rounded-lg px-3 py-2" style="background: {threatBg[t.threat]}">
          <img src={champImg(t.player.champion)} alt={t.player.champion} class="h-8 w-8 rounded-lg" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display = 'none'} />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold" style="color: var(--text-primary)">{t.player.champion}</span>
              <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="color: {threatColors[t.threat]}">{t.threat}</span>
              <span class="rounded px-1 py-0.5 text-[9px]" style="background: var(--bg-primary); color: {t.damageType === 'AP' ? 'var(--accent-blue)' : 'var(--accent-red)'}">{t.damageType}</span>
              {#if t.hasHealing}
                <span class="text-[9px]" style="color: var(--accent-green)">Heals</span>
              {/if}
            </div>
            <p class="text-[10px]" style="color: var(--text-muted)">
              {t.note}
              {#if t.isWeak} -- <span style="color: var(--accent-green)">Skip building against</span>{/if}
            </p>
          </div>
          <span class="text-xs font-medium" style="color: var(--accent-gold)">{(t.goldEstimate / 1000).toFixed(1)}k</span>
        </div>
      {/each}
    </div>
  </div>
</div>
