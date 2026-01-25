// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use common::altitude::Altitude;
use common::angle::Angle;
use common::entity::*;
use common::ticks::Ticks;
use common::util::make_mut_slice;
use common::skill::WARP_COOLDOWN;
use common_util::alloc::{arc_default_n, box_default_n};
use glam::Vec2;
use std::iter::FromIterator;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct WarpState {
    pub target: Vec2,
    pub charge: Ticks,
    pub cooldown: Ticks,
}

/// Additional fields for certain entities (for now, boats). Stored separately for memory efficiency.
#[derive(Debug)]
pub struct EntityExtension {
    // true means altitude target is Altitude::MIN false means Altitude::ZERO.
    // Used by Self::altitude_target().
    // Can't submerge right away to prevent dodging missiles.
    submerge: bool,
    submerge_delay: Ticks,

    /// Whether the player *wants* active sensors. To tell if the player *has* active sensors, use
    /// Used by Self::is_active().
    /// Active stays on for a an extra duration to avoid rapid switching, which could induce flickering on other player's screens.
    active: bool,
    deactivate_delay: Ticks,

    /// Whether to sound horn
    pub horn: bool,
    horn_delay: Ticks,

    /// Ticks of protection ticks remaining, zeroed if showing signs of aggression.
    spawn_protection_remaining: Ticks,

    // 1 reload per armament, 0 = reloaded.
    // Not an arc because converted to a bitset with max len of 32.
    pub reloads: Box<[Ticks]>,

    // 1 angle per turret relative to boat.
    // Arc to save allocations
    pub turrets: Arc<[Angle]>,

    /// Space warp info for special ships.
    warp_state: Option<WarpState>,
    warp_cooldown: Ticks,
    zero_pulse_cooldown: Ticks,
    iaigiri_cooldown: Ticks,
    engine_boost_remaining: Ticks,
    engine_boost_decel_remaining: Ticks,
    engine_boost_cooldown: Ticks,
    sonar_pulse_cooldown: Ticks,
    depth_charge_barrage_cooldown: Ticks,
    air_superiority_cooldown: Ticks,
    emergency_repair_cooldown: Ticks,
    emergency_repair_remaining: Ticks,
    smoke_screen_cooldown: Ticks,
    smoke_screen_remaining: Ticks,
    burst_loading_cooldown: Ticks,
    burst_loading_remaining: Ticks,
    nuclear_strike_charge: Ticks,
    nuclear_strike_cooldown: Ticks,
    energy_shield_cooldown: Ticks,
    energy_shield_remaining: Ticks,
}

impl EntityExtension {
    /// How long spawn protection lasts (it linearly fades over this time).
    const SPAWN_PROTECTION_INITIAL: Ticks = Ticks::from_whole_secs(20);

    /// How long deactivating sensors is delayed.
    const DEACTIVATE_DELAY: Ticks = Ticks::from_repr(5);
    /// How long submerging is delayed.
    const SUBMERGE_DELAY: Ticks = Ticks::from_repr(8);
    /// How long horns are delayed.
    const HORN_DELAY: Ticks = Ticks::from_repr(8);

    /// Allocates reloads and turrets, sized to a particular entity type.
    /// It can also give spawn protection.
    pub fn change_entity_type(&mut self, entity_type: EntityType) {
        // TODO clear active/submerge based on if boat supports them but probably doesn't matter.

        let data = entity_type.data();
        self.spawn_protection_remaining = if entity_type.data().level == 1 {
            Self::SPAWN_PROTECTION_INITIAL
        } else {
            Ticks::ZERO
        };
        self.reloads = box_default_n(data.armaments.len());
        self.turrets = Arc::from_iter(data.turrets.iter().map(|t| t.angle));
        self.warp_state = None;
        self.warp_cooldown = Ticks::ZERO;
        self.zero_pulse_cooldown = Ticks::ZERO;
        self.iaigiri_cooldown = Ticks::ZERO;
        self.engine_boost_remaining = Ticks::ZERO;
        self.engine_boost_decel_remaining = Ticks::ZERO;
        self.engine_boost_cooldown = Ticks::ZERO;
        self.sonar_pulse_cooldown = Ticks::ZERO;
        self.depth_charge_barrage_cooldown = Ticks::ZERO;
        self.air_superiority_cooldown = Ticks::ZERO;
        self.emergency_repair_cooldown = Ticks::ZERO;
        self.emergency_repair_remaining = Ticks::ZERO;
        self.smoke_screen_cooldown = Ticks::ZERO;
        self.smoke_screen_remaining = Ticks::ZERO;
    }

