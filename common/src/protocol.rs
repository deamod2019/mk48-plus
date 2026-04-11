// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::contact::Contact;
use crate::death_reason::DeathReason;
use crate::entity::*;
use crate::guidance::Guidance;
use crate::skill::SkillType;
use crate::terrain::{ChunkId, SerializedChunk};
use crate::ticks::Ticks;
use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Server to client update.
#[cfg_attr(feature = "server", derive(actix::Message))]
#[cfg_attr(feature = "server", rtype(result = "()"))]
#[derive(Debug, Serialize, Deserialize)]
pub struct Update {
    /// All currently visible contacts.
    pub contacts: Vec<Contact>,
    /// World events (1-tick, non-persistent).
    pub events: Vec<WorldEvent>,
    /// Why the player died, if they died, otherwise None.
    pub death_reason: Option<DeathReason>,
    /// Player's current score.
    pub score: u32,
    /// Per-entity-type kill counts for this player's current session.
    #[serde(default)]
    pub kill_log: Vec<(EntityType, u32)>,
    /// Current world border radius.
    pub world_radius: f32,
    pub terrain: Box<TerrainUpdate>,
    /// Whether bot alliance mode is enabled (high-score bots form alliance against players).
    #[serde(default)]
    pub bot_alliance_enabled: bool,
    /// Runtime skill state snapshots for the current player.
    #[serde(default)]
    pub skills: Vec<SkillSnapshot>,
    /// Faction war data (present when faction_mode is enabled).
    #[serde(default)]
    pub faction_data: Option<FactionUpdate>,
    /// Current player's faction.
    #[serde(default)]
    pub my_faction: Option<FactionId>,
    /// Known altar position for this player's faction (None = not yet discovered).
    #[serde(default)]
    pub altar_position: Option<Vec2>,
    /// Current altar sacrifice progress for all factions.
    #[serde(default)]
    pub altar_sacrifice_counts: Vec<u8>,
    /// Whether this update comes from an arena instance.
    #[serde(default)]
    pub arena_mode: bool,
    /// Current faction mode (None = faction mode disabled).
    #[serde(default)]
    pub faction_mode: Option<FactionMode>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum WorldEvent {
    ZeroPulse { center: Vec2, radius: f32 },
    NuclearStrike { center: Vec2, radius: f32 },
    /// Internal event: a boat sacrificed to the altar (server-only, filtered before sending).
    AltarSacrifice { faction: FactionId },
    /// Altar discovered by a faction — broadcast to that faction's players.
    AltarDiscovered { position: Vec2, faction: FactionId },
    /// Altar consumed — a faction completed 5 sacrifices.
    AltarConsumed { faction: FactionId },
}

/// Updates for terrain chunks.
pub type TerrainUpdate = [(ChunkId, SerializedChunk)];

/// Faction mode: determines how teams are arranged.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FactionMode {
    /// 2-team battle: 三（4）舰队 vs 三（3）舰队
    TwoTeam = 1,
    /// 3-team FFA: 红军 vs 蓝军 vs 绿军
    ThreeTeam = 2,
}

impl FactionMode {
    /// Number of factions for this mode.
    pub fn faction_count(&self) -> usize {
        match self {
            Self::TwoTeam => 2,
            Self::ThreeTeam => 3,
        }
    }

    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::TwoTeam),
            2 => Some(Self::ThreeTeam),
            _ => None,
        }
    }
}

impl Default for FactionMode {
    fn default() -> Self {
        Self::ThreeTeam
    }
}

/// Faction identifier for the faction war mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FactionId {
    Red = 0,
    Blue = 1,
    Green = 2,
}

impl FactionId {
    /// Maximum number of factions (used for fixed-size arrays).
    pub const MAX_COUNT: usize = 3;
    /// Legacy constant for backward compatibility.
    pub const COUNT: usize = 3;
    /// Maximum players per faction.
    pub const MAX_PLAYERS_PER_FACTION: u32 = 34;

    pub fn from_index(i: u8, mode: FactionMode) -> Self {
        let count = mode.faction_count() as u8;
        match i % count {
            0 => Self::Red,
            1 => Self::Blue,
            _ => Self::Green,
        }
    }

