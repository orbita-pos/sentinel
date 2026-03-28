use std::sync::Arc;

use serde::Serialize;

use crate::database::Database;

/// Complete item intelligence response sent to the frontend
#[derive(Debug, Clone, Serialize)]
pub struct ItemIntelligence {
    pub my_class: String,
    pub my_damage_type: String,
    pub enemy_damage: DamageBreakdown,
    pub build_path: Vec<BuildPathItem>,
    pub recommendations: Vec<ItemRecommendation>,
    pub on_next_back: Vec<BuyableItem>,
    pub threats: Vec<ThreatInfo>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DamageBreakdown {
    pub ad_pct: i32,
    pub ap_pct: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildPathItem {
    pub component_id: i64,
    pub component_name: String,
    pub builds_into: Vec<BuildTarget>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuildTarget {
    pub item_id: i64,
    pub item_name: String,
    pub total_cost: i64,
    pub remaining_cost: i64,
    pub can_afford: bool,
    pub relevance: String, // "HIGH", "MED", "LOW"
    pub from_items: Vec<ComponentInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ComponentInfo {
    pub id: i64,
    pub name: String,
    pub cost: i64,
    pub owned: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemRecommendation {
    pub item_id: i64,
    pub item_name: String,
    pub cost: i64,
    pub tag: String,
    pub reason: String,
    pub category: String,
    pub can_afford: bool,
    pub from_items: Vec<ComponentInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuyableItem {
    pub item_id: i64,
    pub item_name: String,
    pub cost: i64,
    pub is_complete: bool,
    pub context: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThreatInfo {
    pub champion: String,
    pub damage_type: String,
    pub threat_level: String,
    pub is_fed: bool,
    pub is_weak: bool,
    pub has_healing: bool,
    pub kills: i64,
    pub deaths: i64,
    pub gold: i64,
}

/// Main entry point: generate full item intelligence
pub fn analyze(
    db: &Arc<Database>,
    my_champion: &str,
    my_item_ids: &[i64],
    my_gold: f64,
    enemy_champions: &[String],
    enemy_items: &[Vec<i64>],
    enemy_stats: &[(i64, i64, i64)], // (kills, deaths, gold)
) -> ItemIntelligence {
    let gold = my_gold as i64;
    let my_items_set: std::collections::HashSet<i64> = my_item_ids.iter().copied().collect();

    // ── Resolve my champion class + damage type from DB ──
    let my_tags = db.get_champion_tags(my_champion).unwrap_or_default();
    let my_class = classify_champion(&my_tags);
    let my_damage_type = damage_type_from_tags(&my_tags);

    // ── Analyze enemy threats ──
    let healers = ["Aatrox","DrMundo","Fiora","Irelia","Kayn","Olaf","Soraka",
                   "Sylas","Vladimir","Warwick","Yuumi","Swain","Illaoi","Nasus"];

    let mut threats: Vec<ThreatInfo> = Vec::new();
    let mut enemy_ap = 0;
    let mut enemy_ad = 0;
    let mut any_healing = false;

    for (i, champ) in enemy_champions.iter().enumerate() {
        let tags = db.get_champion_tags(champ).unwrap_or_default();
        let mut dmg_type = damage_type_from_tags(&tags);

        // Check if building opposite items
        if let Some(items) = enemy_items.get(i) {
            let (ap_count, ad_count) = count_item_types(db, items);
            if dmg_type == "AP" && ad_count > ap_count && ad_count >= 2 { dmg_type = "AD".to_string(); }
            if dmg_type == "AD" && ap_count > ad_count && ap_count >= 2 { dmg_type = "AP".to_string(); }
        }

        let (kills, deaths, gold_est) = enemy_stats.get(i).copied().unwrap_or((0, 0, 0));
        let kda = if deaths == 0 { kills + 3 } else { (kills * 2 + 3) / (deaths + 1) };
        let is_fed = kills >= 4 || kda >= 3 || gold_est > 8000;
        let is_weak = deaths >= 4 && kills <= 1;
        let has_healing = healers.contains(&champ.as_str());

        if dmg_type == "AP" { enemy_ap += 1; } else if dmg_type == "AD" { enemy_ad += 1; } else { enemy_ap += 1; enemy_ad += 1; }
        if has_healing && !is_weak { any_healing = true; }

        threats.push(ThreatInfo {
            champion: champ.clone(), damage_type: dmg_type,
            threat_level: if is_fed { "HIGH".into() } else if is_weak { "LOW".into() } else { "MED".into() },
            is_fed, is_weak, has_healing, kills, deaths, gold: gold_est,
        });
    }
    threats.sort_by(|a, b| b.gold.cmp(&a.gold));

    let total = (enemy_ap + enemy_ad).max(1);
    let enemy_damage = DamageBreakdown {
        ad_pct: (enemy_ad * 100 / total) as i32,
        ap_pct: (enemy_ap * 100 / total) as i32,
    };

    // ── Detect build paths from owned components ──
    let build_path = detect_build_paths(db, my_item_ids, &my_items_set, gold, &my_class);

    // ── Generate recommendations ──
    let mut recommendations = Vec::new();
    let has_boots = my_item_ids.iter().any(|id| [3006,3009,3020,3047,3111,3117,3158].contains(id));
    let has_antiheal = my_item_ids.iter().any(|id| [3165,3033,3075,3011].contains(id));

    // Boots
    if !has_boots {
        let boot = recommend_boots(&my_class, enemy_damage.ad_pct, enemy_damage.ap_pct);
        if let Some(b) = boot {
            if let Some(info) = resolve_item(db, b.0, &my_items_set) {
                recommendations.push(ItemRecommendation {
                    item_id: b.0, item_name: info.0, cost: info.1,
                    tag: "BUY".into(), reason: b.1.to_string(), category: "boots".into(),
                    can_afford: gold >= info.1, from_items: info.2,
                });
            }
        }
    }

    // Defensive vs fed threats
    let fed_ap: Vec<&ThreatInfo> = threats.iter().filter(|t| t.is_fed && t.damage_type == "AP").collect();
    let fed_ad: Vec<&ThreatInfo> = threats.iter().filter(|t| t.is_fed && t.damage_type == "AD").collect();

    if !fed_ap.is_empty() {
        if let Some(item) = find_defensive_item(db, "SpellBlock", &my_items_set) {
            let reason = format!("{} is {}/{} (AP)", fed_ap[0].champion, fed_ap[0].kills, fed_ap[0].deaths);
            recommendations.push(ItemRecommendation {
                item_id: item.0, item_name: item.1.clone(), cost: item.2,
                tag: "RUSH".into(), reason, category: "defensive".into(),
                can_afford: gold >= item.2, from_items: item.3,
            });
        }
    }
    if !fed_ad.is_empty() {
        if let Some(item) = find_defensive_item(db, "Armor", &my_items_set) {
            let reason = format!("{} is {}/{} (AD)", fed_ad[0].champion, fed_ad[0].kills, fed_ad[0].deaths);
            recommendations.push(ItemRecommendation {
                item_id: item.0, item_name: item.1.clone(), cost: item.2,
                tag: "RUSH".into(), reason, category: "defensive".into(),
                can_afford: gold >= item.2, from_items: item.3,
            });
        }
    }

    // Offensive by class
    if let Some(item) = find_offensive_item(db, &my_class, &my_damage_type, &my_items_set) {
        recommendations.push(ItemRecommendation {
            item_id: item.0, item_name: item.1.clone(), cost: item.2,
            tag: "CORE".into(), reason: item.4, category: "offensive".into(),
            can_afford: gold >= item.2, from_items: item.3,
        });
    }

    // Anti-heal
    if any_healing && !has_antiheal {
        let heal_champs: Vec<String> = threats.iter().filter(|t| t.has_healing && !t.is_weak).map(|t| t.champion.clone()).collect();
        let ah_id = if my_damage_type == "AP" { 3165 } else { 3033 };
        if let Some(info) = resolve_item(db, ah_id, &my_items_set) {
            recommendations.push(ItemRecommendation {
                item_id: ah_id, item_name: info.0, cost: info.1,
                tag: "BUY".into(), reason: format!("Anti-heal for {}", heal_champs.join(", ")),
                category: "utility".into(), can_afford: gold >= info.1, from_items: info.2,
            });
        }
    }

    // ── "On next back" -- smart gold spending ──
    let on_next_back = compute_buyable(&recommendations, &build_path, gold, &my_items_set, db);

    // ── Warnings ──
    let mut warnings = Vec::new();
    if enemy_damage.ap_pct >= 70 && !my_item_ids.iter().any(|id| is_mr_item(db, *id)) {
        warnings.push(format!("Enemy team is {}% AP damage -- consider MR", enemy_damage.ap_pct));
    }
    if enemy_damage.ad_pct >= 70 && !my_item_ids.iter().any(|id| is_armor_item(db, *id)) {
        warnings.push(format!("Enemy team is {}% AD damage -- consider Armor", enemy_damage.ad_pct));
    }

    ItemIntelligence {
        my_class, my_damage_type, enemy_damage,
        build_path, recommendations, on_next_back, threats, warnings,
    }
}

// ── Helper functions ──

fn classify_champion(tags: &[String]) -> String {
    let first = tags.first().map(|s| s.as_str()).unwrap_or("Fighter");
    match first {
        "Marksman" => "marksman",
        "Mage" => "mage",
        "Assassin" => "assassin",
        "Tank" => "tank",
        "Support" => "support",
        _ => "fighter",
    }.to_string()
}

fn damage_type_from_tags(tags: &[String]) -> String {
    let tag_strs: Vec<&str> = tags.iter().map(|s| s.as_str()).collect();
    if tag_strs.contains(&"Mage") || tag_strs.contains(&"Support") { "AP".to_string() }
    else if tag_strs.contains(&"Marksman") || tag_strs.contains(&"Assassin") { "AD".to_string() }
    else if tag_strs.contains(&"Fighter") { "AD".to_string() }
    else if tag_strs.contains(&"Tank") { "AP".to_string() } // Most tanks do AP
    else { "AD".to_string() }
}

fn count_item_types(db: &Arc<Database>, item_ids: &[i64]) -> (i32, i32) {
    let mut ap = 0;
    let mut ad = 0;
    for id in item_ids {
        if let Ok(Some(info)) = db.get_item_info(*id) {
            let tags = info.get("tags").and_then(|t| t.as_str()).unwrap_or("");
            if tags.contains("SpellDamage") || tags.contains("ManaRegen") { ap += 1; }
            if tags.contains("Damage") && !tags.contains("SpellDamage") { ad += 1; }
        }
    }
    (ap, ad)
}

fn detect_build_paths(
    db: &Arc<Database>, my_items: &[i64], owned: &std::collections::HashSet<i64>,
    gold: i64, my_class: &str,
) -> Vec<BuildPathItem> {
    let mut paths = Vec::new();
    for &item_id in my_items {
        if item_id <= 0 { continue; }
        let upgrades = db.get_items_building_from(item_id).unwrap_or_default();
        if upgrades.is_empty() { continue; }

        let component_name = db.get_item_info(item_id)
            .ok().flatten()
            .and_then(|i| i.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| format!("Item {item_id}"));

        let mut targets: Vec<BuildTarget> = Vec::new();
        for upgrade in &upgrades {
            let uid = upgrade.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
            let uname = upgrade.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
            let ucost = upgrade.get("gold_total").and_then(|v| v.as_i64()).unwrap_or(3000);
            if owned.contains(&uid) || ucost < 1000 { continue; }

            // Calculate remaining cost (total - components already owned)
            let from_str = upgrade.get("from_items").and_then(|v| v.as_str()).unwrap_or("[]");
            let from_ids: Vec<i64> = serde_json::from_str(from_str).unwrap_or_default();
            let mut owned_value: i64 = 0;
            let mut components = Vec::new();
            for &fid in &from_ids {
                let comp_info = db.get_item_info(fid).ok().flatten();
                let comp_cost = comp_info.as_ref().and_then(|i| i.get("gold_total").and_then(|v| v.as_i64())).unwrap_or(0);
                let comp_name = comp_info.as_ref().and_then(|i| i.get("name").and_then(|v| v.as_str())).unwrap_or("?").to_string();
                let is_owned = owned.contains(&fid);
                if is_owned { owned_value += comp_cost; }
                components.push(ComponentInfo { id: fid, name: comp_name, cost: comp_cost, owned: is_owned });
            }

            let remaining = (ucost - owned_value).max(0);
            let relevance = score_item_relevance(&uname, my_class);

            targets.push(BuildTarget {
                item_id: uid, item_name: uname, total_cost: ucost,
                remaining_cost: remaining, can_afford: gold >= remaining,
                relevance, from_items: components,
            });
        }

        targets.sort_by_key(|t| t.remaining_cost);
        if !targets.is_empty() {
            paths.push(BuildPathItem {
                component_id: item_id, component_name, builds_into: targets,
            });
        }
    }
    paths
}

fn score_item_relevance(item_name: &str, class: &str) -> String {
    let name = item_name.to_lowercase();
    let is_ad = name.contains("infinity") || name.contains("bloodthirster") || name.contains("collector")
        || name.contains("phantom") || name.contains("dominik") || name.contains("mortal");
    let is_ap = name.contains("rabadon") || name.contains("zhonya") || name.contains("shadowflame")
        || name.contains("morello") || name.contains("void");

    match class {
        "marksman" | "fighter" if is_ad => "HIGH",
        "mage" | "support" if is_ap => "HIGH",
        "assassin" if is_ad => "HIGH",
        _ if is_ad || is_ap => "MED",
        _ => "LOW",
    }.to_string()
}

fn recommend_boots(class: &str, ad_pct: i32, ap_pct: i32) -> Option<(i64, &'static str)> {
    if ad_pct >= 60 { return Some((3047, "vs AD-heavy team")); }
    if ap_pct >= 60 { return Some((3111, "vs AP + CC")); }
    match class {
        "marksman" => Some((3006, "Attack speed for DPS")),
        "mage" | "support" => Some((3020, "Magic penetration")),
        "assassin" => Some((3117, "Mobility for roaming")),
        "tank" => Some((3047, "Armor for tanking")),
        _ => Some((3047, "General defense")),
    }
}

fn find_defensive_item(
    db: &Arc<Database>, tag: &str, owned: &std::collections::HashSet<i64>,
) -> Option<(i64, String, i64, Vec<ComponentInfo>)> {
    let items = db.get_items_by_tag(tag).unwrap_or_default();
    for item in items {
        let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        if owned.contains(&id) { continue; }
        if let Some(info) = resolve_item(db, id, owned) {
            return Some((id, info.0, info.1, info.2));
        }
    }
    None
}

fn find_offensive_item(
    db: &Arc<Database>, class: &str, dmg_type: &str, owned: &std::collections::HashSet<i64>,
) -> Option<(i64, String, i64, Vec<ComponentInfo>, String)> {
    let tag = if dmg_type == "AP" { "SpellDamage" } else { "Damage" };
    let items = db.get_items_by_tag(tag).unwrap_or_default();

    for item in items {
        let id = item.get("id").and_then(|v| v.as_i64()).unwrap_or(0);
        let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("?").to_string();
        if owned.contains(&id) { continue; }
        // Skip items that don't fit the class
        let relevance = score_item_relevance(&name, class);
        if relevance == "LOW" { continue; }
        if let Some(info) = resolve_item(db, id, owned) {
            let reason = format!("{} item for {}", dmg_type, class);
            return Some((id, info.0, info.1, info.2, reason));
        }
    }
    None
}

fn resolve_item(
    db: &Arc<Database>, item_id: i64, owned: &std::collections::HashSet<i64>,
) -> Option<(String, i64, Vec<ComponentInfo>)> {
    let info = db.get_item_info(item_id).ok()??;
    let name = info.get("name").and_then(|v| v.as_str())?.to_string();
    let cost = info.get("gold_total").and_then(|v| v.as_i64()).unwrap_or(3000);
    let from_str = info.get("from_items").and_then(|v| v.as_str()).unwrap_or("[]");
    let from_ids: Vec<i64> = serde_json::from_str(from_str).unwrap_or_default();

    let components: Vec<ComponentInfo> = from_ids.iter().map(|&fid| {
        let comp = db.get_item_info(fid).ok().flatten();
        ComponentInfo {
            id: fid,
            name: comp.as_ref().and_then(|c| c.get("name").and_then(|n| n.as_str())).unwrap_or("?").to_string(),
            cost: comp.as_ref().and_then(|c| c.get("gold_total").and_then(|n| n.as_i64())).unwrap_or(0),
            owned: owned.contains(&fid),
        }
    }).collect();

    Some((name, cost, components))
}

fn compute_buyable(
    recs: &[ItemRecommendation], paths: &[BuildPathItem],
    gold: i64, owned: &std::collections::HashSet<i64>, db: &Arc<Database>,
) -> Vec<BuyableItem> {
    let mut buyable = Vec::new();

    // 1. Complete recommended items you can afford
    for r in recs {
        if r.can_afford && !owned.contains(&r.item_id) {
            buyable.push(BuyableItem {
                item_id: r.item_id, item_name: r.item_name.clone(),
                cost: r.cost, is_complete: true,
                context: format!("{} -- {}", r.tag, r.reason),
            });
        }
    }

    // 2. Complete build path targets you can afford
    for path in paths {
        for target in &path.builds_into {
            if target.can_afford && !owned.contains(&target.item_id) && !buyable.iter().any(|b| b.item_id == target.item_id) {
                buyable.push(BuyableItem {
                    item_id: target.item_id, item_name: target.item_name.clone(),
                    cost: target.remaining_cost, is_complete: true,
                    context: format!("Complete from {}", path.component_name),
                });
            }
        }
    }

    // 3. Components of recommended items if can't afford complete
    if buyable.is_empty() {
        for r in recs {
            for comp in &r.from_items {
                if gold >= comp.cost && !comp.owned && !buyable.iter().any(|b| b.item_id == comp.id) {
                    buyable.push(BuyableItem {
                        item_id: comp.id, item_name: comp.name.clone(),
                        cost: comp.cost, is_complete: false,
                        context: format!("Component for {}", r.item_name),
                    });
                }
            }
        }
    }

    // 4. Control ward
    if gold >= 75 {
        buyable.push(BuyableItem {
            item_id: 2055, item_name: "Control Ward".into(),
            cost: 75, is_complete: false, context: "Vision".into(),
        });
    }

    buyable.truncate(5);
    buyable
}

fn is_mr_item(db: &Arc<Database>, item_id: i64) -> bool {
    db.get_item_info(item_id).ok().flatten()
        .and_then(|i| i.get("tags").and_then(|t| t.as_str()).map(|s| s.contains("SpellBlock")))
        .unwrap_or(false)
}

fn is_armor_item(db: &Arc<Database>, item_id: i64) -> bool {
    db.get_item_info(item_id).ok().flatten()
        .and_then(|i| i.get("tags").and_then(|t| t.as_str()).map(|s| s.contains("Armor")))
        .unwrap_or(false)
}
