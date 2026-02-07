// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::entity::Entity;
use crate::player::Status;
use crate::protocol::*;
use crate::server::Server;
use crate::world::World;
use common::altitude::Altitude;
use common::angle::Angle;
use common::entity::*;
use common::death_reason::DeathReason;
use common::protocol::*;
use common::terrain::TerrainMutation;
use common::ticks::Ticks;
use common::velocity::Velocity;
use common::skill::{SkillType, WARP_CHARGE, WARP_COOLDOWN, WARP_MAX_RANGE_SCALE, ZERO_PULSE_COOLDOWN, ZERO_PULSE_DURATION, ZERO_PULSE_RADIUS, NUCLEAR_STRIKE_COOLDOWN, NUCLEAR_STRIKE_RADIUS, ENERGY_SHIELD_DURATION, ENERGY_SHIELD_COOLDOWN};
use common::util::{level_to_score, score_to_level};
use common::world::{clamp_y_to_strict_area_border, outside_strict_area, ARCTIC};
use common_util::range::map_ranges;
use game_server::player::PlayerTuple;
use glam::Vec2;
use maybe_parallel_iterator::IntoMaybeParallelIterator;
use rand::{thread_rng, Rng};
use std::ops::Range;
use std::sync::Arc;
use std::time::Duration;

impl CommandTrait for Spawn {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();
        let moderator = player.client().map(|c| c.moderator).unwrap_or(false);
        if player.data.flags.left_game {
            debug_assert!(
                false,
                "should never happen, since messages should not be accepted"
            );
            return Err("cannot spawn after left game");
        }

        if player.data.status.is_alive() {
            return Err("cannot spawn while already alive");
        }

        if !self
            .entity_type
            .can_spawn_as(player.score, player.is_bot(), moderator)
        {
            return Err("cannot spawn as given entity type");
        }

        // These initial positions may be overwritten later.
        let mut spawn_position = Vec2::ZERO;
        let mut spawn_radius = 0.8 * world.radius;

        let mut rng = thread_rng();

        if !(player.is_bot() && rng.gen()) {
            // Default to spawning near the center of the world, with more points making you spawn further north.
            let raw_spawn_y = map_ranges(
                score_to_level(player.score) as f32,
                1.5..(EntityData::MAX_BOAT_LEVEL * 10 / 25) as f32,
                // But lets not stop people from spawning in the arctic for evil and balance reasons
                // now that it has become more accessible
                -0.85 * world.radius..0.85 * world.radius,
                true,
            );
            debug_assert!((-world.radius..=world.radius).contains(&raw_spawn_y));

            // Don't spawn in wrong area.
            let spawn_y = clamp_y_to_strict_area_border(self.entity_type, raw_spawn_y);

            if spawn_y.abs() > world.radius {
                return Err("unable to spawn this type of boat");
            }

            // Solve circle equation.
            let world_half_width_at_spawn_y = (world.radius.powi(2) - spawn_y.powi(2)).sqrt();
            debug_assert!(world_half_width_at_spawn_y <= world.radius);

            // Randomize horizontal a bit. This value will end up in the range
            // [-world_half_width_at_spawn_y / 2, world_half_width_at_spawn_y / 2].
            let spawn_x = (rng.gen::<f32>() - 0.5) * world_half_width_at_spawn_y;

            spawn_position = Vec2::new(spawn_x, spawn_y);
            spawn_radius = world.radius * (1.0 / 3.0);
        }

        debug_assert!(spawn_position.length() <= world.radius);

        /*
        if !player.player_id.is_bot() {
            debug!(
                "player spawning with {} points, with vertical bias {}, near {} r~{}",
                player.score, vertical_bias, spawn_position, spawn_radius
            );
        }
         */

        let exclusion_zone = match &player.data.status {
            // Player is excluded from spawning too close to where another player sunk them, for
            // fairness reasons.
            Status::Dead {
                reason,
                position,
                time,
                ..
            } => {
                // Don't spawn too far away from where you died.
                spawn_position = *position;
                spawn_radius = (0.4 * world.radius).clamp(1200.0, 3000.0).min(world.radius);

                // Don't spawn right where you died either.
                let exclusion_seconds =
                    if player.score > level_to_score(EntityData::MAX_BOAT_LEVEL / 2) {
                        20
                    } else {
                        10
                    };

                if reason.is_due_to_player()
                    && time.elapsed() < Duration::from_secs(exclusion_seconds)
                {
                    Some(*position)
                } else {
                    None
                }
            }
            _ => None,
        };