    /// Name of this faction in the given mode.
    pub fn name(&self, mode: FactionMode) -> &'static str {
        match mode {
            FactionMode::TwoTeam => match self {
                Self::Red => "三（4）舰队",
                Self::Blue => "三（3）舰队",
                Self::Green => "三（3）舰队", // unused in TwoTeam
            },
            FactionMode::ThreeTeam => match self {
                Self::Red => "红军",
                Self::Blue => "蓝军",
                Self::Green => "绿军",
            },
        }
    }

    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn emoji(&self, mode: FactionMode) -> &'static str {
        match mode {
            FactionMode::TwoTeam => match self {
                Self::Red => "🔴",
                Self::Blue => "🔵",
                Self::Green => "🔵",
            },
            FactionMode::ThreeTeam => match self {
                Self::Red => "🔴",
                Self::Blue => "🔵",
                Self::Green => "🟢",
            },
        }
    }

    /// Iterate all factions for a given mode.
    pub fn iter_for_mode(mode: FactionMode) -> impl Iterator<Item = FactionId> {
        (0..mode.faction_count()).map(move |i| Self::from_index(i as u8, mode))
    }
}

/// Per-faction statistics sent to the client.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FactionStats {
    pub total_score: u64,
    pub player_count: u32,
    pub top_player: Option<String>,
    pub top_score: u32,
}

/// Faction war update data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FactionUpdate {
    pub factions: Vec<FactionStats>,
    /// Per-player faction assignments (for rendering faction markers above ships).
    #[serde(default)]
    pub player_factions: Vec<(core_protocol::id::PlayerId, FactionId)>,
}

/// Client to server commands.
#[derive(Clone, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "server", derive(actix::Message))]
#[cfg_attr(feature = "server", rtype(result = "()"))]
pub enum Command {
    Control(Control),
    Spawn(Spawn),
    Upgrade(Upgrade),
    UseSkill(UseSkill),
    Warp(Warp),
    ZeroPulse(ZeroPulse),
    Iaigiri(Iaigiri),
    EngineBoost(EngineBoost),
    SonarPulse(SonarPulse),
    DepthChargeBarrage(DepthChargeBarrage),
    AirSuperiority(AirSuperiority),
    EmergencyRepair(EmergencyRepair),
    SmokeScreen(SmokeScreen),
    BurstLoading(BurstLoading),
    NuclearStrike(NuclearStrike),
    EnergyShield(EnergyShield),
    DredgerSacrifice(DredgerSacrifice),
    Stealth(Stealth),
    UnjustGame(UnjustGame),
    Ironclad(Ironclad),
    YamatoCannon(YamatoCannon),
    OrbitalBombardment(OrbitalBombardment),
    RiftStorm(RiftStorm),
    SetFactionMode(SetFactionMode),
    Cheat(CheatCommand),
}

/// Generic command to control one's ship.
#[derive(Clone, Serialize, PartialEq, Deserialize, Debug)]
pub struct Control {
    /// Steering commands.
    pub guidance: Option<Guidance>,
    /// Submerge submarine.
    pub submerge: bool,
    /// Turret/aircraft/pay target.
    pub aim_target: Option<Vec2>,
    /// Active sensors.
    pub active: bool,
    /// Fire weapon a weapon.
    pub fire: Option<Fire>,
    /// Pay one coin.
    pub pay: Option<Pay>,
    /// Optional hints.
    pub hint: Option<Hint>,
    /// Horn Volume
    pub horn: bool,
}

/// Fire/use a single weapon.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Fire {
    /// The index of the weapon to fire/use, relative to `EntityData.armaments`.
    pub armament_index: u16,
}

/// Provide hints to optimize experience.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Hint {
    /// aspect ratio of screen (width / height).
    /// Allows the server to send the correct amount of terrain.
    pub aspect: f32,
}

impl Default for Hint {
    fn default() -> Self {
        Self { aspect: 1.0 }
    }
}

