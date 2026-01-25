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
}

/// Skill metadata and parameters.
#[derive(Clone, Debug)]
pub struct SkillData {
    pub skill_type: SkillType,
    /// Display name
    pub label: &'static str,
    /// Short description
    pub description: &'static str,
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
        ])
    }
}

// ============================================================================
// Skill Data Definitions
// ============================================================================

pub static WARP_DATA: SkillData = SkillData {
    skill_type: SkillType::Warp,
    label: "Warp",
    description: "Teleport to target location",
    cooldown: Ticks::from_whole_secs(20),
    duration: None,
    charge_time: Some(Ticks::from_whole_secs(3)),
};

/// Warp charge time
pub const WARP_CHARGE: Ticks = Ticks::from_whole_secs(3);
/// Warp cooldown
pub const WARP_COOLDOWN: Ticks = Ticks::from_whole_secs(20);
/// Maximum warp range as fraction of visual range
pub const WARP_MAX_RANGE_SCALE: f32 = 1.0;

pub static ENGINE_BOOST_DATA: SkillData = SkillData {
    skill_type: SkillType::EngineBoost,
    label: "Engine Boost",
    description: "Temporary maximum speed boost",
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
    description: "Dash attack leaving mines in your wake",
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
    description: "Reveal nearby submarines",
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
    description: "Launch multiple depth charges in a spread pattern",
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
    description: "Deploy combat drones",
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
    description: "Repair your ship over time",
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
    description: "Deploy smoke for concealment and weapon disruption",
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
    description: "Freeze nearby enemies",
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
    description: "Reduces weapon reload time to 0.05s for 30 seconds",
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
    description: "Destroy all enemies within 1000m radius",
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
    description: "Absorb 90% damage for 8 seconds",
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
