// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::complete_ref::CompleteRef;
use crate::contact_ref::ContactRef;
use crate::server::Server;
use crate::player::Status;
use common::altitude::Altitude;
use common::angle::Angle;
use common::complete::CompleteTrait;
use common::contact::ContactTrait;
use common::entity::*;
use common::guidance::Guidance;
use common::protocol::*;
use common::terrain;
use common::terrain::Terrain;
use common_util::range::gen_radius;
use common::protocol::FactionId;
use core_protocol::id::PlayerId;
use game_server::game_service::{BotAction, GameArenaService};
use game_server::player::{PlayerRepo, PlayerTuple};
use glam::Vec2;
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng};
use std::sync::Arc;

/// Bot implements a ship-controlling AI that is, in many ways, equivalent to a player.
pub struct Bot {
    /// Chance of attacking, randomized to improve variety of bots.
    aggression: f32,
    /// Amount to offset steering by. This creates more interesting behavior.
    steer_bias: Angle,
    /// Amount to offset aiming by. This creates more interesting hit patterns.
    aim_bias: Vec2,
    /// Maximum level bot will try to upgrade to, randomized to improve variety of bots.
    level_ambition: u8,
    /// Whether the bot spawned at least once, and therefore is capable of rage-quitting.
    spawned_at_least_once: bool,
    /// The value of submerge previously sent.
    was_submerging: bool,
    /// Makes sure bot's planes etc despawn
    has_waited_one_tick: bool,
    /// Persistent commitment: once true, bot heads to altar until dead.
    sacrifice_committed: bool,
    /// Tracks whether the bot was alive last tick (for death/respawn detection).
    was_alive_last_tick: bool,
}

/// Altar info passed from get_input to bot update.
#[derive(Clone, Copy, Default)]
pub struct AltarInfo {
    pub position: Option<Vec2>,
    pub sacrifice_count: u8,
    /// Whether this bot is eligible for sacrifice (bottom 10 by level in faction).
    pub is_sacrifice_eligible: bool,
    /// This bot's current level (for scoring formula).
    pub my_level: u8,
    /// Whether this faction has at least one real (non-bot) player.
    pub faction_has_real_player: bool,
}

/// Wrapper for bot input that includes both the game state and altar info.
pub struct BotInput<'a, I: Iterator<Item = ContactRef<'a>>> {
    pub complete: CompleteRef<'a, I>,
    pub altar: AltarInfo,
}

impl Default for Bot {
    fn default() -> Self {
        let mut rng = thread_rng();

        fn random_level(rng: &mut ThreadRng) -> u8 {
            rng.gen_range(5..=EntityData::MAX_BOAT_LEVEL)
        }

        Self {
            // Raise aggression to a power such that lower values are more common.
            aggression: rng.gen::<f32>().powi(2) * Self::MAX_AGGRESSION,
            steer_bias: rng.gen::<Angle>() * 0.1,
            aim_bias: gen_radius(&mut rng, 5.0),
            // Bias towards lower levels.
            level_ambition: random_level(&mut rng).min(random_level(&mut rng)),
            spawned_at_least_once: false,
            was_submerging: false,
            has_waited_one_tick: false,
            sacrifice_committed: false,
            was_alive_last_tick: false,
        }
    }
}

impl Bot {
    /// This arbitrary value controls how chill the bots are. If too high, bots are trigger-happy
    /// maniacs, and the waters get filled with stray torpedoes.
    const MAX_AGGRESSION: f32 = 0.35;

    /// Returns true if there is land or border at the given position.
    fn is_land_or_border(pos: Vec2, terrain: &Terrain, world_radius: f32) -> bool {
        if pos.length_squared() > world_radius.powi(2) {
            return true;
        }

        terrain.sample(pos).unwrap_or(Altitude::MIN) >= terrain::SAND_LEVEL
    }

    /// Pre-checks if a given armament can be fired based on altitude/surfacing conditions.
    /// This mirrors the checks in world_inbound.rs to avoid generating invalid fire commands.
    fn can_fire_armament<C: ContactTrait>(
        boat: &C,
        boat_data: &EntityData,
        armament_data: &EntityData,
    ) -> bool {
        let altitude = boat.altitude();
        
        // Can't fire if boat is a submerged former submarine (or non-submarine that upgraded from sub)
        if altitude.is_submerged() {
            if boat_data.sub_kind != EntitySubKind::Submarine {
                return false;
            }
            // Submarines can't fire certain weapons while submerged
            if matches!(
                armament_data.sub_kind,
                EntitySubKind::Shell | EntitySubKind::Sam | EntitySubKind::TankShell
            ) || matches!(armament_data.kind, EntityKind::Aircraft) {
                return false;
            }
        }
        
        // Can't fire if flying high (except for aircraft/starships)
        if altitude > Altitude(50) && !matches!(
            boat_data.sub_kind,
            EntitySubKind::Aeroplane | EntitySubKind::Starship | EntitySubKind::Helicopter
        ) {
            return false;
        }
        
        true
    }

