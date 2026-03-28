<script lang="ts">
  import type { LiveGameState, LivePlayerState, LiveItem } from "../../types/livegame.js";
  import { currentPatch } from "../../stores/champions.js";

  let { state }: { state: LiveGameState } = $props();
  let patch = $derived($currentPatch);

  // ── Champion damage type database ──
  const CHAMP_DAMAGE: Record<string, "AP" | "AD" | "Mixed"> = {
    Ahri:"AP",Akali:"AP",Anivia:"AP",Annie:"AP",AurelionSol:"AP",Azir:"AP",Brand:"AP",
    Cassiopeia:"AP",Diana:"AP",Ekko:"AP",Elise:"AP",Evelynn:"AP",Fiddlesticks:"AP",
    Fizz:"AP",Hwei:"AP",Karma:"AP",Karthus:"AP",Kassadin:"AP",Katarina:"Mixed",
    Kennen:"AP",Leblanc:"AP",Lillia:"AP",Lissandra:"AP",Lux:"AP",Malzahar:"AP",
    Morgana:"AP",Neeko:"AP",Nidalee:"AP",Orianna:"AP",Rumble:"AP",Ryze:"AP",
    Seraphine:"AP",Singed:"AP",Sona:"AP",Soraka:"AP",Swain:"AP",Syndra:"AP",
    Taliyah:"AP",Teemo:"AP",TwistedFate:"AP",Veigar:"AP",VelKoz:"AP",Vex:"AP",
    Viktor:"AP",Vladimir:"AP",Xerath:"AP",Ziggs:"AP",Zilean:"AP",Zoe:"AP",Zyra:"AP",
    Sylas:"AP",Nami:"AP",Janna:"AP",Lulu:"AP",Yuumi:"AP",Milio:"AP",
    Heimerdinger:"AP",Smolder:"AP",Aurora:"AP",Amumu:"AP",Blitzcrank:"AP",
    ChoGath:"AP",Galio:"AP",Gragas:"AP",Leona:"AP",Malphite:"AP",Maokai:"AP",
    Nautilus:"AP",Nunu:"AP",Ornn:"AP",Rammus:"AP",Sejuani:"AP",Shen:"AP",
    TahmKench:"AP",Taric:"AP",Thresh:"AP",Zac:"AP",Mordekaiser:"AP",Gwen:"AP",
    Rakan:"AP",
    Aatrox:"AD",Aphelios:"AD",Ashe:"AD",Caitlyn:"AD",Camille:"AD",Darius:"AD",
    Draven:"AD",Fiora:"AD",Gangplank:"AD",Garen:"AD",Graves:"AD",Hecarim:"AD",
    Illaoi:"AD",Irelia:"AD",JarvanIV:"AD",Jayce:"AD",Jhin:"AD",Jinx:"AD",
    Khazix:"AD",LeeSin:"AD",Lucian:"AD",MasterYi:"AD",MissFortune:"AD",Nasus:"AD",
    Nocturne:"AD",Olaf:"AD",Pantheon:"AD",Pyke:"AD",Qiyana:"AD",Quinn:"AD",
    RekSai:"AD",Renekton:"AD",Rengar:"AD",Riven:"AD",Samira:"AD",Senna:"AD",
    Sett:"AD",Sivir:"AD",Talon:"AD",Tristana:"AD",Trundle:"AD",Tryndamere:"AD",
    Twitch:"AD",Urgot:"AD",Vi:"AD",Viego:"AD",Wukong:"AD",Xayah:"AD",
    XinZhao:"AD",Yasuo:"AD",Yone:"AD",Zed:"AD",Zeri:"AD",Nilah:"AD",
    BelVeth:"AD",KSante:"AD",Naafiri:"AD",Braum:"AD",DrMundo:"AD",Gnar:"AD",
    Poppy:"AD",Skarner:"AD",Yorick:"AD",
    Ezreal:"Mixed",Jax:"Mixed",Kaisa:"Mixed",KogMaw:"Mixed",Shaco:"Mixed",
    Varus:"Mixed",Volibear:"Mixed",Warwick:"Mixed",Kayn:"AD",
  };

  // ── Champion role mapping (for offensive item recommendations) ──
  const CHAMP_CLASS: Record<string, "mage" | "marksman" | "assassin" | "fighter" | "tank" | "support"> = {
    // Marksmen
    Aphelios:"marksman",Ashe:"marksman",Caitlyn:"marksman",Draven:"marksman",Ezreal:"marksman",
    Jhin:"marksman",Jinx:"marksman",Kaisa:"marksman",KogMaw:"marksman",Lucian:"marksman",
    MissFortune:"marksman",Samira:"marksman",Senna:"marksman",Sivir:"marksman",Tristana:"marksman",
    Twitch:"marksman",Varus:"marksman",Vayne:"marksman",Xayah:"marksman",Zeri:"marksman",
    Smolder:"marksman",Nilah:"marksman",
    // Assassins
    Akali:"assassin",Ekko:"assassin",Evelynn:"assassin",Fizz:"assassin",Kassadin:"assassin",
    Katarina:"assassin",Khazix:"assassin",Leblanc:"assassin",Naafiri:"assassin",Nocturne:"assassin",
    Pyke:"assassin",Qiyana:"assassin",Rengar:"assassin",Shaco:"assassin",Talon:"assassin",
    Viego:"assassin",Zed:"assassin",
    // Fighters
    Aatrox:"fighter",Camille:"fighter",Darius:"fighter",Fiora:"fighter",Gangplank:"fighter",
    Garen:"fighter",Gnar:"fighter",Gwen:"fighter",Hecarim:"fighter",Illaoi:"fighter",
    Irelia:"fighter",JarvanIV:"fighter",Jax:"fighter",Jayce:"fighter",Kayn:"fighter",
    LeeSin:"fighter",MasterYi:"fighter",Mordekaiser:"fighter",Nasus:"fighter",Olaf:"fighter",
    Pantheon:"fighter",RekSai:"fighter",Renekton:"fighter",Riven:"fighter",Sett:"fighter",
    Trundle:"fighter",Tryndamere:"fighter",Urgot:"fighter",Vi:"fighter",Volibear:"fighter",
    Warwick:"fighter",Wukong:"fighter",XinZhao:"fighter",Yasuo:"fighter",Yone:"fighter",
    Yorick:"fighter",KSante:"fighter",BelVeth:"fighter",
    // Mages
    Ahri:"mage",Anivia:"mage",Annie:"mage",AurelionSol:"mage",Azir:"mage",Brand:"mage",
    Cassiopeia:"mage",Diana:"mage",Fiddlesticks:"mage",Heimerdinger:"mage",Hwei:"mage",
    Karma:"mage",Karthus:"mage",Kennen:"mage",Lillia:"mage",Lissandra:"mage",Lux:"mage",
    Malzahar:"mage",Neeko:"mage",Nidalee:"mage",Orianna:"mage",Rumble:"mage",Ryze:"mage",
    Singed:"mage",Swain:"mage",Syndra:"mage",Taliyah:"mage",Teemo:"mage",TwistedFate:"mage",
    Veigar:"mage",VelKoz:"mage",Vex:"mage",Viktor:"mage",Vladimir:"mage",Xerath:"mage",
    Ziggs:"mage",Zilean:"mage",Zoe:"mage",Zyra:"mage",Sylas:"mage",Aurora:"mage",
    // Tanks
    Amumu:"tank",Blitzcrank:"tank",Braum:"tank",ChoGath:"tank",DrMundo:"tank",Galio:"tank",
    Gragas:"tank",Leona:"tank",Malphite:"tank",Maokai:"tank",Nautilus:"tank",Nunu:"tank",
    Ornn:"tank",Poppy:"tank",Rammus:"tank",Sejuani:"tank",Shen:"tank",Skarner:"tank",
    TahmKench:"tank",Zac:"tank",
    // Supports
    Janna:"support",Lulu:"support",Milio:"support",Morgana:"support",Nami:"support",
    Rakan:"support",Seraphine:"support",Sona:"support",Soraka:"support",Taric:"support",
    Thresh:"support",Yuumi:"support",
  };

  // ── Item databases ──
  const AP_ITEM_IDS = new Set([3089,3157,3165,3116,3003,3504,3907,3102,4645,6655,6656,6657,3118,3152,2502,4628,4629,6653,4005]);
  const AD_ITEM_IDS = new Set([3031,3072,3036,3033,3074,3071,3004,3142,6676,6694,6695,3814,6693,3179,6696,6697,6698,6699,3508,3046,6671,6672]);

  const HEALERS = new Set(["Aatrox","DrMundo","Fiora","Irelia","Kayn","Olaf","Soraka","Sylas","Vladimir","Warwick","Yuumi","Swain","Illaoi","Nasus"]);
  const SHIELD_HEAVY = new Set(["Lulu","Janna","Karma","Seraphine","Sona","Ivern"]);

  interface ItemRec {
    id: number;
    name: string;
    cost: number;
    reason: string;
    tag: string;
    tagColor: string;
    category: "offensive" | "defensive" | "boots" | "utility";
    buildsFrom: { id: number; name: string; cost: number }[];
  }

  // ── Item cost + recipe database ──
  const ITEM_DATA: Record<number, { cost: number; from: { id: number; n: string; c: number }[] }> = {
    // Boots
    3006: { cost: 1100, from: [{ id: 1001, n: "Boots", c: 300 }] },
    3009: { cost: 900, from: [{ id: 1001, n: "Boots", c: 300 }] },
    3020: { cost: 1100, from: [{ id: 1001, n: "Boots", c: 300 }] },
    3047: { cost: 1100, from: [{ id: 1001, n: "Boots", c: 300 }, { id: 1029, n: "Cloth Armor", c: 300 }] },
    3111: { cost: 1100, from: [{ id: 1001, n: "Boots", c: 300 }, { id: 1033, n: "Null-Magic Mantle", c: 450 }] },
    // AD
    3031: { cost: 3400, from: [{ id: 1038, n: "B.F. Sword", c: 1300 }, { id: 1018, n: "Cloak of Agility", c: 600 }] },
    3072: { cost: 3400, from: [{ id: 1038, n: "B.F. Sword", c: 1300 }, { id: 1053, n: "Vampiric Scepter", c: 900 }] },
    3036: { cost: 3000, from: [{ id: 3035, n: "Last Whisper", c: 1450 }] },
    3033: { cost: 3000, from: [{ id: 3035, n: "Last Whisper", c: 1450 }, { id: 2015, n: "Executioner's", c: 800 }] },
    3074: { cost: 3300, from: [{ id: 3077, n: "Tiamat", c: 1200 }, { id: 1053, n: "Vampiric Scepter", c: 900 }] },
    3071: { cost: 3100, from: [{ id: 3133, n: "Caulfield's Warhammer", c: 1100 }, { id: 3067, n: "Kindlegem", c: 800 }] },
    3142: { cost: 2800, from: [{ id: 3134, n: "Serrated Dirk", c: 1100 }] },
    6676: { cost: 3000, from: [{ id: 3134, n: "Serrated Dirk", c: 1100 }, { id: 1018, n: "Cloak of Agility", c: 600 }] },
    6694: { cost: 3200, from: [{ id: 3035, n: "Last Whisper", c: 1450 }, { id: 3133, n: "Caulfield's Warhammer", c: 1100 }] },
    3046: { cost: 2600, from: [{ id: 1042, n: "Dagger", c: 300 }, { id: 3086, n: "Zeal", c: 1050 }] },
    3161: { cost: 3400, from: [{ id: 3133, n: "Caulfield's Warhammer", c: 1100 }, { id: 3067, n: "Kindlegem", c: 800 }] },
    6333: { cost: 3300, from: [{ id: 3133, n: "Caulfield's Warhammer", c: 1100 }, { id: 1037, n: "Pickaxe", c: 875 }] },
    // AP
    3089: { cost: 3600, from: [{ id: 1058, n: "Needlessly Large Rod", c: 1250 }] },
    3157: { cost: 2600, from: [{ id: 3191, n: "Seeker's Armguard", c: 1000 }, { id: 3108, n: "Fiendish Codex", c: 900 }] },
    3116: { cost: 2600, from: [{ id: 1052, n: "Amplifying Tome", c: 435 }, { id: 1028, n: "Ruby Crystal", c: 400 }] },
    4645: { cost: 2800, from: [{ id: 1058, n: "Needlessly Large Rod", c: 1250 }] },
    3165: { cost: 2500, from: [{ id: 3916, n: "Oblivion Orb", c: 800 }, { id: 1026, n: "Blasting Wand", c: 850 }] },
    // Defensive
    4401: { cost: 2800, from: [{ id: 1057, n: "Negatron Cloak", c: 900 }, { id: 1028, n: "Ruby Crystal", c: 400 }] },
    3194: { cost: 2900, from: [{ id: 1057, n: "Negatron Cloak", c: 900 }, { id: 1028, n: "Ruby Crystal", c: 400 }] },
    3102: { cost: 2500, from: [{ id: 1058, n: "Needlessly Large Rod", c: 1250 }, { id: 1033, n: "Null-Magic Mantle", c: 450 }] },
    3065: { cost: 2900, from: [{ id: 3211, n: "Spectre's Cowl", c: 1250 }, { id: 3067, n: "Kindlegem", c: 800 }] },
    3143: { cost: 2700, from: [{ id: 3082, n: "Warden's Mail", c: 1000 }, { id: 1011, n: "Giant's Belt", c: 900 }] },
    3110: { cost: 2500, from: [{ id: 3082, n: "Warden's Mail", c: 1000 }, { id: 3024, n: "Glacial Buckler", c: 900 }] },
    3075: { cost: 2700, from: [{ id: 3076, n: "Bramble Vest", c: 800 }, { id: 1011, n: "Giant's Belt", c: 900 }] },
    3742: { cost: 2900, from: [{ id: 1031, n: "Chain Vest", c: 800 }, { id: 1028, n: "Ruby Crystal", c: 400 }] },
    // Utility
    6609: { cost: 2600, from: [{ id: 3134, n: "Serrated Dirk", c: 1100 }] },
  };

  function getItemData(id: number) {
    return ITEM_DATA[id] ?? { cost: 3000, from: [] };
  }

  // ── Detect damage type from actual items ──
  function detectDmg(champ: string, items: LiveItem[]): "AP" | "AD" | "Mixed" {
    const base = CHAMP_DAMAGE[champ] ?? "AD";
    let ap = 0, ad = 0;
    for (const i of items) { if (AP_ITEM_IDS.has(i.item_id)) ap++; if (AD_ITEM_IDS.has(i.item_id)) ad++; }
    if (base === "AP" && ad > ap && ad >= 2) return "AD";
    if (base === "AD" && ap > ad && ap >= 2) return "AP";
    return base;
  }

  // ── Threat info ──
  interface ThreatInfo {
    p: LivePlayerState;
    threat: "HIGH" | "MED" | "LOW";
    dmg: "AP" | "AD" | "Mixed";
    fed: boolean;
    weak: boolean;
    heals: boolean;
    gold: number;
    kda: number;
  }

  let me = $derived(state.my_team.find(p => p.is_local_player));
  let myChamp = $derived(me?.champion ?? "");
  let myClass = $derived(CHAMP_CLASS[myChamp] ?? "fighter");
  let myGold = $derived(state.active_player.current_gold);
  let myItemIds = $derived(new Set(me?.items.map(i => i.item_id) ?? []));
  let myItemGold = $derived(me?.items.reduce((s, i) => s + i.price, 0) ?? 0);

  // Has boots?
  const BOOT_IDS = new Set([3006,3009,3020,3047,3111,3117,3158]);
  let hasBoots = $derived(me?.items.some(i => BOOT_IDS.has(i.item_id)) ?? false);

  let threats = $derived((): ThreatInfo[] => {
    return state.enemy_team.map(p => {
      const gold = p.items.reduce((s, i) => s + i.price, 0);
      const kda = p.deaths === 0 ? p.kills + p.assists : (p.kills + p.assists) / p.deaths;
      const fed = p.kills >= 4 || (kda >= 3.5 && p.kills >= 2) || gold > 8000;
      const weak = p.deaths >= 4 && p.kills <= 1;
      return {
        p, threat: fed ? "HIGH" : weak ? "LOW" : "MED",
        dmg: detectDmg(p.champion, p.items), fed, weak,
        heals: HEALERS.has(p.champion), gold, kda,
      };
    }).sort((a, b) => b.gold - a.gold);
  });

  // ── Damage breakdown ──
  let dmgBreakdown = $derived(() => {
    let ap = 0, ad = 0;
    for (const t of threats()) {
      if (t.dmg === "AP") ap++; else if (t.dmg === "AD") ad++; else { ap += 0.5; ad += 0.5; }
    }
    const tot = ap + ad || 1;
    return { ap, ad, apPct: Math.round(ap / tot * 100), adPct: Math.round(ad / tot * 100) };
  });

  // ── Power spike warnings ──
  let powerSpikes = $derived((): string[] => {
    const warns: string[] = [];
    for (const t of threats()) {
      if (t.p.champion === "Kayle" && t.p.level >= 14 && t.p.level < 16)
        warns.push(`Kayle reaches level 16 soon -- fight before she scales`);
      if (t.p.champion === "Kassadin" && t.p.level >= 14 && t.p.level < 16)
        warns.push(`Kassadin approaching 16 -- shut him down now`);
      if (t.p.champion === "Veigar" && state.game_time > 1500)
        warns.push(`Veigar scales infinitely -- close the game`);
      if (t.p.champion === "Nasus" && t.p.cs > 200 && state.game_time > 1200)
        warns.push(`Nasus has ${t.p.cs} stacks potential -- don't let him free farm`);
      // Generic: enemy completing a major item
      const majorItems = t.p.items.filter(i => i.price >= 3000);
      if (majorItems.length >= 3 && t.fed)
        warns.push(`${t.p.champion} has ${majorItems.length} completed items -- focus or avoid`);
    }
    return warns.slice(0, 3);
  });

  // ── Build recommendations ──
  let recommendations = $derived((): ItemRec[] => {
    const recs: ItemRec[] = [];
    const has = myItemIds;
    const dm = dmgBreakdown();
    const fedAP = threats().filter(t => t.fed && t.dmg === "AP");
    const fedAD = threats().filter(t => t.fed && t.dmg === "AD");
    const anyHeals = threats().some(t => t.heals && !t.weak);
    const anyShields = threats().some(t => SHIELD_HEAVY.has(t.p.champion) && !t.weak);
    const hasAntiheal = [3165,3033,3075,3011].some(id => has.has(id));

    function rec(id: number, name: string, reason: string, tag: string, tagColor: string, category: ItemRec["category"]): ItemRec {
      const data = getItemData(id);
      return { id, name, cost: data.cost, reason, tag, tagColor, category, buildsFrom: data.from.map(f => ({ id: f.id, name: f.n, cost: f.c })) };
    }

    // ── BOOTS (if you don't have any) ──
    if (!hasBoots) {
      if (dm.adPct >= 60 || fedAD.length >= 2) {
        recs.push(rec(3047, "Plated Steelcaps", `${dm.adPct}% AD threats`, "BUY", "var(--accent-blue)", "boots"));
      } else if (dm.apPct >= 60 || fedAP.length >= 2) {
        recs.push(rec(3111, "Mercury's Treads", `${dm.apPct}% AP + CC threats`, "BUY", "var(--accent-blue)", "boots"));
      } else if (myClass === "marksman") {
        recs.push(rec(3006, "Berserker's Greaves", "Attack speed for DPS", "BUY", "var(--accent-blue)", "boots"));
      } else if (myClass === "mage") {
        recs.push(rec(3020, "Sorcerer's Shoes", "Magic penetration", "BUY", "var(--accent-blue)", "boots"));
      }
    }

    // ── DEFENSIVE: vs fed threats ──
    for (const t of fedAP) {
      const mrItems = [
        { id: 4401, n: "Force of Nature", r: "vs sustained AP" },
        { id: 3194, n: "Kaenic Rookern", r: "magic shield" },
        { id: 3102, n: "Banshee's Veil", r: "spell shield" },
        { id: 3065, n: "Spirit Visage", r: "MR + healing amp" },
      ];
      const pick = mrItems.find(i => !has.has(i.id) && !recs.some(r => r.id === i.id));
      if (pick) {
        recs.push(rec(pick.id, pick.n, `${t.p.champion} is ${t.p.kills}/${t.p.deaths} (AP) -- ${pick.r}`, "RUSH", "var(--accent-red)", "defensive"));
        break;
      }
    }
    for (const t of fedAD) {
      const armorItems = [
        { id: 3143, n: "Randuin's Omen", r: "vs crit ADC" },
        { id: 3110, n: "Frozen Heart", r: "vs attack speed" },
        { id: 3075, n: "Thornmail", r: "vs auto-attackers" },
        { id: 3742, n: "Dead Man's Plate", r: "armor + mobility" },
      ];
      const pick = armorItems.find(i => !has.has(i.id) && !recs.some(r => r.id === i.id));
      if (pick) {
        recs.push(rec(pick.id, pick.n, `${t.p.champion} is ${t.p.kills}/${t.p.deaths} (AD) -- ${pick.r}`, "RUSH", "var(--accent-red)", "defensive"));
        break;
      }
    }

    // ── OFFENSIVE: based on your class ──
    if (myClass === "marksman") {
      const adcItems = [
        { id: 3031, n: "Infinity Edge", r: "Crit damage spike" },
        { id: 3046, n: "Phantom Dancer", r: "Attack speed + crit" },
        { id: 3036, n: "Lord Dominik's", r: "Armor pen vs tanks" },
        { id: 3072, n: "Bloodthirster", r: "Lifesteal + shield" },
      ];
      const pick = adcItems.find(i => !has.has(i.id) && !recs.some(r => r.id === i.id));
      if (pick) recs.push(rec(pick.id, pick.n, pick.r, "CORE", "var(--accent-green)", "offensive"));
    } else if (myClass === "mage" || myClass === "support") {
      const mageItems = [
        { id: 3089, n: "Rabadon's Deathcap", r: "Max AP damage" },
        { id: 3157, n: "Zhonya's Hourglass", r: "Armor + stasis" },
        { id: 3116, n: "Rylai's Crystal", r: "Slow on abilities" },
        { id: 4645, n: "Shadowflame", r: "Magic pen vs squishies" },
      ];
      const pick = mageItems.find(i => !has.has(i.id) && !recs.some(r => r.id === i.id));
      if (pick) recs.push(rec(pick.id, pick.n, pick.r, "CORE", "var(--accent-green)", "offensive"));
    } else if (myClass === "assassin") {
      const assItems = [
        { id: 6676, n: "The Collector", r: "Execute + lethality" },
        { id: 3142, n: "Youmuu's Ghostblade", r: "Lethality + mobility" },
        { id: 6694, n: "Serylda's Grudge", r: "Armor pen + slow" },
      ];
      const pick = assItems.find(i => !has.has(i.id) && !recs.some(r => r.id === i.id));
      if (pick) recs.push(rec(pick.id, pick.n, pick.r, "CORE", "var(--accent-green)", "offensive"));
    } else if (myClass === "fighter") {
      const fItems = [
        { id: 3074, n: "Ravenous Hydra", r: "Lifesteal + wave clear" },
        { id: 3071, n: "Black Cleaver", r: "Armor shred + HP" },
        { id: 3161, n: "Spear of Shojin", r: "Ability haste + AD" },
        { id: 6333, n: "Death's Dance", r: "Damage delay + heal on kill" },
      ];
      const pick = fItems.find(i => !has.has(i.id) && !recs.some(r => r.id === i.id));
      if (pick) recs.push(rec(pick.id, pick.n, pick.r, "CORE", "var(--accent-green)", "offensive"));
    }

    // ── UTILITY: Anti-heal ──
    if (anyHeals && !hasAntiheal) {
      const healChamps = threats().filter(t => t.heals && !t.weak).map(t => t.p.champion).join(", ");
      if (myClass === "mage" || myClass === "support") {
        recs.push(rec(3165, "Morellonomicon", `Anti-heal for ${healChamps}`, "BUY", "var(--accent-blue)", "utility"));
      } else {
        recs.push(rec(3033, "Mortal Reminder", `Anti-heal for ${healChamps}`, "BUY", "var(--accent-blue)", "utility"));
      }
    }

    // ── UTILITY: Anti-shield ──
    if (anyShields && !has.has(6609)) {
      recs.push(rec(6609, "Serpent's Fang", `Shield breaker vs ${threats().filter(t => SHIELD_HEAVY.has(t.p.champion)).map(t => t.p.champion).join(", ")}`, "CONSIDER", "var(--text-muted)", "utility"));
    }

    return recs.slice(0, 6);
  });

  // ── "You can buy" -- prioritize complete items, fall back to components ──
  let affordableNow = $derived((): { id: number; name: string; cost: number; isComplete: boolean }[] => {
    const results: { id: number; name: string; cost: number; isComplete: boolean }[] = [];
    const gold = myGold;
    const has = myItemIds;

    // First: check recommended items you can afford complete
    for (const r of recommendations()) {
      if (gold >= r.cost && !has.has(r.id)) {
        results.push({ id: r.id, name: r.name, cost: r.cost, isComplete: true });
      }
    }

    // If we can't afford any complete items, show components of recommended items
    if (results.length === 0) {
      for (const r of recommendations()) {
        for (const comp of r.buildsFrom) {
          if (gold >= comp.cost && !has.has(comp.id) && !results.some(x => x.id === comp.id)) {
            results.push({ id: comp.id, name: `${comp.name} (for ${r.name})`, cost: comp.cost, isComplete: false });
          }
        }
      }
    }

    // Always suggest Control Ward if affordable
    if (gold >= 75 && !results.some(r => r.id === 2055)) {
      results.push({ id: 2055, name: "Control Ward", cost: 75, isComplete: false });
    }

    return results.slice(0, 5);
  });

  // ── Helpers ──
  function champImg(n: string) { return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/champion/${n}.png`; }
  function itemImg(id: number) { return `https://ddragon.leagueoflegends.com/cdn/${patch}/img/item/${id}.png`; }
  function fmtGold(g: number) { return g >= 1000 ? (g/1000).toFixed(1)+"k" : g.toString(); }

  const threatColors = { HIGH: "var(--accent-red)", MED: "var(--accent-gold)", LOW: "var(--accent-green)" };
  const threatBg = { HIGH: "rgba(239,68,68,0.08)", MED: "rgba(234,179,8,0.06)", LOW: "rgba(34,197,94,0.06)" };
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
            <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="background: var(--bg-tertiary); color: var(--text-muted)">{myClass.toUpperCase()}</span>
            <span class="text-xs" style="color: var(--text-muted)">Lv{me.level}</span>
          </div>
          <div class="text-lg font-black">
            <span style="color: var(--accent-green)">{me.kills}</span><span style="color: var(--text-muted)"> / </span><span style="color: var(--accent-red)">{me.deaths}</span><span style="color: var(--text-muted)"> / </span>{me.assists}
          </div>
        </div>
        <div class="text-right">
          <div class="text-sm font-bold" style="color: var(--accent-gold)">{fmtGold(myItemGold)} spent</div>
          <div class="text-lg font-bold" style="color: var(--accent-gold)">{Math.floor(myGold)}g</div>
          <div class="text-[9px]" style="color: var(--text-muted)">in pocket</div>
        </div>
      </div>
      <!-- Item slots (6) -->
      <div class="flex gap-1.5">
        {#each Array(6) as _, i}
          {@const item = me.items[i]}
          <div class="h-10 w-10 overflow-hidden rounded-lg" style="background: var(--bg-primary); border: 1px solid var(--border)">
            {#if item}
              <img src={itemImg(item.item_id)} alt={item.name} class="h-full w-full object-cover" title="{item.name} ({item.price}g)" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
            {/if}
          </div>
        {/each}
        <div class="ml-2 flex items-center text-[10px]" style="color: var(--text-muted)">{me.cs} CS | {me.ward_score.toFixed(0)} vision</div>
      </div>
    </div>
  {/if}

  <!-- ═══ POWER SPIKE WARNINGS ═══ -->
  {#if powerSpikes().length > 0}
    <div class="space-y-1.5">
      {#each powerSpikes() as warn}
        <div class="flex items-center gap-2 rounded-lg border-l-3 px-3 py-2" style="background: rgba(234,179,8,0.08); border-left: 3px solid var(--accent-gold)">
          <span class="text-xs" style="color: var(--accent-gold)">WARNING</span>
          <span class="text-xs" style="color: var(--text-secondary)">{warn}</span>
        </div>
      {/each}
    </div>
  {/if}

  <!-- ═══ ENEMY DAMAGE COMPOSITION ═══ -->
  <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
    <p class="mb-2 text-[10px] font-bold uppercase tracking-wide" style="color: var(--text-muted)">Enemy Damage</p>
    <div class="flex items-center gap-3">
      <div class="h-4 flex-1 overflow-hidden rounded-full" style="background: var(--bg-primary)">
        <div class="flex h-full">
          <div class="h-full rounded-l-full transition-all" style="width: {dmgBreakdown().adPct}%; background: linear-gradient(90deg, #ef4444, #f87171)"></div>
          <div class="h-full rounded-r-full transition-all" style="width: {dmgBreakdown().apPct}%; background: linear-gradient(90deg, #3b82f6, #60a5fa)"></div>
        </div>
      </div>
      <div class="flex gap-3 text-xs font-bold shrink-0">
        <span style="color: var(--accent-red)">AD {dmgBreakdown().adPct}%</span>
        <span style="color: var(--accent-blue)">AP {dmgBreakdown().apPct}%</span>
      </div>
    </div>
  </div>

  <!-- ═══ BUILD RECOMMENDATIONS ═══ -->
  {#if recommendations().length > 0}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="mb-3 text-[10px] font-bold uppercase tracking-wide" style="color: var(--accent-purple)">Recommended Items</p>
      <div class="space-y-2">
        {#each recommendations() as r}
          <div class="rounded-lg px-3 py-2.5" style="background: var(--bg-tertiary)">
            <div class="flex items-center gap-3">
              <div class="h-10 w-10 shrink-0 overflow-hidden rounded-lg" style="background: var(--bg-primary); border: 2px solid {r.tagColor}">
                <img src={itemImg(r.id)} alt={r.name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              </div>
              <div class="flex-1 min-w-0">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-bold" style="color: var(--text-primary)">{r.name}</span>
                  <span class="rounded px-1.5 py-0.5 text-[9px] font-bold" style="color: {r.tagColor}">{r.tag}</span>
                  <span class="text-[10px] font-medium" style="color: var(--accent-gold)">{fmtGold(r.cost)}</span>
                  {#if myGold >= r.cost}
                    <span class="rounded px-1 py-0.5 text-[8px] font-bold" style="background: var(--accent-green); color: white">CAN BUY</span>
                  {/if}
                </div>
                <p class="text-[10px] mt-0.5" style="color: var(--text-muted)">{r.reason}</p>
              </div>
            </div>
            <!-- Recipe: builds from -->
            {#if r.buildsFrom.length > 0}
              <div class="mt-2 flex items-center gap-1.5 pl-1">
                <span class="text-[9px]" style="color: var(--text-muted)">Recipe:</span>
                {#each r.buildsFrom as comp}
                  <div class="flex items-center gap-1 rounded px-1.5 py-0.5" style="background: var(--bg-primary)">
                    <div class="h-4 w-4 overflow-hidden rounded">
                      <img src={itemImg(comp.id)} alt={comp.name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
                    </div>
                    <span class="text-[9px]" style="color: {myItemIds.has(comp.id) ? 'var(--accent-green)' : 'var(--text-muted)'}">{comp.name}</span>
                    {#if myItemIds.has(comp.id)}
                      <span class="text-[8px]" style="color: var(--accent-green)">OK</span>
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

  <!-- ═══ YOU CAN BUY NOW ═══ -->
  {#if myGold >= 75 && affordableNow().length > 0}
    <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
      <p class="mb-2 text-[10px] font-bold uppercase tracking-wide" style="color: var(--accent-gold)">
        On your next back ({Math.floor(myGold)}g available)
      </p>
      <div class="flex flex-wrap gap-2">
        {#each affordableNow() as c}
          <div
            class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5"
            style="background: var(--bg-tertiary); {c.isComplete ? 'border: 1px solid var(--accent-green)' : ''}"
          >
            <div class="h-7 w-7 overflow-hidden rounded" style="background: var(--bg-primary)">
              <img src={itemImg(c.id)} alt={c.name} class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
            </div>
            <div>
              <span class="text-[10px] font-medium" style="color: var(--text-primary)">{c.name}</span>
              {#if c.isComplete}
                <span class="ml-1 text-[8px] font-bold" style="color: var(--accent-green)">COMPLETE</span>
              {/if}
            </div>
            <span class="text-[9px] font-medium" style="color: var(--accent-gold)">{fmtGold(c.cost)}</span>
          </div>
        {/each}
      </div>
    </div>
  {/if}

  <!-- ═══ THREAT ASSESSMENT ═══ -->
  <div class="rounded-xl border p-4" style="background: var(--bg-secondary); border-color: var(--border)">
    <p class="mb-3 text-[10px] font-bold uppercase tracking-wide" style="color: var(--text-muted)">Threat Assessment</p>
    <div class="space-y-1.5">
      {#each threats() as t}
        <div class="flex items-center gap-2.5 rounded-lg px-3 py-2" style="background: {threatBg[t.threat]}">
          <img src={champImg(t.p.champion)} alt={t.p.champion} class="h-9 w-9 rounded-lg" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
          <div class="flex-1 min-w-0">
            <div class="flex items-center gap-1.5">
              <span class="text-xs font-bold" style="color: var(--text-primary)">{t.p.champion}</span>
              <span class="rounded px-1 py-0.5 text-[8px] font-bold" style="color: {threatColors[t.threat]}">{t.threat}</span>
              <span class="rounded px-1 py-0.5 text-[8px]" style="background: var(--bg-primary); color: {t.dmg === 'AP' ? 'var(--accent-blue)' : t.dmg === 'AD' ? 'var(--accent-red)' : 'var(--accent-purple)'}">{t.dmg}</span>
              {#if t.heals}<span class="text-[8px]" style="color: var(--accent-green)">HEALS</span>{/if}
            </div>
            <div class="text-[10px]" style="color: var(--text-muted)">
              {t.p.kills}/{t.p.deaths}/{t.p.assists}
              {#if t.weak} -- <span style="color: var(--accent-green)">not a threat, skip</span>{/if}
              {#if t.fed} -- <span style="color: var(--accent-red)">build against this</span>{/if}
            </div>
          </div>
          <!-- Enemy items mini -->
          <div class="flex gap-0.5 shrink-0">
            {#each t.p.items.slice(0, 4) as item}
              <div class="h-5 w-5 overflow-hidden rounded" style="background: var(--bg-primary)">
                <img src={itemImg(item.item_id)} alt="" class="h-full w-full object-cover" onerror={(e) => (e.currentTarget as HTMLImageElement).style.display='none'} />
              </div>
            {/each}
          </div>
          <span class="text-[10px] font-medium shrink-0" style="color: var(--accent-gold)">{fmtGold(t.gold)}</span>
        </div>
      {/each}
    </div>
  </div>
</div>