    /// Returns the target altitude of the boat from submerge.
    pub fn altitude_target(&self) -> Altitude {
        if self.submerge && self.submerge_delay == Ticks::ZERO {
            Altitude::MIN
        } else {
            Altitude::ZERO
        }
    }

    /// Sets submerge, possibly also setting submerge_delay to an appropriate value.
    pub fn set_submerge(&mut self, submerge: bool) {
        if submerge && !self.submerge {
            self.submerge_delay = Self::SUBMERGE_DELAY;
        }
        self.submerge = submerge;
    }

    /// Sounds horn, sets delay
    pub fn sound_horn(&mut self, horn: bool) {
        if horn && !self.horn && self.horn_delay == Ticks::ZERO {
            self.horn_delay = Self::HORN_DELAY;
        }
        self.horn = horn;
    }

    pub fn is_horn(&self) -> bool {
        self.horn || self.horn_delay > Ticks::ZERO
    }

    /// Returns whether active sensors, or within deactivate sensor delay.
    pub fn is_active(&self) -> bool {
        self.active || self.deactivate_delay > Ticks::ZERO
    }

    /// Sets active, possibly also setting deactivate_delay to an appropriate value.
    pub fn set_active(&mut self, active: bool) {
        if !active && self.active {
            self.deactivate_delay = Self::DEACTIVATE_DELAY;
        }
        self.active = active;
    }

    /// Returns a multiplier for damage taken, taking into account spawn protection.
    pub fn spawn_protection(&self) -> f32 {
        (Self::SPAWN_PROTECTION_INITIAL - self.spawn_protection_remaining).to_secs()
            / Self::SPAWN_PROTECTION_INITIAL.to_secs()
    }

    /// Clears any remaining spawn protection (useful if showing signs of aggression, and thus
    /// no longer deserving of spawn protection).
    pub fn clear_spawn_protection(&mut self) {
        self.spawn_protection_remaining = Ticks::ZERO;
    }

    /// Subtracts from the player's tickers:
    /// submerge
    /// deactivate_delay
    /// horn_delay
    /// spawn_protection_remaining
    pub fn update_tickers(&mut self, delta: Ticks) {
        self.submerge_delay = self.submerge_delay.saturating_sub(delta);
        self.deactivate_delay = self.deactivate_delay.saturating_sub(delta);
        self.horn_delay = self.horn_delay.saturating_sub(delta);
        self.spawn_protection_remaining = self.spawn_protection_remaining.saturating_sub(delta);
        self.zero_pulse_cooldown = self.zero_pulse_cooldown.saturating_sub(delta);
    }

    /// reloads_mut returns a mutable reference to the reloads component of the extension.
    pub fn reloads_mut(&mut self) -> &mut [Ticks] {
        &mut self.reloads
    }

    /// reloads_mut returns a mutable reference to the turret angles component of the extension.
    pub fn turrets_mut(&mut self) -> &mut [Angle] {
        make_mut_slice(&mut self.turrets)
    }

