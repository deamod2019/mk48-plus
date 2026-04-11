// SPDX-FileCopyrightText: 2025 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Unified skill system for MK48+.
//!
//! This module consolidates all skill definitions into a single, type-safe structure.

use crate::ticks::Ticks;
use serde::{Deserialize, Serialize};

/// All available skill types in the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillType {
    /// Teleport to target location (used by Zumwalt)
    Warp,
    /// Temporary speed boost (used by Minelayer49)
    EngineBoost,
    /// Dash attack leaving mines behind (used by Minelayer49)
    Iaigiri,
    /// Active sonar pulse revealing submarines (used by HunterKiller77)
    SonarPulse,
    /// Launch multiple depth charges in a spread (used by HunterKiller77)
    DepthChargeBarrage,
    /// Deploy combat drones (used by FortressCarrier)
    AirSuperiority,
    /// Repair ship over time (used by FortressCarrier)
    EmergencyRepair,
    /// Deploy smoke screen for concealment (used by Tianwangxing)
    SmokeScreen,
    /// Freeze nearby enemies (used by StarDestroyer)
    ZeroPulse,
    /// Burst reload for faster firing (used by Richelieu)
    BurstLoading,
    /// Nuclear strike - destroy all enemies in range (used by UnscInfinite)
    NuclearStrike,
    /// Energy shield - absorbs damage for duration (used by StellarFrigate)
    EnergyShield,
    /// Dredger sacrifice - spawn oil platform, kill self (used by Dredger2)
    DredgerSacrifice,
    /// Stealth - become invisible to enemy radar (used by Assimilator2)
    Stealth,
    /// Unjust Game - swap position and motion state with target entity
    UnjustGame,
    /// Last Stand - auto-triggers on lethal damage, 10s berserk then death
    LastStand,
    /// Ironclad - reflect 30% damage to attacker for 15s
    Ironclad,
    /// Yamato Cannon - charge 3s, fire devastating frontal beam
    YamatoCannon,
    /// Orbital Bombardment - rain fire on area for 5s
    OrbitalBombardment,
    /// Rift Storm - spawn random explosions in radius
    RiftStorm,
}

/// What kind of explicit target a skill expects from the client.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillTargetKind {
    None,
    Position,
    Entity,
}

/// Whether a skill is user-triggered or passive.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SkillActivationKind {
    Active,
    Passive,
}

/// Skill metadata and parameters.
#[derive(Clone, Debug)]
pub struct SkillData {
    pub skill_type: SkillType,
    /// Display name
    pub label: &'static str,
    /// Chinese display name
    pub label_cn: &'static str,
    /// Short description
    pub description: &'static str,
    /// UI icon prefix
    pub icon: &'static str,
    /// UI hotkey
    pub hotkey: Option<char>,
    /// Explicit target type for this skill
    pub target_kind: SkillTargetKind,
    /// Whether this skill is active or passive
    pub activation_kind: SkillActivationKind,
    /// Time between skill uses
    pub cooldown: Ticks,
    /// How long the effect lasts (if applicable)
    pub duration: Option<Ticks>,
    /// Charge/cast time before activation (if applicable)
    pub charge_time: Option<Ticks>,
}