        if player.team_id().is_some() || player.invitation_accepted().is_some() {
            // TODO: Inefficient to scan all entities; only need to scan all players. Unfortunately,
            // that data is not available here, currently.
            if let Some((_, team_boat)) = world
                .entities
                .par_iter()
                .into_maybe_parallel_iter()
                .find_any(|(_, entity)| {
                    let data = entity.data();
                    if data.kind != EntityKind::Boat {
                        return false;
                    }

                    if let Some(exclusion_zone) = exclusion_zone {
                        if entity.transform.position.distance_squared(exclusion_zone)
                            < 1100f32.powi(2)
                        {
                            return false;
                        }
                    }

                    let is_team_member = player.team_id().is_some()
                        && entity.borrow_player().team_id() == player.team_id();

                    let was_invited_by = player.invitation_accepted().is_some()
                        && entity.borrow_player().player_id
                            == player.invitation_accepted().as_ref().unwrap().player_id;

                    is_team_member || was_invited_by
                })
            {
                spawn_position = team_boat.transform.position;
                spawn_radius = team_boat.data().radius + 25.0;
            }
        }

        drop(player);

        let mut boat = Entity::new(self.entity_type, Some(Arc::clone(player_tuple)));
        boat.transform.position = spawn_position;
        //#[cfg(debug_assertions)]
        //let begin = std::time::Instant::now();
        if world.spawn_here_or_nearby(boat, spawn_radius, exclusion_zone) {
            /*
            #[cfg(debug_assertions)]
            println!(
                "took {:?} to spawn a {:?}",
                begin.elapsed(),
                self.entity_type
            );
             */
            Ok(())
        } else {
            Err("failed to find enough space to spawn")
        }
    }
}

impl CommandTrait for Control {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let mut player = player_tuple.borrow_player_mut();

        // Pre-borrow.
        let world_radius = world.radius;

        return if let Status::Alive {
            entity_index,
            aim_target,
            ..
        } = &mut player.data.status
        {
            let entity = &mut world.entities[*entity_index];

            // Movement
            if let Some(guidance) = self.guidance {
                entity.guidance = guidance;
            }
            *aim_target = if let Some(mut aim_target) = self.aim_target {
                sanitize_floats(aim_target.as_mut(), -world_radius * 2.0..world_radius * 2.0)?;
                Some(
                    (aim_target - entity.transform.position)
                        .clamp_length_max(entity.data().sensors.max_range())
                        + entity.transform.position,
                )
            } else {
                None
            };
            let extension = entity.extension_mut();
            extension.set_submerge(self.submerge);
            extension.set_active(self.active);
            extension.sound_horn(self.horn);

            drop(player);

            if let Some(fire) = &self.fire {
                fire.apply(world, player_tuple)?;
            }

            if let Some(pay) = &self.pay {
                pay.apply(world, player_tuple)?;
            }

            if let Some(hint) = &self.hint {
                hint.apply(world, player_tuple)?;
            }

            Ok(())
        } else {
            Err("cannot control while not alive")
        };
    }
}

impl CommandTrait for Warp {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive {
                entity_index, ..
            } => entity_index,
            _ => return Err("cannot warp while not alive"),
        };

        let entity = &mut world.entities[entity_index];
        let data = entity.data();
        // Check if entity has Warp skill
        if !data.has_skill(SkillType::Warp) {
            return Err("warp not supported");
        }

        if entity.extension().is_warp_busy() {
            return Err("warp busy");
        }

        let mut target = self.target;
        let valid_range = -world.radius * 2.0..world.radius * 2.0;
        target.x = sanitize_float(target.x, valid_range.clone())?;
        target.y = sanitize_float(target.y, valid_range)?;

        // 限制在当前可视范围附近，防止穿越全图。
        let max_offset = data.camera_range() * WARP_MAX_RANGE_SCALE;
        let current = entity.transform.position;
        let delta = target - current;
        let clamped_target = current + delta.clamp_length_max(max_offset);

        // 保证落点不出边界。
        let border_limit = world.radius - data.length.max(100.0);
        let length_sq = clamped_target.length_squared();
        let target = if length_sq > border_limit.powi(2) {
            clamped_target.normalize_or_zero() * border_limit
        } else {
            clamped_target
        };

        entity
            .extension_mut()
            .start_warp(target, WARP_CHARGE, WARP_COOLDOWN)?;
        Ok(())
    }
}

