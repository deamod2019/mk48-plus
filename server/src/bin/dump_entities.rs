// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later
//
// 用法（在 server 目录）:
//   cargo run --bin dump_entities
//
// 功能:
//   遍历 EntityType，把武器/舰船信息导出到 client/src/entities_data.json，
//   供 ships_new 页签或其他离线展示使用。可随时手工运行刷新 JSON。

use std::fs;
use std::path::PathBuf;

use std::collections::HashMap;
use std::fs::read_to_string;
use std::str::FromStr;

use common::entity::{Armament, EntityData, EntityKind, EntityType, Turret};
use common::ticks::Ticks;
use serde::Serialize;

#[derive(Serialize)]
struct WeaponOut {
    id: String,
    label: String,
    kind: String,
    damage: f32,
    reload: f32,
    speed: f32,
    range: f32,
}

#[derive(Serialize)]
struct ArmOut {
    weapon_id: String,
    count: u8,
    range: f32,
    notes: Option<String>,
}

#[derive(Serialize)]
struct ShipOut {
    id: String,
    label: String,
    level: u8,
    kind: String,
    sub_kind: String,
    speed: f32,
    health: f32,
    length: f32,
    draft: f32,
    range: f32,
    depth: f32,
    reload: f32,
    anti_aircraft: f32,
    torpedo_resistance: f32,
    stealth: f32,
    npc: bool,
    armaments: Vec<ArmOut>,
}

#[derive(Serialize)]
struct Out {
    weapons: Vec<WeaponOut>,
    ships: Vec<ShipOut>,
}

#[derive(Default, Clone)]
struct PropsPatch {
    range: Option<f32>,
    speed: Option<f32>,
    reload: Option<f32>,
    damage: Option<f32>,
}

fn ticks_to_secs(t: Ticks) -> f32 {
    t.to_secs() as f32
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut weapons = Vec::new();
    let mut weapon_ranges: HashMap<String, f32> = HashMap::new();
    let mut ships = Vec::new();
    let props_patch = parse_props_patch()?;

    for t in EntityType::iter() {
        let d: &EntityData = t.data();
        match d.kind {
            EntityKind::Weapon => {
                let mut w = WeaponOut {
                    id: t.as_str().to_string(),
                    label: d.label.to_string(),
                    kind: format!("{:?}", d.sub_kind),
                    damage: d.damage,
                    reload: ticks_to_secs(d.reload),
                    speed: d.speed.to_mps(),
                    range: d.range,
                };
                if let Some(p) = props_patch.get(&w.id) {
                    if let Some(r) = p.range { w.range = r; }
                    if let Some(spd) = p.speed { w.speed = spd; }
                    if let Some(rel) = p.reload { w.reload = rel; }
                    if let Some(dmg) = p.damage { w.damage = dmg; }
                }
                weapon_ranges.insert(w.id.clone(), w.range);
                weapons.push(w);
            }
            EntityKind::Boat => {
                let mut armaments = collect_armaments(d.armaments, &weapon_ranges);
                let turrets = collect_turrets(d.turrets, &weapon_ranges);
                armaments.extend(turrets);
                let max_weapon_range = armaments.iter().map(|a| a.range).fold(0.0, f32::max);
                let ship_range = if d.range > 0.0 {
                    d.range
                } else if max_weapon_range > 0.0 {
                    max_weapon_range
                } else {
                    d.sensors.visual.range
                };

                ships.push(ShipOut {
                    id: t.as_str().to_string(),
                    label: d.label.to_string(),
                    level: d.level,
                    kind: format!("{:?}", d.kind),
                    sub_kind: format!("{:?}", d.sub_kind),
                    speed: d.speed.to_mps(),
                    health: d.damage,
                    length: d.length,
                    draft: d.draft.to_meters(),
                    range: ship_range,
                    depth: d.depth.to_meters(),
                    reload: ticks_to_secs(d.reload),
                    anti_aircraft: d.anti_aircraft,
                    torpedo_resistance: d.torpedo_resistance,
                    stealth: d.stealth,
                    npc: d.npc,
                    armaments,
                });
            }
            _ => {}
        }
    }

    // 输出路径: client/src/entities_data.json
    let out_path: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../client/src/entities_data.json");
    let out = Out { weapons, ships };
    let json = serde_json::to_string_pretty(&out)?;

    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&out_path, json)?;

    println!("Wrote {}", out_path.display());
    Ok(())
}

