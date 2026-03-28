use std::collections::HashMap;

/// Rune tree name → style ID mapping
/// [S5 fix] Returns Option instead of silently defaulting
pub fn tree_id(name: &str) -> Option<i64> {
    match name.to_lowercase().as_str() {
        "precision" => Some(8000),
        "domination" => Some(8100),
        "sorcery" => Some(8200),
        "resolve" => Some(8400),
        "inspiration" => Some(8300),
        _ => {
            tracing::warn!("Unknown rune tree: {name}");
            None
        }
    }
}

/// Build a comprehensive rune name → ID map
pub fn rune_id_map() -> HashMap<String, i64> {
    let mut m = HashMap::new();

    // ── Precision ──
    m.insert("Press the Attack".into(), 8005);
    m.insert("Lethal Tempo".into(), 8008);
    m.insert("Fleet Footwork".into(), 8021);
    m.insert("Conqueror".into(), 8010);
    m.insert("Overheal".into(), 9101);
    m.insert("Triumph".into(), 9111);
    m.insert("Presence of Mind".into(), 8009);
    // [S4 fix] Removed stale Legend: Alacrity (now Legend: Haste) and Triumph (now Absorb Life)
    m.insert("Legend: Haste".into(), 9104);
    m.insert("Legend: Tenacity".into(), 9105);
    m.insert("Legend: Bloodline".into(), 9103);
    m.insert("Absorb Life".into(), 9111);
    m.insert("Coup de Grace".into(), 8014);
    m.insert("Cut Down".into(), 8017);
    m.insert("Last Stand".into(), 8299);

    // ── Domination ──
    m.insert("Electrocute".into(), 8112);
    m.insert("Predator".into(), 8124);
    m.insert("Dark Harvest".into(), 8128);
    m.insert("Hail of Blades".into(), 9923);
    m.insert("Cheap Shot".into(), 8126);
    m.insert("Taste of Blood".into(), 8139);
    m.insert("Sudden Impact".into(), 8143);
    m.insert("Zombie Ward".into(), 8136);
    m.insert("Ghost Poro".into(), 8120);
    m.insert("Eyeball Collection".into(), 8138);
    m.insert("Treasure Hunter".into(), 8135);
    m.insert("Ingenious Hunter".into(), 8134);
    m.insert("Relentless Hunter".into(), 8105);
    m.insert("Ultimate Hunter".into(), 8106);

    // ── Sorcery ──
    m.insert("Summon Aery".into(), 8214);
    m.insert("Arcane Comet".into(), 8229);
    m.insert("Phase Rush".into(), 8230);
    m.insert("Nullifying Orb".into(), 8224);
    m.insert("Manaflow Band".into(), 8226);
    m.insert("Nimbus Cloak".into(), 8275);
    m.insert("Transcendence".into(), 8210);
    m.insert("Celerity".into(), 8234);
    m.insert("Absolute Focus".into(), 8233);
    m.insert("Scorch".into(), 8237);
    m.insert("Waterwalking".into(), 8232);
    m.insert("Gathering Storm".into(), 8236);

    // ── Resolve ──
    m.insert("Grasp of the Undying".into(), 8437);
    m.insert("Aftershock".into(), 8439);
    m.insert("Guardian".into(), 8465);
    m.insert("Demolish".into(), 8446);
    m.insert("Font of Life".into(), 8463);
    m.insert("Shield Bash".into(), 8401);
    m.insert("Conditioning".into(), 8429);
    m.insert("Second Wind".into(), 8444);
    m.insert("Bone Plating".into(), 8473);
    m.insert("Overgrowth".into(), 8451);
    m.insert("Revitalize".into(), 8453);
    m.insert("Unflinching".into(), 8242);

    // ── Inspiration ──
    m.insert("Glacial Augment".into(), 8351);
    m.insert("Unsealed Spellbook".into(), 8360);
    m.insert("First Strike".into(), 8369);
    m.insert("Hextech Flashtraption".into(), 8306);
    m.insert("Magical Footwear".into(), 8304);
    m.insert("Cash Back".into(), 8321);
    m.insert("Triple Tonic".into(), 8313);
    m.insert("Time Warp Tonic".into(), 8352);
    m.insert("Biscuit Delivery".into(), 8345);
    m.insert("Cosmic Insight".into(), 8347);
    m.insert("Approach Velocity".into(), 8410);
    m.insert("Jack of All Trades".into(), 8316);

    // ── Stat mods ──
    m.insert("Adaptive Force".into(), 5008);
    m.insert("Attack Speed".into(), 5005);
    m.insert("Ability Haste".into(), 5007);
    m.insert("Move Speed".into(), 5010);
    m.insert("Health".into(), 5001);
    m.insert("Armor".into(), 5002);
    m.insert("Magic Resist".into(), 5003);
    m.insert("Health Scaling".into(), 5011);
    m.insert("Tenacity and Slow Resist".into(), 5013);

    m
}

/// Convert rune names to IDs using the mapping
pub fn resolve_rune_ids(rune_names: &[String]) -> Vec<i64> {
    let map = rune_id_map();
    rune_names
        .iter()
        .filter_map(|name| {
            map.get(name).copied().or_else(|| {
                // Try fuzzy match (lowercase)
                let lower = name.to_lowercase();
                map.iter()
                    .find(|(k, _)| k.to_lowercase() == lower)
                    .map(|(_, v)| *v)
            })
        })
        .collect()
}