impl CommandTrait for ZeroPulse {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();

        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot pulse while not alive"),
        };

        let entity = &mut world.entities[entity_index];
        // Check if entity has ZeroPulse skill
        if !entity.data().has_skill(SkillType::ZeroPulse) {
            return Err("zero pulse not supported");
        }

        entity
            .extension_mut()
            .start_zero_pulse(ZERO_PULSE_COOLDOWN)?;

        let center = entity.transform.position;
        let radius = ZERO_PULSE_RADIUS;

        let targets: Vec<_> = world
            .entities
            .iter_radius(center, radius)
            .filter_map(|(target_index, target)| {
                let data = target.data();
                if !(data.kind == EntityKind::Boat || data.kind == EntityKind::Aircraft) {
                    return None;
                }
                if target.is_friendly_to_player(Some(player_tuple)) {
                    return None;
                }
                Some(target_index)
            })
            .collect();

        for target_index in targets {
            world.entities[target_index].freeze_for(ZERO_PULSE_DURATION);
        }

        world
            .events
            .push(WorldEvent::ZeroPulse { center, radius });

        Ok(())
    }
}

impl CommandTrait for Fire {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();

        return if let Status::Alive {
            entity_index,
            aim_target,
            ..
        } = player.data.status
        {
            // Prevents limited armaments from being invalidated since all limited armaments are destroyed on upgrade.
            if player.data.flags.upgraded {
                return Err("cannot fire right after upgrading");
            }

            let entity = &mut world.entities[entity_index];

            if entity.extension().is_warping() {
                return Err("cannot fire while warping");
            }
            if entity.is_frozen() {
                return Err("cannot fire while frozen");
            }

            let data = entity.data();

            let index = self.armament_index as usize;
            if index >= data.armaments.len() {
                return Err("armament index out of bounds");
            }

            if entity.extension().reloads[index] != Ticks::ZERO {
                return Err("armament not yet reloaded");
            }

            let armament = &data.armaments[index];
            let armament_entity_data = armament.entity_type.data();

            // Can't fire if boat is a submerged former submarine.
            if entity.altitude.is_submerged()
                && (data.sub_kind != EntitySubKind::Submarine
                    || matches!(armament_entity_data.kind, EntityKind::Aircraft)
                    || matches!(
                        armament_entity_data.sub_kind,
                        EntitySubKind::Shell | EntitySubKind::Sam | EntitySubKind::TankShell
                    ))
            {
                return Err("cannot fire while surfacing as a boat");
            }

            if entity.altitude > Altitude(50)
                && !(matches!(
                    data.sub_kind,
                    EntitySubKind::Aeroplane | EntitySubKind::Starship | EntitySubKind::Helicopter
                ))
            {
                return Err("cannot fire while flying high (lol)");
            }

            if let Some(turret_index) = armament.turret {
                let turret_angle = entity.extension().turrets[turret_index];
                let turret = &data.turrets[turret_index];

                // The aim may be outside the range but the turret must not be fired if the turret's
                // current angle is outside the range.
                if !turret.within_azimuth(turret_angle) {
                    return Err("invalid turret azimuth");
                }
            }

            let armament_transform =
                entity.transform + data.armament_transform(&entity.extension().turrets, index);

            if armament_entity_data.sub_kind == EntitySubKind::Depositor {
                if let Some(mut target) = aim_target {
                    // Can't deposit in arctic.
                    target.y = target.y.min((ARCTIC - 50.0) - 2.0 * common::terrain::SCALE);

                    // Clamp target is in valid range from depositor or error if too far.
                    const DEPOSITOR_RANGE: f32 = 60.0;
                    let depositor = armament_transform.position;
                    let pos =
                        clamp_to_range(depositor, target, DEPOSITOR_RANGE, DEPOSITOR_RANGE * 2.0)?;

                    world.terrain.modify(TerrainMutation::simple(pos, 120.0));
                } else {
                    return Err("cannot deposit without aim target");
                }
            } else if armament_entity_data.sub_kind == EntitySubKind::Shovel {
                if let Some(mut target) = aim_target {
                    // Can't deposit in arctic.
                    target.y = target.y.min((ARCTIC - 50.0) - 2.0 * common::terrain::SCALE);

                    // Clamp target is in valid range from depositor or error if too far.
                    const DEPOSITOR_RANGE: f32 = 60.0;
                    let shovel = armament_transform.position;
                    let pos =
                        clamp_to_range(shovel, target, DEPOSITOR_RANGE, DEPOSITOR_RANGE * 2.0)?;

                    world.terrain.modify(TerrainMutation::simple(pos, -120.0));
                } else {
                    return Err("cannot shovel without aim target");
                }
            } else if armament_entity_data.sub_kind == EntitySubKind::Mine {
                let player_arc = Arc::clone(player_tuple);

                drop(player);
                let mut armament_entity = Entity::new(armament.entity_type, Some(player_arc));

                armament_entity.transform = armament_transform;
                armament_entity.altitude = entity.altitude;
                armament_entity.transform.velocity = armament_entity.transform.velocity * 0.667;
                if !world.spawn_here_or_nearby(armament_entity, 0.0, None) {
                    return Err("failed to fire from current location");
                }
            } else {
                // Fire weapon.
                let player_arc = Arc::clone(player_tuple);

                drop(player);
                let mut armament_entity = Entity::new(armament.entity_type, Some(player_arc));

                armament_entity.transform = armament_transform;
                armament_entity.altitude = entity.altitude;

                let aim_angle = aim_target
                    .map(|aim| Angle::from(aim - armament_entity.transform.position))
                    .unwrap_or(entity.transform.direction);

                armament_entity.guidance.velocity_target = armament_entity_data.speed;
                armament_entity.guidance.direction_target = aim_angle;

                if armament.vertical {
                    // Vertically-launched armaments can be launched in any horizontal direction.
                    armament_entity.transform.direction = armament_entity.guidance.direction_target;
                }

                // Some weapons experience random deviation on launch
                let deviation = match armament_entity_data.sub_kind {
                    EntitySubKind::Rocket | EntitySubKind::RocketTorpedo => 0.05,
                    EntitySubKind::Shell | EntitySubKind::TankShell => 0.01,
                    EntitySubKind::Laser => 0.0,
                    _ => 0.03,
                };
                armament_entity.transform.direction += thread_rng().gen::<Angle>() * deviation;

                if !world.spawn_here_or_nearby(armament_entity, 0.0, None) {
                    return Err("failed to fire from current location");
                }
            }

            let entity = &mut world.entities[entity_index];
            entity.consume_armament(index);
            entity.extension_mut().clear_spawn_protection();

            Ok(())
        } else {
            Err("cannot fire while not alive")
        };
    }
}