    pub fn start_warp(
        &mut self,
        target: Vec2,
        charge: Ticks,
        cooldown: Ticks,
    ) -> Result<(), &'static str> {
        if self.is_warp_busy() {
            return Err("warp on cooldown");
        }
        self.warp_state = Some(WarpState {
            target,
            charge,
            cooldown,
        });
        Ok(())
    }

    pub fn is_warp_busy(&self) -> bool {
        self.warp_state.is_some() || self.warp_cooldown != Ticks::ZERO
    }

    pub fn is_warping(&self) -> bool {
        self.warp_state.is_some()
    }

    pub fn warp_charge_remaining(&self) -> Ticks {
        self.warp_state
            .as_ref()
            .map(|w| w.charge)
            .unwrap_or(Ticks::ZERO)
    }

    pub fn warp_cooldown_remaining(&self) -> Ticks {
        self.warp_cooldown
    }

    pub fn zero_pulse_cooldown_remaining(&self) -> Ticks {
        self.zero_pulse_cooldown
    }

    pub fn start_zero_pulse(&mut self, cooldown: Ticks) -> Result<(), &'static str> {
        if self.zero_pulse_cooldown != Ticks::ZERO {
            return Err("zero pulse on cooldown");
        }
        self.zero_pulse_cooldown = cooldown;
        Ok(())
    }

    /// Advances warp timers. Returns Some(target) when teleport should occur.
    pub fn advance_warp(&mut self, delta: Ticks) -> Option<Vec2> {
        if let Some(mut warp) = self.warp_state.take() {
            warp.charge = warp.charge.saturating_sub(delta);
            if warp.charge == Ticks::ZERO {
                let target = warp.target;
                self.warp_cooldown = warp.cooldown.max(WARP_COOLDOWN);
                return Some(target);
            } else {
                self.warp_state = Some(warp);
            }
        } else {
            self.warp_cooldown = self.warp_cooldown.saturating_sub(delta);
        }
        None
    }
}

impl Default for EntityExtension {
    /// default allocates an empty entity extension, suitable as not having a boat.
    /// Once a boat is spawned/upgraded change_entity_type must be called.
    fn default() -> Self {
        Self {
            submerge: false,
            submerge_delay: Ticks::ZERO,
            active: true,
            deactivate_delay: Ticks::ZERO,
            horn: false,
            horn_delay: Ticks::ZERO,
            spawn_protection_remaining: Self::SPAWN_PROTECTION_INITIAL,
            reloads: box_default_n(0),
            turrets: arc_default_n(0),
            warp_state: None,
            warp_cooldown: Ticks::ZERO,
            zero_pulse_cooldown: Ticks::ZERO,
            iaigiri_cooldown: Ticks::ZERO,
            engine_boost_remaining: Ticks::ZERO,
            engine_boost_decel_remaining: Ticks::ZERO,
            engine_boost_cooldown: Ticks::ZERO,
            sonar_pulse_cooldown: Ticks::ZERO,
            depth_charge_barrage_cooldown: Ticks::ZERO,
            air_superiority_cooldown: Ticks::ZERO,
            emergency_repair_cooldown: Ticks::ZERO,
            emergency_repair_remaining: Ticks::ZERO,
            smoke_screen_cooldown: Ticks::ZERO,
            smoke_screen_remaining: Ticks::ZERO,
            burst_loading_cooldown: Ticks::ZERO,
            burst_loading_remaining: Ticks::ZERO,
            nuclear_strike_charge: Ticks::ZERO,
            nuclear_strike_cooldown: Ticks::ZERO,
            energy_shield_cooldown: Ticks::ZERO,
            energy_shield_remaining: Ticks::ZERO,
        }
    }
}