impl SkillType {
    /// Get the static data for this skill type.
    pub fn data(&self) -> &'static SkillData {
        match self {
            SkillType::Warp => &WARP_DATA,
            SkillType::EngineBoost => &ENGINE_BOOST_DATA,
            SkillType::Iaigiri => &IAIGIRI_DATA,
            SkillType::SonarPulse => &SONAR_PULSE_DATA,
            SkillType::DepthChargeBarrage => &DEPTH_CHARGE_BARRAGE_DATA,
            SkillType::AirSuperiority => &AIR_SUPERIORITY_DATA,
            SkillType::EmergencyRepair => &EMERGENCY_REPAIR_DATA,
            SkillType::SmokeScreen => &SMOKE_SCREEN_DATA,
            SkillType::ZeroPulse => &ZERO_PULSE_DATA,
            SkillType::BurstLoading => &BURST_LOADING_DATA,
            SkillType::NuclearStrike => &NUCLEAR_STRIKE_DATA,
            SkillType::EnergyShield => &ENERGY_SHIELD_DATA,
            SkillType::DredgerSacrifice => &DREDGER_SACRIFICE_DATA,
            SkillType::Stealth => &STEALTH_DATA,
            SkillType::UnjustGame => &UNJUST_GAME_DATA,
            SkillType::LastStand => &LAST_STAND_DATA,
            SkillType::Ironclad => &IRONCLAD_DATA,
            SkillType::YamatoCannon => &YAMATO_CANNON_DATA,
            SkillType::OrbitalBombardment => &ORBITAL_BOMBARDMENT_DATA,
            SkillType::RiftStorm => &RIFT_STORM_DATA,
        }
    }

    /// Returns all skill types.
    pub fn iter() -> impl Iterator<Item = SkillType> {
        IntoIterator::into_iter([
            SkillType::Warp,
            SkillType::EngineBoost,
            SkillType::Iaigiri,
            SkillType::SonarPulse,
            SkillType::DepthChargeBarrage,
            SkillType::AirSuperiority,
            SkillType::EmergencyRepair,
            SkillType::SmokeScreen,
            SkillType::ZeroPulse,
            SkillType::BurstLoading,
            SkillType::NuclearStrike,
            SkillType::EnergyShield,
            SkillType::DredgerSacrifice,
            SkillType::Stealth,
            SkillType::UnjustGame,
            SkillType::LastStand,
            SkillType::Ironclad,
            SkillType::YamatoCannon,
            SkillType::OrbitalBombardment,
            SkillType::RiftStorm,
        ])
    }

    /// Get the explicit target kind for this skill.
    pub fn target_kind(&self) -> SkillTargetKind {
        self.data().target_kind
    }

    /// Get the activation kind for this skill.
    pub fn activation_kind(&self) -> SkillActivationKind {
        self.data().activation_kind
    }

    /// Whether the skill requires target selection (e.g., Warp, Iaigiri).
    pub fn requires_targeting(&self) -> bool {
        self.target_kind() != SkillTargetKind::None
    }

    /// Whether the skill is passive (auto-triggers, no button click needed).
    pub fn is_passive(&self) -> bool {
        self.activation_kind() == SkillActivationKind::Passive
    }

    /// Whether the skill has a duration phase (active_remaining).
    pub fn has_duration(&self) -> bool {
        self.data().duration.is_some()
    }

    /// Get the UI hotkey for this skill.
    pub fn hotkey(&self) -> Option<char> {
        self.data().hotkey
    }

    /// Get the Chinese label for UI display.
    pub fn label_cn(&self) -> &'static str {
        self.data().label_cn
    }

    /// Get the UI icon prefix (emoji or symbol).
    pub fn icon(&self) -> &'static str {
        self.data().icon
    }
}

// ============================================================================
// Skill Data Definitions
// ============================================================================

pub static WARP_DATA: SkillData = SkillData {
    skill_type: SkillType::Warp,
    label: "Warp",
    label_cn: "空间跃迁",
    description: "Teleport to target location",
    icon: "",
    hotkey: None,
    target_kind: SkillTargetKind::Position,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(20),
    duration: None,
    charge_time: None,
};

/// Warp charge time
pub const WARP_CHARGE: Ticks = Ticks::from_whole_secs(0);
/// Warp cooldown
pub const WARP_COOLDOWN: Ticks = Ticks::from_whole_secs(20);
/// Maximum warp range as fraction of visual range
pub const WARP_MAX_RANGE_SCALE: f32 = 1.0;