impl CommandTrait for Pay {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let mut player = player_tuple.borrow_player_mut();

        return if let Status::Alive {
            entity_index,
            aim_target: Some(target),
            ..
        } = player.data.status
        {
            let entity = &world.entities[entity_index];

            // Clamp pay to range or error if too far.
            let max_range = entity.data().radii().end;
            let cutoff_range = max_range * 4.0;
            let target =
                clamp_to_range(entity.transform.position, target, max_range, cutoff_range)?;

            let pay = 10; // Value of coin.
            let withdraw = pay * 2; // Payment has 50% efficiency.

            if player.score < level_to_score(entity.data().level) + withdraw {
                return Err("insufficient funds");
            }

            let mut payment = Entity::new(
                EntityType::Coin,
                Some(Arc::clone(entity.player.as_ref().unwrap())),
            );

            payment.transform.position = target;
            payment.altitude = entity.altitude;

            // If payment successfully spawns, withdraw funds.
            if world.spawn_here_or_nearby(payment, 1.0, None) {
                player.score -= withdraw;
            }

            Ok(())
        } else {
            Err("cannot pay while not alive and aiming")
        };
    }
}

impl CommandTrait for Hint {
    fn apply(
        &self,
        _: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        player_tuple.borrow_player_mut().data.hint = Hint {
            aspect: sanitize_float(self.aspect, 0.5..2.0)?,
        };
        Ok(())
    }
}