    /// update processes a complete update and returns some command (or None to quit).
    fn update_with_altar<'a, U: 'a + CompleteTrait<'a>>(
        &mut self,
        mut update: U,
        player_id: PlayerId,
        players: &PlayerRepo<Server>,
        altar_info: AltarInfo,
    ) -> BotAction<Command> {
        let mut rng = thread_rng();

        // Faction-aware friendly detection: look up bot's own faction once.
        let faction_mode = crate::runtime_config::hot_faction_mode();
        let my_faction: Option<FactionId> = if faction_mode {
            players.borrow_player(player_id).and_then(|p| p.data.faction)
        } else {
            None
        };

        // Boss bots always aim for max level.
        let is_boss = players.borrow_player(player_id).map_or(false, |p| p.data.is_boss);
        if is_boss {
            self.level_ambition = EntityData::MAX_BOAT_LEVEL;
        }

        let mut contacts = update.contacts();
        let terrain = update.terrain();

        if let Some(boat) = contacts
            .next()
            .filter(|c| c.is_boat() && c.player_id() == Some(player_id))
        {
            self.spawned_at_least_once = true;

            let boat_type: EntityType = boat.entity_type().unwrap();
            let data: &EntityData = boat_type.data();
            let health_percent = 1.0 - boat.damage().to_secs() / data.max_health().to_secs();

            // Weighted sums of direction vectors for various purposes.
            let mut movement = Vec2::ZERO;

            let attract = |weighted_sum: &mut Vec2, target_delta: Vec2, distance_squared: f32| {
                *weighted_sum += target_delta / (1.0 + distance_squared);
            };

            let repel = |weighted_sum: &mut Vec2, target_delta: Vec2, distance_squared: f32| {
                attract(weighted_sum, -target_delta, distance_squared);
            };

            let spring = |weighted_sum: &mut Vec2, target_delta: Vec2, desired_distance: f32| {
                let distance = target_delta.length();
                let displacement = distance - desired_distance;
                *weighted_sum = target_delta * displacement / (displacement.powi(2) + 1.0);
            };

            // Terrain avoidance — skip entirely for sacrifice-committed bots.
            if !self.sacrifice_committed {
                let (scan_radius_mult, num_samples, repel_denom_mult) = if is_boss {
                    (3.0_f32, 20u32, 0.1_f32)
                } else {
                    (1.0, 10, 0.5)
                };
                for i in 0..num_samples {
                    let angle =
                        Angle::from_radians(i as f32 * (2.0 * std::f32::consts::PI / num_samples as f32));
                    let delta_position = angle.to_vec() * data.length * scan_radius_mult;
                    if boat_type != EntityType::Sherman
                        && boat_type != EntityType::Abrams
                        && Self::is_land_or_border(
                            boat.transform().position + delta_position,
                            terrain,
                            update.world_radius(),
                        )
                    {
                        repel(&mut movement, delta_position, repel_denom_mult * data.length.powi(2));
                    } else if Self::is_land_or_border(
                        boat.transform().position + delta_position,
                        terrain,
                        update.world_radius(),
                    ) {
                        attract(&mut movement, delta_position, repel_denom_mult * data.length.powi(2));
                    }
                }
            }

            let mut closest_enemy: Option<(U::Contact, f32)> = None;

            // Scan sensor contacts to help make decisions.
            for contact in contacts {
                if contact.id() == boat.id() {
                    // Skip processing self.
                    continue;
                }

                if let Some(contact_data) = contact.entity_type().map(EntityType::data) {
                    let delta_position = contact.transform().position - boat.transform().position;
                    let distance_squared = delta_position.length_squared();

                    let friendly = contact.player_id() == Some(player_id)
                        || (my_faction.is_some()
                            && contact.player_id()
                                .and_then(|cid| players.borrow_player(cid))
                                .and_then(|p| p.data.faction)
                                == my_faction);

                    if contact_data.kind == EntityKind::Collectible {
                        attract(&mut movement, delta_position, distance_squared);
                    } else if (!friendly || contact_data.kind == EntityKind::Boat)
                        && !(!friendly
                            && contact_data.kind == EntityKind::Boat
                            && data.sub_kind == EntitySubKind::Ram)
                    {
                        repel(&mut movement, delta_position, distance_squared);
                    }

                    if friendly {
                        if contact_data.kind == EntityKind::Boat {
                            spring(
                                &mut movement,
                                delta_position,
                                data.radius + contact_data.radius,
                            );
                        }
                    } else if match contact_data.kind {
                        // Don't kill smol/peaceful boats unless they get too close.
                        EntityKind::Boat => {
                            (contact_data.level + 1 >= data.level
                                && !matches!(
                                    contact_data.sub_kind,
                                    EntitySubKind::Dredger
                                        | EntitySubKind::Icebreaker
                                        | EntitySubKind::Passenger
                                ))
                                || contact.player_id().map(|id| id.is_bot()).unwrap_or(false)
                                || distance_squared < 1.5 * data.radius.powi(2)
                                || health_percent < 1.0 / 3.0
                        }
                        EntityKind::Aircraft => true,
                        EntityKind::Weapon => matches!(
                            contact_data.sub_kind,
                            EntitySubKind::Missile | EntitySubKind::Torpedo
                        ),
                        EntityKind::Obstacle => {
                            // Don't repel from altar if this bot might sacrifice.
                            let is_altar = contact.entity_type() == Some(EntityType::DropletAltar);
                            if !is_altar {
                                repel(
                                    &mut movement,
                                    delta_position,
                                    (distance_squared - contact_data.radius.powi(2)).max(0.0),
                                );
                            }
                            false
                        }
                        _ => false,
                    } {
                        if let Some(existing) = &closest_enemy {
                            if distance_squared < existing.1 {
                                closest_enemy = Some((contact, distance_squared));
                            }
                        } else {
                            closest_enemy = Some((contact, distance_squared));
                        }
                    }
                }
            }

            let mut best_firing_solution = None;

            if let Some((ref enemy, _)) = closest_enemy {
                let reloads = boat.reloads();
                let enemy_data = enemy.data();
                for (i, armament) in data.armaments.iter().enumerate() {
                    if !reloads[i] {
                        // Not yet reloaded.
                        continue;
                    }

                    let armament_entity_data: &EntityData = armament.entity_type.data();
                    
                    // Pre-check if we can fire this armament (avoids generating invalid commands)
                    if !Self::can_fire_armament(&boat, data, armament_entity_data) {
                        continue;
                    }
                    
                    if !matches!(
                        armament_entity_data.kind,
                        EntityKind::Weapon | EntityKind::Aircraft | EntityKind::Decoy
                    ) {
                        continue;
                    }

                    let relevant = match enemy_data.kind {
                        EntityKind::Aircraft | EntityKind::Weapon => {
                            if enemy.altitude().is_airborne() {
                                matches!(armament_entity_data.sub_kind, EntitySubKind::Sam)
                            } else if enemy_data.sub_kind == EntitySubKind::Torpedo
                                && enemy_data.sensors.sonar.range > 0.0
                            {
                                armament_entity_data.kind == EntityKind::Decoy
                                    && armament_entity_data.sub_kind == EntitySubKind::Sonar
                            } else {
                                false
                            }
                        }
                        EntityKind::Boat => {
                            if enemy.data().level == 1
                                && armament_entity_data.sub_kind == EntitySubKind::Shell
                            {
                                // Don't attempt to sink level 1 boats with shells, as it is very
                                // toxic for new players to die in this way.
                                false
                            } else if enemy.altitude().is_submerged() {
                                matches!(
                                    armament_entity_data.sub_kind,
                                    EntitySubKind::Torpedo
                                        | EntitySubKind::Plane
                                        | EntitySubKind::Heli
                                        | EntitySubKind::DepthCharge
                                        | EntitySubKind::RocketTorpedo
                                )
                            } else {
                                matches!(
                                    armament_entity_data.sub_kind,
                                    EntitySubKind::Torpedo
                                        | EntitySubKind::Plane
                                        | EntitySubKind::Heli
                                        | EntitySubKind::DepthCharge
                                        | EntitySubKind::Rocket
                                        | EntitySubKind::Missile
                                        | EntitySubKind::Shell
                                )
                            }
                        }
                        _ => false,
                    };

                    if !relevant {
                        continue;
                    }

                    if let Some(turret_index) = armament.turret {
                        if !data.turrets[turret_index].within_azimuth(boat.turrets()[turret_index])
                        {
                            // Out of azimuth range; cannot fire.
                            continue;
                        }
                    }

                    let transform = *boat.transform() + data.armament_transform(boat.turrets(), i);
                    let angle = Angle::from(enemy.transform().position - transform.position);

                    let mut angle_diff = (angle - transform.direction).abs();
                    if armament.vertical
                        || matches!(
                            armament_entity_data.kind,
                            EntityKind::Aircraft | EntityKind::Decoy
                        )
                    {
                        angle_diff = Angle::ZERO;
                    }

                    if angle_diff > Angle::from_degrees(60.0) {
                        continue;
                    }

                    let firing_solution = (i as u8, enemy.transform().position, angle_diff);

                    if firing_solution.2
                        < best_firing_solution
                            .map(|s: (u8, Vec2, Angle)| s.2)
                            .unwrap_or(Angle::MAX)
                    {
                        best_firing_solution = Some(firing_solution);
                    }
                }
            }

            // ---- Droplet Altar bot behavior ----
            let mut altar_sacrifice_mode = false;

            // Death reset: if just respawned (transition dead→alive), clear commitment.
            if !self.was_alive_last_tick {
                self.sacrifice_committed = false;
            }
            self.was_alive_last_tick = true;

            if let Some(altar_pos) = altar_info.position {
                let boat_pos = boat.transform().position;
                let to_altar = altar_pos - boat_pos;
                let dist_to_altar = to_altar.length();

                // Reset commitment if no longer eligible (e.g. leveled up past threshold).
                if self.sacrifice_committed && !altar_info.is_sacrifice_eligible {
                    self.sacrifice_committed = false;
                }

                if !is_boss && altar_info.is_sacrifice_eligible {
                    // Eligible bot: use scoring formula for commit probability.
                    if !self.sacrifice_committed {
                        let level = altar_info.my_level.max(1) as f32;
                        let level_factor = 1.0 / level;
                        let distance_factor = 1.0 / (1.0 + dist_to_altar / 1000.0);
                        let urgency = if altar_info.sacrifice_count >= 3 { 3.0 } else { 1.0 };
                        let score = level_factor * distance_factor * urgency;
                        let commit_chance = (score * 0.01_f32).min(0.1);

                        if rng.gen_bool(commit_chance as f64) {
                            self.sacrifice_committed = true;
                        }
                    }
                    if self.sacrifice_committed {
                        // Hard override — beeline to altar (terrain scan already skipped).
                        movement = to_altar;
                        altar_sacrifice_mode = true;
                    }
                } else if (is_boss || data.level >= 8) && health_percent > 0.5 {
                    // High-level bot: patrol near altar.
                    const PATROL_INNER: f32 = 500.0;
                    const PATROL_OUTER: f32 = 800.0;

                    if dist_to_altar > PATROL_OUTER {
                        movement += to_altar.normalize_or_zero() * 2.0;
                    } else if dist_to_altar < PATROL_INNER {
                        movement -= to_altar.normalize_or_zero() * 1.0;
                    } else {
                        let tangent = Vec2::new(-to_altar.y, to_altar.x).normalize_or_zero();
                        movement += tangent * 1.5;
                    }
                }

            } else {
                // No altar known (altar destroyed or not discovered) — reset commitment.
                self.sacrifice_committed = false;
            }
            // ---- End Droplet Altar bot behavior ----

            self.was_submerging = if data.sub_kind == EntitySubKind::Submarine {
                // More positive values mean want to surface, more negative values mean want to dive.
                let surface_bias = health_percent - self.aggression * (2.0 / Self::MAX_AGGRESSION);

                // Hysteresis.
                if self.was_submerging && surface_bias >= 0.1 {
                    false
                } else if !self.was_submerging && surface_bias <= -0.1 {
                    true
                } else {
                    self.was_submerging
                }
            } else {
                false
            };

            let mut ret = Command::Control(Control {
                guidance: Some(Guidance {
                    direction_target: Angle::from(movement) + self.steer_bias,
                    velocity_target: if altar_sacrifice_mode { data.speed } else { data.speed * 0.8 },
                }),
                submerge: self.was_submerging,
                aim_target: best_firing_solution.map(|solution| solution.1 + self.aim_bias),
                active: health_percent >= 0.5,
                fire: best_firing_solution
                    .filter(|_| rng.gen_bool(self.aggression as f64))
                    .map(|sol| Fire {
                        armament_index: sol.0,
                    }),
                pay: None,
                hint: None,
                horn: false,
            });

            if rng.gen_bool(self.aggression as f64) && data.level < self.level_ambition {
                // Upgrade, if possible.
                if let Some(entity_type) = boat_type
                    .upgrade_options(update.score(), true, false)
                    .choose(&mut rng)
                {
                    ret = Command::Upgrade(Upgrade { entity_type });
                }
            }

            // --- Boss skill AI ---
            // Priority-ordered skill activation. Server rejects if on cooldown,
            // so we can attempt every tick without tracking CDs.
            if is_boss {
                use common::skill::SkillType;

                let closest_dist_sq = closest_enemy.as_ref().map(|(_, d)| *d);
                let has_enemy = closest_enemy.is_some();

                // 1. Emergency Repair — critical HP
                if health_percent < 0.4 && data.has_skill(SkillType::EmergencyRepair) {
                    ret = Command::EmergencyRepair(EmergencyRepair);
                }
                // 2. Energy Shield — moderate HP or under fire
                else if health_percent < 0.6 && data.has_skill(SkillType::EnergyShield) {
                    ret = Command::EnergyShield(EnergyShield);
                }
                // 3. Zero Pulse — enemy within 1000m (dist_sq < 1_000_000)
                else if closest_dist_sq.map_or(false, |d| d < 1_000_000.0)
                    && data.has_skill(SkillType::ZeroPulse)
                {
                    ret = Command::ZeroPulse(ZeroPulse);
                }
                // 4. Air Superiority — enemy detected
                else if has_enemy && data.has_skill(SkillType::AirSuperiority) {
                    ret = Command::AirSuperiority(AirSuperiority);
                }
                // 5. Burst Loading — enemy in weapon range
                else if closest_dist_sq.map_or(false, |d| d < data.range * data.range)
                    && data.has_skill(SkillType::BurstLoading)
                {
                    ret = Command::BurstLoading(BurstLoading);
                }
                // 6. Smoke Screen — retreating under fire
                else if health_percent < 0.5 && data.has_skill(SkillType::SmokeScreen) {
                    ret = Command::SmokeScreen(SmokeScreen);
                }
                // 7. Stealth — enemy detected
                else if has_enemy && data.has_skill(SkillType::Stealth) {
                    ret = Command::Stealth(Stealth);
                }
                // 8. Warp — escape death
                else if health_percent < 0.25 && data.has_skill(SkillType::Warp) {
                    let escape_dir = Angle::from_radians(
                        rng.gen_range(0.0..std::f32::consts::TAU),
                    );
                    let target = boat.transform().position
                        + escape_dir.to_vec() * data.sensors.visual.range * 0.8;
                    ret = Command::Warp(Warp { target });
                }
                // NuclearStrike intentionally excluded — too destructive for AI.
            }

            BotAction::Some(ret)
        } else {
            // Bot is dead — mark for death→alive transition detection.
            self.was_alive_last_tick = false;
            if self.spawned_at_least_once && (rng.gen_bool(1.0 / 3.0)) {
                // Rage quit.
                BotAction::Quit
            } else if self.has_waited_one_tick {
                BotAction::Some(Command::Spawn(Spawn {
                    entity_type: EntityType::spawn_options(0, true, false)
                        .choose(&mut rng)
                        .expect("there must be at least one entity type to spawn as"),
                }))
            } else {
                self.has_waited_one_tick = true;
                BotAction::None
            }
        }
    }
}