pub static ENGINE_BOOST_DATA: SkillData = SkillData {
    skill_type: SkillType::EngineBoost,
    label: "Engine Boost",
    label_cn: "引擎增压",
    description: "Temporary maximum speed boost",
    icon: "",
    hotkey: Some('K'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(10),
    duration: Some(Ticks::from_whole_secs(25)),
    charge_time: None,
};

/// Engine boost max speed (106kn = 54.56 m/s)
pub const ENGINE_BOOST_MAX_SPEED_MPS: f32 = 54.56;
/// Max speed phase duration
pub const ENGINE_BOOST_MAX_DURATION: Ticks = Ticks::from_whole_secs(20);
/// Deceleration target speed (90kn = 46.30 m/s)
pub const ENGINE_BOOST_DECEL_SPEED_MPS: f32 = 46.30;
/// Deceleration phase duration
pub const ENGINE_BOOST_DECEL_DURATION: Ticks = Ticks::from_whole_secs(5);
/// Engine boost cooldown
pub const ENGINE_BOOST_COOLDOWN: Ticks = Ticks::from_whole_secs(10);

pub static IAIGIRI_DATA: SkillData = SkillData {
    skill_type: SkillType::Iaigiri,
    label: "Iaigiri",
    label_cn: "居合斩",
    description: "Dash attack leaving mines in your wake",
    icon: "",
    hotkey: Some('J'),
    target_kind: SkillTargetKind::Position,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(20),
    duration: None,
    charge_time: Some(Ticks::from_whole_secs(1)),
};

/// Iaigiri charge time
pub const IAIGIRI_CHARGE: Ticks = Ticks::from_whole_secs(1);
/// Iaigiri cooldown
pub const IAIGIRI_COOLDOWN: Ticks = Ticks::from_whole_secs(20);
/// Maximum range as fraction of visual range
pub const IAIGIRI_MAX_RANGE_SCALE: f32 = 0.8;
/// Number of mines deployed
pub const IAIGIRI_MINE_COUNT: u8 = 20;

pub static SONAR_PULSE_DATA: SkillData = SkillData {
    skill_type: SkillType::SonarPulse,
    label: "Sonar Pulse",
    label_cn: "主动声纳",
    description: "Reveal nearby submarines",
    icon: "",
    hotkey: Some('J'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(30),
    duration: Some(Ticks::from_whole_secs(10)),
    charge_time: None,
};

/// Sonar pulse detection radius
pub const SONAR_PULSE_RADIUS: f32 = 1500.0;
/// Sonar mark duration
pub const SONAR_PULSE_DURATION: Ticks = Ticks::from_whole_secs(10);
/// Sonar pulse cooldown
pub const SONAR_PULSE_COOLDOWN: Ticks = Ticks::from_whole_secs(30);

pub static DEPTH_CHARGE_BARRAGE_DATA: SkillData = SkillData {
    skill_type: SkillType::DepthChargeBarrage,
    label: "Depth Charge Barrage",
    label_cn: "深弹齐射",
    description: "Launch multiple depth charges in a spread pattern",
    icon: "",
    hotkey: Some('K'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(25),
    duration: None,
    charge_time: None,
};

/// Number of depth charges
pub const DCB_COUNT: u32 = 12;
/// Spread angle in degrees
pub const DCB_SPREAD_ANGLE: f32 = 120.0;
/// Launch range in meters
pub const DCB_RANGE: f32 = 200.0;
/// Depth charge barrage cooldown
pub const DCB_COOLDOWN: Ticks = Ticks::from_whole_secs(25);

pub static AIR_SUPERIORITY_DATA: SkillData = SkillData {
    skill_type: SkillType::AirSuperiority,
    label: "Air Superiority",
    label_cn: "制空权",
    description: "Deploy combat drones",
    icon: "",
    hotkey: Some('J'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(45),
    duration: Some(Ticks::from_whole_secs(20)),
    charge_time: None,
};

/// Number of drones deployed
pub const DRONE_COUNT: u32 = 10;
/// Drone lifetime
pub const DRONE_DURATION: Ticks = Ticks::from_whole_secs(20);
/// Drone movement speed (m/s)
pub const DRONE_SPEED: f32 = 50.0;
/// Drone damage per hit
pub const DRONE_DAMAGE: f32 = 0.2;
/// Air superiority cooldown
pub const AIR_SUPERIORITY_COOLDOWN: Ticks = Ticks::from_whole_secs(45);

pub static EMERGENCY_REPAIR_DATA: SkillData = SkillData {
    skill_type: SkillType::EmergencyRepair,
    label: "Emergency Repair",
    label_cn: "紧急维修",
    description: "Repair your ship over time",
    icon: "",
    hotkey: Some('K'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(60),
    duration: Some(Ticks::from_whole_secs(15)),
    charge_time: None,
};

/// Repair duration
pub const REPAIR_DURATION: Ticks = Ticks::from_whole_secs(15);
/// Repair amount as fraction of max HP
pub const REPAIR_AMOUNT: f32 = 0.20;
/// Speed penalty during repair
pub const REPAIR_SPEED_PENALTY: f32 = 0.25;
/// Emergency repair cooldown
pub const EMERGENCY_REPAIR_COOLDOWN: Ticks = Ticks::from_whole_secs(60);

pub static SMOKE_SCREEN_DATA: SkillData = SkillData {
    skill_type: SkillType::SmokeScreen,
    label: "Smoke Screen",
    label_cn: "烟幕",
    description: "Deploy smoke for concealment and weapon disruption",
    icon: "",
    hotkey: Some('L'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(60),
    duration: Some(Ticks::from_whole_secs(30)),
    charge_time: None,
};

/// Smoke screen radius in meters
pub const SMOKE_SCREEN_RADIUS: f32 = 200.0;
/// Smoke screen duration
pub const SMOKE_SCREEN_DURATION: Ticks = Ticks::from_whole_secs(30);
/// Smoke screen cooldown (includes duration + post-effect cooldown)
pub const SMOKE_SCREEN_COOLDOWN: Ticks = Ticks::from_whole_secs(60);

pub static ZERO_PULSE_DATA: SkillData = SkillData {
    skill_type: SkillType::ZeroPulse,
    label: "Zero Pulse",
    label_cn: "绝对零度",
    description: "Freeze nearby enemies",
    icon: "",
    hotkey: Some('Q'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(30),
    duration: Some(Ticks::from_whole_secs(10)),
    charge_time: None,
};

/// Zero pulse radius in meters
pub const ZERO_PULSE_RADIUS: f32 = 1000.0;
/// Zero pulse effect duration
pub const ZERO_PULSE_DURATION: Ticks = Ticks::from_whole_secs(10);
/// Zero pulse cooldown
pub const ZERO_PULSE_COOLDOWN: Ticks = Ticks::from_whole_secs(30);

pub static BURST_LOADING_DATA: SkillData = SkillData {
    skill_type: SkillType::BurstLoading,
    label: "Burst Loading",
    label_cn: "爆发装填",
    description: "Reduces weapon reload time to 0.05s for 30 seconds",
    icon: "",
    hotkey: Some('B'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(45),
    duration: Some(Ticks::from_whole_secs(30)),
    charge_time: None,
};

/// Burst loading cooldown
pub const BURST_LOADING_COOLDOWN: Ticks = Ticks::from_whole_secs(45);
/// Burst loading effect duration
pub const BURST_LOADING_DURATION: Ticks = Ticks::from_whole_secs(30);
/// Burst loading modified reload time in seconds
pub const BURST_LOADING_RELOAD_SECS: f32 = 0.05;

pub static NUCLEAR_STRIKE_DATA: SkillData = SkillData {
    skill_type: SkillType::NuclearStrike,
    label: "Nuclear Strike",
    label_cn: "核打击",
    description: "Destroy all enemies within 1000m radius",
    icon: "☢",
    hotkey: Some('N'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(120),
    duration: None,
    charge_time: Some(Ticks::from_whole_secs(5)),
};

/// Nuclear strike charge time
pub const NUCLEAR_STRIKE_CHARGE: Ticks = Ticks::from_whole_secs(5);
/// Nuclear strike cooldown
pub const NUCLEAR_STRIKE_COOLDOWN: Ticks = Ticks::from_whole_secs(2);
/// Nuclear strike effect radius in meters
pub const NUCLEAR_STRIKE_RADIUS: f32 = 1000.0;

// ============ Energy Shield skill constants ============
pub static ENERGY_SHIELD_DATA: SkillData = SkillData {
    skill_type: SkillType::EnergyShield,
    label: "Energy Shield",
    label_cn: "能量护盾",
    description: "Absorb 90% damage for 8 seconds",
    icon: "🛡",
    hotkey: Some('K'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(45),
    duration: Some(Ticks::from_whole_secs(8)),
    charge_time: None,
};

/// Energy shield duration
pub const ENERGY_SHIELD_DURATION: Ticks = Ticks::from_whole_secs(8);
/// Energy shield cooldown
pub const ENERGY_SHIELD_COOLDOWN: Ticks = Ticks::from_whole_secs(45);
/// Energy shield damage absorption rate (0.9 = 90% absorbed)
pub const ENERGY_SHIELD_ABSORPTION: f32 = 0.9;

// ============ Dredger Sacrifice skill constants ============
pub static DREDGER_SACRIFICE_DATA: SkillData = SkillData {
    skill_type: SkillType::DredgerSacrifice,
    label: "Dredger Sacrifice",
    label_cn: "挖泥船牺牲",
    description: "Sacrifice your dredger to create an oil platform",
    icon: "⚓",
    hotkey: Some('G'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(60),
    duration: None,
    charge_time: None,
};

/// Dredger sacrifice cooldown (player dies immediately so this is mostly cosmetic)
pub const DREDGER_SACRIFICE_COOLDOWN: Ticks = Ticks::from_whole_secs(60);

// ============ Stealth skill constants ============
pub static STEALTH_DATA: SkillData = SkillData {
    skill_type: SkillType::Stealth,
    label: "Stealth",
    label_cn: "隐身",
    description: "Become invisible to enemy radar for 30 seconds",
    icon: "👻",
    hotkey: Some('H'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(60),
    duration: Some(Ticks::from_whole_secs(30)),
    charge_time: None,
};

/// Stealth active duration
pub const STEALTH_DURATION: Ticks = Ticks::from_whole_secs(30);
/// Stealth cooldown
pub const STEALTH_COOLDOWN: Ticks = Ticks::from_whole_secs(60);

// ============ Unjust Game skill constants ============
pub static UNJUST_GAME_DATA: SkillData = SkillData {
    skill_type: SkillType::UnjustGame,
    label: "Unjust Game",
    label_cn: "不义游戏",
    description: "Swap position and motion state with target entity",
    icon: "🔄",
    hotkey: Some('U'),
    target_kind: SkillTargetKind::Entity,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_repr(1), // ~0.1s
    duration: None,
    charge_time: None,
};

/// Unjust game cooldown (~0.1s)
pub const UNJUST_GAME_COOLDOWN: Ticks = Ticks::from_repr(1);

// ============ Last Stand skill constants ============
pub static LAST_STAND_DATA: SkillData = SkillData {
    skill_type: SkillType::LastStand,
    label: "Last Stand",
    label_cn: "孤注一掷",
    description: "Auto-triggers on lethal damage: 10s berserk (2x damage, 2x fire rate), then death",
    icon: "💀",
    hotkey: None,
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Passive,
    cooldown: Ticks::from_whole_secs(180),
    duration: Some(Ticks::from_whole_secs(10)),
    charge_time: None,
};

/// Last Stand berserk duration
pub const LAST_STAND_DURATION: Ticks = Ticks::from_whole_secs(10);
/// Last Stand cooldown (once per life effectively)
pub const LAST_STAND_COOLDOWN: Ticks = Ticks::from_whole_secs(180);
/// Last Stand damage multiplier
pub const LAST_STAND_DAMAGE_MULT: f32 = 2.0;

// ============ Ironclad skill constants ============
pub static IRONCLAD_DATA: SkillData = SkillData {
    skill_type: SkillType::Ironclad,
    label: "Ironclad",
    label_cn: "铁壁",
    description: "Reflect 30% of incoming damage to attacker for 15 seconds",
    icon: "🛡",
    hotkey: Some('Z'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(60),
    duration: Some(Ticks::from_whole_secs(15)),
    charge_time: None,
};

/// Ironclad active duration
pub const IRONCLAD_DURATION: Ticks = Ticks::from_whole_secs(15);
/// Ironclad cooldown
pub const IRONCLAD_COOLDOWN: Ticks = Ticks::from_whole_secs(60);
/// Fraction of damage reflected
pub const IRONCLAD_REFLECT: f32 = 0.3;

// ============ Yamato Cannon skill constants ============
pub static YAMATO_CANNON_DATA: SkillData = SkillData {
    skill_type: SkillType::YamatoCannon,
    label: "Yamato Cannon",
    label_cn: "波动炮",
    description: "Charge 3s, then fire a devastating beam in front direction",
    icon: "⚡",
    hotkey: Some('M'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(90),
    duration: None,
    charge_time: Some(Ticks::from_whole_secs(3)),
};

/// Yamato Cannon charge time
pub const YAMATO_CANNON_CHARGE: Ticks = Ticks::from_whole_secs(3);
/// Yamato Cannon cooldown
pub const YAMATO_CANNON_COOLDOWN: Ticks = Ticks::from_whole_secs(90);
/// Yamato Cannon beam range in meters
pub const YAMATO_CANNON_RANGE: f32 = 2000.0;
/// Yamato Cannon beam width in meters
pub const YAMATO_CANNON_WIDTH: f32 = 50.0;
/// Yamato Cannon damage
pub const YAMATO_CANNON_DAMAGE: f32 = 100.0;

// ============ Orbital Bombardment skill constants ============
pub static ORBITAL_BOMBARDMENT_DATA: SkillData = SkillData {
    skill_type: SkillType::OrbitalBombardment,
    label: "Orbital Bombardment",
    label_cn: "轨道轰炸",
    description: "Rain fire on a 500m area for 5 seconds",
    icon: "🔥",
    hotkey: Some('O'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(120),
    duration: Some(Ticks::from_whole_secs(5)),
    charge_time: None,
};

/// Orbital bombardment radius
pub const ORBITAL_BOMBARDMENT_RADIUS: f32 = 500.0;
/// Orbital bombardment duration
pub const ORBITAL_BOMBARDMENT_DURATION: Ticks = Ticks::from_whole_secs(5);
/// Orbital bombardment cooldown
pub const ORBITAL_BOMBARDMENT_COOLDOWN: Ticks = Ticks::from_whole_secs(120);
/// Damage per tick during bombardment
pub const ORBITAL_BOMBARDMENT_DPS: f32 = 20.0;

// ============ Rift Storm skill constants ============
pub static RIFT_STORM_DATA: SkillData = SkillData {
    skill_type: SkillType::RiftStorm,
    label: "Rift Storm",
    label_cn: "裂隙风暴",
    description: "Spawn random explosions in 800m radius, destroying enemies",
    icon: "💥",
    hotkey: Some('R'),
    target_kind: SkillTargetKind::None,
    activation_kind: SkillActivationKind::Active,
    cooldown: Ticks::from_whole_secs(60),
    duration: None,
    charge_time: None,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_metadata_is_self_consistent() {
        for skill in SkillType::iter() {
            let data = skill.data();
            assert_eq!(data.skill_type, skill);
            assert_eq!(skill.hotkey(), data.hotkey);
            assert_eq!(skill.label_cn(), data.label_cn);
            assert_eq!(skill.icon(), data.icon);
            assert_eq!(skill.is_passive(), data.activation_kind == SkillActivationKind::Passive);
            assert_eq!(skill.requires_targeting(), data.target_kind != SkillTargetKind::None);
        }
    }

    #[test]
    fn targeting_skill_kinds_match_current_design() {
        assert_eq!(SkillType::Warp.target_kind(), SkillTargetKind::Position);
        assert_eq!(SkillType::Iaigiri.target_kind(), SkillTargetKind::Position);
        assert_eq!(SkillType::UnjustGame.target_kind(), SkillTargetKind::Entity);
        assert_eq!(SkillType::LastStand.activation_kind(), SkillActivationKind::Passive);
    }
}

/// Rift storm effect radius
pub const RIFT_STORM_RADIUS: f32 = 800.0;
/// Rift storm cooldown
pub const RIFT_STORM_COOLDOWN: Ticks = Ticks::from_whole_secs(60);