impl CommandTrait for Upgrade {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let mut player = player_tuple.borrow_player_mut();
        let status = &mut player.data.status;

        if let Status::Alive { entity_index, .. } = status {
            let entity = &mut world.entities[*entity_index];
            let moderator = player.client().map(|c| c.moderator).unwrap_or(false);
            if !entity.entity_type.can_upgrade_to(
                self.entity_type,
                player.score,
                player.is_bot(),
                moderator,
            ) {
                return Err("cannot upgrade to provided entity type");
            }

            if outside_strict_area(self.entity_type, entity.transform.position) {
                return Err("cannot upgrade outside the correct area");
            }

            player.data.flags.upgraded = true;

            let below_full_potential = self.entity_type.data().level < score_to_level(player.score);

            drop(player);

            entity.change_entity_type(self.entity_type, &mut world.arena, below_full_potential);

            Ok(())
        } else {
            Err("cannot upgrade while not alive")
        }
    }
}

/// Returns an error if the float isn't finite. Otherwise, clamps it to the provided range.
fn sanitize_float(float: f32, valid: Range<f32>) -> Result<f32, &'static str> {
    if float.is_finite() {
        Ok(float.clamp(valid.start, valid.end))
    } else {
        Err("float not finite")
    }
}

/// Applies sanitize_float to each element.
fn sanitize_floats<'a, F: IntoIterator<Item = &'a mut f32>>(
    floats: F,
    valid: Range<f32>,
) -> Result<(), &'static str> {
    for float in floats {
        *float = sanitize_float(*float, valid.clone())?;
    }
    Ok(())
}

/// Clamps a center -> target vector to `range` and errors if it's length is greater than
/// `cutoff_range`.
fn clamp_to_range(
    center: Vec2,
    target: Vec2,
    range: f32,
    cutoff_range: f32,
) -> Result<Vec2, &'static str> {
    let delta = target - center;
    if delta.length_squared() > cutoff_range.powi(2) {
        Err("outside maximum range")
    } else {
        Ok(center + delta.clamp_length_max(range))
    }
}


impl CommandTrait for Iaigiri {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{IAIGIRI_COOLDOWN, IAIGIRI_MAX_RANGE_SCALE, IAIGIRI_MINE_COUNT};
        
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot iaigiri while not alive"),
        };

        let entity = &world.entities[entity_index];
        
        // Check if entity has Iaigiri skill
        if !entity.data().has_skill(SkillType::Iaigiri) {
            return Err("iaigiri not supported");
        }

        if entity.extension().iaigiri_cooldown_remaining() != Ticks::ZERO {
            return Err("iaigiri on cooldown");
        }

        let data = entity.data();
        
        // Calculate target position
        let mut target = self.target;
        let valid_range = -world.radius * 2.0..world.radius * 2.0;
        target.x = sanitize_float(target.x, valid_range.clone()).unwrap_or(entity.transform.position.x);
        target.y = sanitize_float(target.y, valid_range).unwrap_or(entity.transform.position.y);

        let max_offset = data.camera_range() * IAIGIRI_MAX_RANGE_SCALE;
        let start = entity.transform.position;
        let delta = target - start;
        let end = start + delta.clamp_length_max(max_offset);

        // Clamp to world border
        let border_limit = world.radius - data.length.max(100.0);
        let end = if end.length_squared() > border_limit.powi(2) {
            end.normalize_or_zero() * border_limit
        } else {
            end
        };

        let altitude = entity.altitude;
        
        // Start cooldown
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_iaigiri(IAIGIRI_COOLDOWN)?;
        
        // Spawn mines along path
        let mine_count = IAIGIRI_MINE_COUNT as usize;
        for i in 0..mine_count {
            let t = (i as f32 + 0.5) / mine_count as f32;
            let mine_pos = start.lerp(end, t);
            
            let mut mine = Entity::new(EntityType::IaigiriMine, Some(Arc::clone(player_tuple)));
            mine.transform.position = mine_pos;
            mine.altitude = altitude;
            world.spawn_here_or_nearby(mine, 5.0, None);
        }

        // Teleport to end position
        let entity = &mut world.entities[entity_index];
        entity.transform.position = end;

        Ok(())
    }
}