/// Pay one coin. TODO: Can't use Option<empty struct>, as serde_json serializes both [`None`] and
/// [`Some`] to `"null"`.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub struct Pay;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Spawn {
    /// What to spawn as. Must be an affordable boat.
    pub entity_type: EntityType,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Upgrade {
    /// What to upgrade to. Must be an affordable boat of higher level.
    pub entity_type: EntityType,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct UseSkill {
    /// Skill being used.
    pub skill: SkillType,
    /// Explicit target payload, if any.
    pub target: SkillTarget,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum SkillTarget {
    None,
    Position(Vec2),
    Entity(EntityId),
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct SkillSnapshot {
    pub skill: SkillType,
    pub cooldown_remaining: Ticks,
    pub active_remaining: Ticks,
    pub charge_remaining: Ticks,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Warp {
    /// 目标世界坐标，服务器会再次裁剪。
    pub target: Vec2,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ZeroPulse;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Iaigiri {
    pub target: Vec2,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EngineBoost;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SonarPulse;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DepthChargeBarrage;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct AirSuperiority;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EmergencyRepair;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SmokeScreen;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct BurstLoading;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NuclearStrike;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct EnergyShield;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DredgerSacrifice;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Stealth;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UnjustGame {
    /// Target entity to swap with.
    pub target_id: EntityId,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Ironclad;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct YamatoCannon;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct OrbitalBombardment;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RiftStorm;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct SetFactionMode {
    pub mode: FactionMode,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CheatCommand {
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::altitude::Altitude;
    use crate::contact::ReloadsStorage;
    use crate::entity::EntityId;
    use crate::guidance::Guidance;
    use crate::ticks::Ticks;
    use crate::transform::Transform;
    use crate::velocity::Velocity;
    use bincode::{DefaultOptions, Options};
    use bitvec::array::BitArray;
    use core_protocol::id::PlayerId;
    use glam::vec2;
    use rand::prelude::*;
    use std::num::NonZeroU32;

    #[test]
    fn serialize() {
        EntityType::from_str(EntityType::Barrel.as_str()).unwrap();

        let mut rng = thread_rng();
        for _ in 0..10000 {
            let entity_type: Option<EntityType> = rng
                .gen_bool(0.5)
                .then(|| EntityType::iter().choose(&mut rng).unwrap());
            let is_boat = entity_type.map_or(false, |t| t.data().kind == EntityKind::Boat);

            let c = Contact::new(
                Altitude::from_u8(rng.gen()),
                Ticks::from_secs(rng.gen::<f32>() * 10.0),
                entity_type,
                Guidance {
                    direction_target: rng.gen(),
                    velocity_target: Velocity::from_mps(rng.gen::<f32>() * 3.0),
                },
                EntityId::new(rng.gen_range(1..u32::MAX)).unwrap(),
                rng.gen_bool(0.5)
                    .then(|| PlayerId(NonZeroU32::new(rng.gen_range(1..u32::MAX)).unwrap())),
                (is_boat && rng.gen_bool(0.5)).then(|| {
                    let mut arr = BitArray::<ReloadsStorage>::ZERO;
                    for (_, mut r) in entity_type
                        .unwrap()
                        .data()
                        .armaments
                        .iter()
                        .zip(arr.iter_mut())
                    {
                        *r = rng.gen();
                    }
                    arr
                }),
                Transform {
                    position: vec2(
                        rng.gen::<f32>() * 1000.0 - 500.0,
                        rng.gen::<f32>() * 1000.0 - 500.0,
                    ),
                    velocity: Velocity::from_mps(rng.gen::<f32>() * 3.0),
                    direction: rng.gen(),
                },
                is_boat.then(|| {
                    entity_type
                        .unwrap()
                        .data()
                        .turrets
                        .iter()
                        .map(|_| rng.gen())
                        .collect()
                }),
                rng.gen_bool(0.5), // horn
            );

            let options = DefaultOptions::new()
                .with_fixint_encoding()
                .allow_trailing_bytes();

            let bytes = options.serialize(&c).unwrap();

            match options.deserialize::<Contact>(&bytes) {
                Ok(contact) => {
                    assert_eq!(c, contact)
                }
                Err(err) => {
                    println!("len: {}, bytes: {:?}", bytes.len(), &bytes);
                    println!("contact: {:?}", &c);

                    let byte = bytes[0];
                    for i in 0u32..8 {
                        println!("byte {}: {}", i, byte & (1 << i) != 0)
                    }
                    panic!("{}", err);
                }
            }
        }
    }
}