impl game_server::game_service::Bot<Server> for Bot {
    type Input<'a> = BotInput<'a, impl Iterator<Item = ContactRef<'a>>>;

    fn get_input<'a>(
        server: &'a Server,
        player: &'a Arc<PlayerTuple<Server>>,
        _players: &'a PlayerRepo<Server>,
    ) -> Self::Input<'a> {
        let altar = {
            let p = player.borrow_player();
            if let Some(faction) = p.data.faction {
                let idx = faction.index();
                // Check if this bot's level is at or below the bottom-10 threshold.
                let my_level = if let Status::Alive { entity_index, .. } = p.data.status {
                    server.world.entities[entity_index].data().level
                } else {
                    0
                };
                let threshold = server.altar_sacrifice_level_threshold[idx];
                let has_real = _players.iter_borrow().any(|p| {
                    !p.is_bot() && p.data.faction == Some(faction)
                });
                AltarInfo {
                    position: server.altar_known_position[idx],
                    sacrifice_count: server.altar_sacrifices[idx],
                    is_sacrifice_eligible: my_level <= threshold && my_level > 0,
                    my_level,
                    faction_has_real_player: has_real,
                }
            } else {
                AltarInfo::default()
            }
        };
        BotInput {
            complete: server.world.get_player_complete(player),
            altar,
        }
    }

    fn update(
        &mut self,
        input: Self::Input<'_>,
        player_id: PlayerId,
        players: &PlayerRepo<Server>,
    ) -> BotAction<<Server as GameArenaService>::GameRequest> {
        let altar_info = input.altar;
        self.update_with_altar(input.complete, player_id, players, altar_info)
    }
}