impl CommandTrait for EngineBoost {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{ENGINE_BOOST_MAX_DURATION, ENGINE_BOOST_DECEL_DURATION, ENGINE_BOOST_COOLDOWN};
        
        
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot boost while not alive"),
        };

        let entity = &mut world.entities[entity_index];
        
        // Check if entity has EngineBoost skill
        if !entity.data().has_skill(SkillType::EngineBoost) {
            return Err("engine boost not supported");
        }

        
        entity.extension_mut().start_engine_boost(
            ENGINE_BOOST_MAX_DURATION,
            ENGINE_BOOST_DECEL_DURATION,
            ENGINE_BOOST_COOLDOWN,
        )?;

        Ok(())
    }
}

impl CommandTrait for SonarPulse {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{SONAR_PULSE_COOLDOWN, SONAR_PULSE_RADIUS};
        
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot sonar pulse while not alive"),
        };

        let entity = &world.entities[entity_index];
        
        // Check if entity has SonarPulse skill
        if !entity.data().has_skill(SkillType::SonarPulse) {
            return Err("sonar pulse not supported");
        }

        if entity.extension().sonar_pulse_cooldown_remaining() != Ticks::ZERO {
            return Err("sonar pulse on cooldown");
        }

        let center = entity.transform.position;

        // Start cooldown
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_sonar_pulse(SONAR_PULSE_COOLDOWN)?;

        // Find and reveal submerged submarines in range
        // Note: In a full implementation, we would need a "revealed" state that
        // gets synced to clients. For now, we'll just set active sensors temporarily.
        let targets: Vec<_> = world
            .entities
            .iter_radius(center, SONAR_PULSE_RADIUS)
            .filter_map(|(target_index, target)| {
                let data = target.data();
                if data.sub_kind != EntitySubKind::Submarine {
                    return None;
                }
                if !target.altitude.is_submerged() {
                    return None;
                }
                if target.is_friendly_to_player(Some(player_tuple)) {
                    return None;
                }
                Some(target_index)
            })
            .collect();

        // Mark targets as detected (set their active sensor flag temporarily)
        for target_index in targets {
            let target = &mut world.entities[target_index];
            target.extension_mut().set_active(true);
        }

        // Broadcast sonar pulse event for visual effect
        world.events.push(WorldEvent::ZeroPulse { center, radius: SONAR_PULSE_RADIUS });

        Ok(())
    }
}

impl CommandTrait for DepthChargeBarrage {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{DCB_COOLDOWN, DCB_COUNT, DCB_RANGE, DCB_SPREAD_ANGLE};
        
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot depth charge barrage while not alive"),
        };

        let entity = &world.entities[entity_index];
        
        // Check if entity has DepthChargeBarrage skill
        if !entity.data().has_skill(SkillType::DepthChargeBarrage) {
            return Err("depth charge barrage not supported");
        }

        if entity.extension().depth_charge_barrage_cooldown_remaining() != Ticks::ZERO {
            return Err("depth charge barrage on cooldown");
        }

        let center = entity.transform.position;
        let direction = entity.transform.direction;
        let altitude = entity.altitude;

        // Start cooldown
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_depth_charge_barrage(DCB_COOLDOWN)?;

        // Spawn depth charges in a fan pattern
        let half_angle = DCB_SPREAD_ANGLE / 2.0;
        for i in 0..DCB_COUNT {
            // Calculate angle offset within the fan
            let t = if DCB_COUNT > 1 {
                (i as f32) / ((DCB_COUNT - 1) as f32)
            } else {
                0.5
            };
            let angle_offset_deg = -half_angle + t * DCB_SPREAD_ANGLE;
            let angle_offset = Angle::from_degrees(angle_offset_deg);
            let launch_direction = direction + angle_offset;
            
            // Calculate spawn position
            let offset = launch_direction.to_vec() * DCB_RANGE;
            let spawn_pos = center + offset;

            let mut depth_charge = Entity::new(EntityType::Mark9, Some(Arc::clone(player_tuple)));
            depth_charge.transform.position = spawn_pos;
            depth_charge.transform.direction = launch_direction;
            depth_charge.altitude = altitude;
            
            world.spawn_here_or_nearby(depth_charge, 10.0, None);
        }

        Ok(())
    }
}