fn collect_armaments(list: &[Armament], weapon_ranges: &HashMap<String, f32>) -> Vec<ArmOut> {
    let mut counts: HashMap<String, (u8, f32)> = HashMap::new();
    for a in list {
        let key = a.entity_type.as_str().to_string();
        let r = *weapon_ranges.get(&key).unwrap_or(&0.0);
        counts
            .entry(key)
            .and_modify(|entry| {
                entry.0 = entry.0.saturating_add(1);
                entry.1 = entry.1.max(r);
            })
            .or_insert((1, r));
    }
    counts
        .into_iter()
        .map(|(weapon_id, (count, range))| ArmOut {
            weapon_id,
            count,
            range,
            notes: None,
        })
        .collect()
}

fn collect_turrets(list: &[Turret], weapon_ranges: &HashMap<String, f32>) -> Vec<ArmOut> {
    let mut counts: HashMap<String, (u8, f32)> = HashMap::new();
    for t in list {
        if let Some(entity_type) = t.entity_type {
            let key = entity_type.as_str().to_string();
            let r = *weapon_ranges.get(&key).unwrap_or(&0.0);
            counts
                .entry(key)
                .and_modify(|entry| {
                    entry.0 = entry.0.saturating_add(1);
                    entry.1 = entry.1.max(r);
                })
                .or_insert((1, r));
        }
    }
    counts
        .into_iter()
        .map(|(weapon_id, (count, range))| ArmOut {
            weapon_id,
            count,
            range,
            notes: None,
        })
        .collect()
}

fn parse_props_patch() -> Result<HashMap<String, PropsPatch>, Box<dyn std::error::Error>> {
    let mut map = HashMap::new();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../common/src/entity/_type.rs");
    let content = read_to_string(path)?;
    let mut in_weapon = false;
    let mut pending: PropsPatch = PropsPatch::default();
    let mut _pending_for: Option<String> = None;

    for line in content.lines() {
        let l = line.trim();
        if l.starts_with("#[entity(Weapon") {
            in_weapon = true;
        } else if l.starts_with("#[entity(") {
            in_weapon = false;
            pending = PropsPatch::default();
            _pending_for = None;
        } else if l.starts_with("#[props(") && in_weapon {
            let props_str = l.trim_start_matches("#[props(").trim_end_matches(')');
            for part in props_str.split(',') {
                let p = part.trim();
                if let Some(v) = p.strip_prefix("range = ") {
                    if let Ok(f) = f32::from_str(v) { pending.range = Some(f); }
                } else if let Some(v) = p.strip_prefix("speed = ") {
                    if let Ok(f) = f32::from_str(v) { pending.speed = Some(f); }
                } else if let Some(v) = p.strip_prefix("reload = ") {
                    if let Ok(f) = f32::from_str(v) { pending.reload = Some(f); }
                } else if let Some(v) = p.strip_prefix("damage = ") {
                    if let Ok(f) = f32::from_str(v) { pending.damage = Some(f); }
                }
            }
        } else if l.ends_with(',') {
            let name = l.trim_end_matches(',').trim();
            if !name.is_empty() && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                if in_weapon && pending_has_value(&pending) {
                    map.insert(name.to_string(), pending.clone());
                }
                _pending_for = Some(name.to_string());
                pending = PropsPatch::default();
            }
        }
    }
    Ok(map)
}

fn pending_has_value(p: &PropsPatch) -> bool {
    p.range.is_some() || p.speed.is_some() || p.reload.is_some() || p.damage.is_some()
}