impl EntityExtension {
    // Iaigiri methods
    pub fn start_iaigiri(&mut self, cooldown: Ticks) -> Result<(), &'static str> {
        if self.iaigiri_cooldown != Ticks::ZERO {
            return Err("iaigiri on cooldown");
        }
        self.iaigiri_cooldown = cooldown;
        Ok(())
    }

    pub fn iaigiri_cooldown_remaining(&self) -> Ticks {
        self.iaigiri_cooldown
    }

    // Engine boost methods
    pub fn start_engine_boost(&mut self, duration: Ticks, decel_duration: Ticks, cooldown: Ticks) -> Result<(), &'static str> {
        if self.engine_boost_cooldown != Ticks::ZERO || self.engine_boost_remaining != Ticks::ZERO {
            return Err("engine boost on cooldown");
        }
        self.engine_boost_remaining = duration;
        self.engine_boost_decel_remaining = decel_duration;
        self.engine_boost_cooldown = cooldown;
        Ok(())
    }

    pub fn is_engine_boosting(&self) -> bool {
        self.engine_boost_remaining != Ticks::ZERO || self.engine_boost_decel_remaining != Ticks::ZERO
    }

    pub fn engine_boost_speed_multiplier(&self) -> f32 {
        if self.engine_boost_remaining != Ticks::ZERO {
            // Full boost phase: 106 knots
            return 106.0 / 36.0; // ~2.94x multiplier for 36kn base speed
        } else if self.engine_boost_decel_remaining != Ticks::ZERO {
            // Decel phase: interpolate from 106 to 90 knots
            let t = self.engine_boost_decel_remaining.to_secs() / 5.0;
            return (90.0 + 16.0 * t) / 36.0;
        }
        1.0
    }

    pub fn advance_engine_boost(&mut self, delta: Ticks) {
        if self.engine_boost_remaining != Ticks::ZERO {
            self.engine_boost_remaining = self.engine_boost_remaining.saturating_sub(delta);
        } else if self.engine_boost_decel_remaining != Ticks::ZERO {
            self.engine_boost_decel_remaining = self.engine_boost_decel_remaining.saturating_sub(delta);
        } else {
            self.engine_boost_cooldown = self.engine_boost_cooldown.saturating_sub(delta);
        }
    }

    pub fn advance_iaigiri(&mut self, delta: Ticks) {
        self.iaigiri_cooldown = self.iaigiri_cooldown.saturating_sub(delta);
    }

    // SonarPulse methods
    pub fn start_sonar_pulse(&mut self, cooldown: Ticks) -> Result<(), &'static str> {
        if self.sonar_pulse_cooldown != Ticks::ZERO {
            return Err("sonar pulse on cooldown");
        }
        self.sonar_pulse_cooldown = cooldown;
        Ok(())
    }

    pub fn sonar_pulse_cooldown_remaining(&self) -> Ticks {
        self.sonar_pulse_cooldown
    }

    pub fn advance_sonar_pulse(&mut self, delta: Ticks) {
        self.sonar_pulse_cooldown = self.sonar_pulse_cooldown.saturating_sub(delta);
    }

    // DepthChargeBarrage methods
    pub fn start_depth_charge_barrage(&mut self, cooldown: Ticks) -> Result<(), &'static str> {
        if self.depth_charge_barrage_cooldown != Ticks::ZERO {
            return Err("depth charge barrage on cooldown");
        }
        self.depth_charge_barrage_cooldown = cooldown;
        Ok(())
    }

    pub fn depth_charge_barrage_cooldown_remaining(&self) -> Ticks {
        self.depth_charge_barrage_cooldown
    }

    pub fn advance_depth_charge_barrage(&mut self, delta: Ticks) {
        self.depth_charge_barrage_cooldown = self.depth_charge_barrage_cooldown.saturating_sub(delta);
    }

    // AirSuperiority methods
    pub fn start_air_superiority(&mut self, cooldown: Ticks) -> Result<(), &'static str> {
        if self.air_superiority_cooldown != Ticks::ZERO {
            return Err("air superiority on cooldown");
        }
        self.air_superiority_cooldown = cooldown;
        Ok(())
    }

    pub fn air_superiority_cooldown_remaining(&self) -> Ticks {
        self.air_superiority_cooldown
    }

    pub fn advance_air_superiority(&mut self, delta: Ticks) {
        self.air_superiority_cooldown = self.air_superiority_cooldown.saturating_sub(delta);
    }

    // EmergencyRepair methods
    pub fn start_emergency_repair(&mut self, duration: Ticks, cooldown: Ticks) -> Result<(), &'static str> {
        if self.emergency_repair_cooldown != Ticks::ZERO {
            return Err("emergency repair on cooldown");
        }
        self.emergency_repair_remaining = duration;
        self.emergency_repair_cooldown = cooldown;
        Ok(())
    }

    pub fn emergency_repair_cooldown_remaining(&self) -> Ticks {
        self.emergency_repair_cooldown
    }

    pub fn emergency_repair_remaining(&self) -> Ticks {
        self.emergency_repair_remaining
    }

    pub fn is_repairing(&self) -> bool {
        self.emergency_repair_remaining != Ticks::ZERO
    }

    pub fn advance_emergency_repair(&mut self, delta: Ticks) {
        self.emergency_repair_remaining = self.emergency_repair_remaining.saturating_sub(delta);
        self.emergency_repair_cooldown = self.emergency_repair_cooldown.saturating_sub(delta);
    }

    // Smoke screen methods
    pub fn start_smoke_screen(&mut self, duration: Ticks, cooldown: Ticks) -> Result<(), &'static str> {
        if self.smoke_screen_cooldown != Ticks::ZERO {
            return Err("smoke screen on cooldown");
        }
        self.smoke_screen_remaining = duration;
        self.smoke_screen_cooldown = cooldown;
        Ok(())
    }

    pub fn smoke_screen_cooldown_remaining(&self) -> Ticks {
        self.smoke_screen_cooldown
    }

    pub fn smoke_screen_remaining(&self) -> Ticks {
        self.smoke_screen_remaining
    }

    pub fn is_smoke_active(&self) -> bool {
        self.smoke_screen_remaining != Ticks::ZERO
    }

    pub fn advance_smoke_screen(&mut self, delta: Ticks) {
        self.smoke_screen_remaining = self.smoke_screen_remaining.saturating_sub(delta);
        self.smoke_screen_cooldown = self.smoke_screen_cooldown.saturating_sub(delta);
    }

    // Burst loading methods
    pub fn start_burst_loading(&mut self, duration: Ticks, cooldown: Ticks) -> Result<(), &'static str> {
        if self.burst_loading_cooldown != Ticks::ZERO {
            return Err("burst loading on cooldown");
        }
        // Start burst loading effect
        self.burst_loading_remaining = duration;
        self.burst_loading_cooldown = cooldown;
        Ok(())
    }

    pub fn burst_loading_cooldown_remaining(&self) -> Ticks {
        self.burst_loading_cooldown
    }

    pub fn burst_loading_remaining(&self) -> Ticks {
        self.burst_loading_remaining
    }

    pub fn is_burst_loading_active(&self) -> bool {
        self.burst_loading_remaining != Ticks::ZERO
    }

    pub fn advance_burst_loading(&mut self, delta: Ticks) {
        self.burst_loading_remaining = self.burst_loading_remaining.saturating_sub(delta);
        self.burst_loading_cooldown = self.burst_loading_cooldown.saturating_sub(delta);
    }

    // Nuclear Strike methods
    pub fn start_nuclear_strike(&mut self, charge: Ticks, cooldown: Ticks) -> Result<(), &'static str> {
        if self.nuclear_strike_cooldown != Ticks::ZERO {
            return Err("nuclear strike is on cooldown");
        }
        if self.nuclear_strike_charge != Ticks::ZERO {
            return Err("nuclear strike already charging");
        }
        self.nuclear_strike_charge = charge;
        self.nuclear_strike_cooldown = cooldown;
        Ok(())
    }

    pub fn nuclear_strike_charge_remaining(&self) -> Ticks {
        self.nuclear_strike_charge
    }

    pub fn nuclear_strike_cooldown_remaining(&self) -> Ticks {
        self.nuclear_strike_cooldown
    }

    /// Advances nuclear strike timers. Returns true when strike should execute.
    pub fn advance_nuclear_strike(&mut self, delta: Ticks) -> bool {
        if self.nuclear_strike_charge != Ticks::ZERO {
            self.nuclear_strike_charge = self.nuclear_strike_charge.saturating_sub(delta);
            if self.nuclear_strike_charge == Ticks::ZERO {
                // Charge complete, strike should execute
                return true;
            }
        }
        self.nuclear_strike_cooldown = self.nuclear_strike_cooldown.saturating_sub(delta);
        false
    }

    // Energy Shield methods
    pub fn start_energy_shield(&mut self, duration: Ticks, cooldown: Ticks) -> Result<(), &'static str> {
        if self.energy_shield_cooldown != Ticks::ZERO {
            return Err("energy shield is on cooldown");
        }
        if self.energy_shield_remaining != Ticks::ZERO {
            return Err("energy shield already active");
        }
        self.energy_shield_remaining = duration;
        self.energy_shield_cooldown = cooldown;
        Ok(())
    }

    pub fn is_energy_shield_active(&self) -> bool {
        self.energy_shield_remaining != Ticks::ZERO
    }

    pub fn energy_shield_remaining(&self) -> Ticks {
        self.energy_shield_remaining
    }

    pub fn energy_shield_cooldown_remaining(&self) -> Ticks {
        self.energy_shield_cooldown
    }

    /// Advances energy shield timers.
    pub fn advance_energy_shield(&mut self, delta: Ticks) {
        self.energy_shield_remaining = self.energy_shield_remaining.saturating_sub(delta);
        self.energy_shield_cooldown = self.energy_shield_cooldown.saturating_sub(delta);
    }
}