impl CommandTrait for AirSuperiority {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::AIR_SUPERIORITY_COOLDOWN;
        use common::entity::EntityKind;
        
        // Get entity_index from player status
        let entity_index = {
            let player = player_tuple.borrow_player();
            match player.data.status {
                Status::Alive { entity_index, .. } => entity_index,
                _ => return Err("not alive"),
            }
        }; // player borrow dropped here

        // Check entity type, get aircraft type, and check cooldown
        let aircraft_type = {
            let entity = &world.entities[entity_index];
            
            // Check if entity has AirSuperiority skill
            if !entity.data().has_skill(SkillType::AirSuperiority) {
                return Err("air superiority not supported");
            }

            if entity.extension().air_superiority_cooldown_remaining() != Ticks::ZERO {
                return Err("air superiority on cooldown");
            }
            
            // Find the first aircraft type from armaments
            entity.data().armaments.iter()
                .find_map(|arm| {
                    let arm_type = arm.entity_type;
                    if arm_type.data().kind == EntityKind::Aircraft {
                        Some(arm_type)
                    } else {
                        None
                    }
                })
                .ok_or("no aircraft armaments")?
        };

        // Get position and direction
        let (center, direction) = {
            let entity = &world.entities[entity_index];
            (entity.transform.position, entity.transform.direction)
        };

        // Start cooldown
        {
            let entity = &mut world.entities[entity_index];
            entity.extension_mut().start_air_superiority(AIR_SUPERIORITY_COOLDOWN)?;
        }

        // Spawn Aircraft with player ownership
        // Aircraft don't trigger create_index (only Boats do), so this is safe
        for i in 0..10 {
            let angle_offset = Angle::from_degrees((i as f32 - 5.0) * 20.0);
            let spawn_direction = direction + angle_offset;
            let offset = spawn_direction.to_vec() * 100.0;
            let spawn_pos = center + offset;

            // Create Aircraft WITH player ownership using detected aircraft type
            let mut drone = Entity::new(aircraft_type, Some(Arc::clone(player_tuple)));
            drone.transform.position = spawn_pos;
            drone.transform.direction = spawn_direction;
            drone.transform.velocity = Velocity::from_mps(50.0);
            drone.guidance.direction_target = spawn_direction;
            drone.guidance.velocity_target = Velocity::from_mps(50.0);
            
            // world.add is safe for Aircraft with player - doesn't call create_index
            world.add(drone);
        }

        Ok(())
    }
}



impl CommandTrait for EmergencyRepair {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{EMERGENCY_REPAIR_COOLDOWN, REPAIR_DURATION};
        
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("not alive"),
        };

        let entity = &world.entities[entity_index];
        
        // Check if entity has EmergencyRepair skill
        if !entity.data().has_skill(SkillType::EmergencyRepair) {
            return Err("emergency repair not supported");
        }

        if entity.extension().emergency_repair_cooldown_remaining() != Ticks::ZERO {
            return Err("emergency repair on cooldown");
        }

        // Start repair
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_emergency_repair(REPAIR_DURATION, EMERGENCY_REPAIR_COOLDOWN)?;

        Ok(())
    }
}

impl CommandTrait for SmokeScreen {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{SMOKE_SCREEN_COOLDOWN, SMOKE_SCREEN_DURATION};
        
        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("not alive"),
        };

        let entity = &world.entities[entity_index];
        
        // Check if entity has SmokeScreen skill
        if !entity.data().has_skill(SkillType::SmokeScreen) {
            return Err("smoke screen not supported");
        }

        if entity.extension().smoke_screen_cooldown_remaining() != Ticks::ZERO {
            return Err("smoke screen on cooldown");
        }

        // Start smoke screen
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_smoke_screen(SMOKE_SCREEN_DURATION, SMOKE_SCREEN_COOLDOWN)?;

        Ok(())
    }
}

impl CommandTrait for BurstLoading {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        use common::skill::{BURST_LOADING_COOLDOWN, BURST_LOADING_DURATION};

        let player = player_tuple.borrow_player();
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("not alive"),
        };
        let entity = &world.entities[entity_index];

        // Check if entity has BurstLoading skill
        if !entity.data().has_skill(SkillType::BurstLoading) {
            return Err("entity does not have burst loading skill");
        }

        if entity.extension().burst_loading_cooldown_remaining() != Ticks::ZERO {
            return Err("burst loading on cooldown");
        }

        // Start burst loading effect (duration-based reload speed buff)
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_burst_loading(BURST_LOADING_DURATION, BURST_LOADING_COOLDOWN)?;

        Ok(())
    }
}

impl CommandTrait for NuclearStrike {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        log::warn!("NuclearStrike command received!");
        let player = player_tuple.borrow_player();

        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot use nuclear strike while not alive"),
        };
        
        // Drop the player borrow BEFORE calling is_friendly_to_player in the iterator
        drop(player);

        let entity = &world.entities[entity_index];
        
        // Check if entity has NuclearStrike skill
        if !entity.data().has_skill(SkillType::NuclearStrike) {
            return Err("entity does not have nuclear strike skill");
        }

        // Check cooldown
        if entity.extension().nuclear_strike_cooldown_remaining() != Ticks::ZERO {
            return Err("nuclear strike is on cooldown");
        }

        let center = entity.transform.position;
        let radius = NUCLEAR_STRIKE_RADIUS;

        // Start cooldown
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_nuclear_strike(Ticks::ZERO, NUCLEAR_STRIKE_COOLDOWN)?;

        // Find all non-friendly combat entities in radius (boats, aircraft, weapons - NOT collectibles)
        let mut targets: Vec<_> = world
            .entities
            .iter_radius(center, radius)
            .filter_map(|(target_index, target)| {
                if target_index == entity_index { return None; }
                let data = target.data();
                // Only affect boats, aircraft, and weapons (skip collectibles, obstacles, etc.)
                if !matches!(data.kind, EntityKind::Boat | EntityKind::Aircraft | EntityKind::Weapon) {
                    return None;
                }
                // Skip friendly entities
                if target.is_friendly_to_player(Some(player_tuple)) { return None; }
                Some(target_index)
            })
            .collect();

        // Sort in reverse order to avoid index invalidation when removing
        targets.sort_by(|a, b| b.cmp(a));

        log::warn!("NuclearStrike: Found {} combat targets in radius {}", targets.len(), radius);

        // Instant kill all targets by removing them from the world
        for target_index in targets {
            let target_data = world.entities[target_index].data();
            log::warn!("  Killing target: {:?}, kind={:?}", target_data.label, target_data.kind);
            // Use world.remove() to properly kill and trigger death effects
            world.remove(target_index, DeathReason::Unknown);
        }

        log::warn!("NuclearStrike: Pushing WorldEvent::NuclearStrike");
        // Push event for visual effects on client
        world
            .events
            .push(WorldEvent::NuclearStrike { center, radius });
        log::warn!("NuclearStrike: events Vec now has {} events", world.events.len());

        Ok(())
    }
}

impl CommandTrait for EnergyShield {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &std::sync::Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str> {
        let player = player_tuple.borrow_player();
        
        let entity_index = match player.data.status {
            Status::Alive { entity_index, .. } => entity_index,
            _ => return Err("cannot use energy shield while not alive"),
        };
        drop(player);

        let entity = &world.entities[entity_index];
        
        // Check if entity has EnergyShield skill
        if !entity.data().has_skill(SkillType::EnergyShield) {
            return Err("ship does not have energy shield skill");
        }

        // Start shield
        let entity = &mut world.entities[entity_index];
        entity.extension_mut().start_energy_shield(ENERGY_SHIELD_DURATION, ENERGY_SHIELD_COOLDOWN)?;

        log::warn!("EnergyShield activated for {:?}", entity.data().label);

        Ok(())
    }
}
