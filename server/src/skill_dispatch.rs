// SPDX-FileCopyrightText: 2026 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::entity::Entity;
use crate::entities::EntityIndex;
use crate::player::Status;
use crate::server::Server;
use crate::world::World;
use common::altitude::Altitude;
use common::angle::Angle;
use common::death_reason::DeathReason;
use common::entity::*;
use common::guidance::Guidance;
use common::protocol::*;
use common::skill::{SkillActivationKind, SkillTargetKind, SkillType};
use common::ticks::Ticks;
use common::transform::Transform;
use common::velocity::Velocity;
use game_server::player::PlayerTuple;
use glam::Vec2;
use std::ops::Range;
use std::sync::Arc;

pub fn dispatch_use_skill(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
    use_skill: &UseSkill,
) -> Result<(), &'static str> {
    dispatch_skill(world, player_tuple, use_skill.skill, use_skill.target.clone())
}

pub fn dispatch_skill(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
    skill: SkillType,
    target: SkillTarget,
) -> Result<(), &'static str> {
    validate_skill_request(world, player_tuple, skill, &target)?;

    match (skill, target) {
        (SkillType::Warp, SkillTarget::Position(target)) => warp(world, player_tuple, target),
        (SkillType::ZeroPulse, SkillTarget::None) => zero_pulse(world, player_tuple),
        (SkillType::Iaigiri, SkillTarget::Position(target)) => iaigiri(world, player_tuple, target),
        (SkillType::EngineBoost, SkillTarget::None) => engine_boost(world, player_tuple),
        (SkillType::SonarPulse, SkillTarget::None) => sonar_pulse(world, player_tuple),
        (SkillType::DepthChargeBarrage, SkillTarget::None) => {
            depth_charge_barrage(world, player_tuple)
        }
        (SkillType::AirSuperiority, SkillTarget::None) => air_superiority(world, player_tuple),
        (SkillType::EmergencyRepair, SkillTarget::None) => emergency_repair(world, player_tuple),
        (SkillType::SmokeScreen, SkillTarget::None) => smoke_screen(world, player_tuple),
        (SkillType::BurstLoading, SkillTarget::None) => burst_loading(world, player_tuple),
        (SkillType::NuclearStrike, SkillTarget::None) => nuclear_strike(world, player_tuple),
        (SkillType::EnergyShield, SkillTarget::None) => energy_shield(world, player_tuple),
        (SkillType::DredgerSacrifice, SkillTarget::None) => {
            dredger_sacrifice(world, player_tuple)
        }
        (SkillType::Stealth, SkillTarget::None) => stealth(world, player_tuple),
        (SkillType::UnjustGame, SkillTarget::Entity(target_id)) => {
            unjust_game(world, player_tuple, target_id)
        }
        (SkillType::Ironclad, SkillTarget::None) => ironclad(world, player_tuple),
        (SkillType::YamatoCannon, SkillTarget::None) => yamato_cannon(world, player_tuple),
        (SkillType::OrbitalBombardment, SkillTarget::None) => {
            orbital_bombardment(world, player_tuple)
        }
        (SkillType::RiftStorm, SkillTarget::None) => rift_storm(world, player_tuple),
        _ => Err("skill target mismatch"),
    }
}

fn validate_skill_request(
    world: &World,
    player_tuple: &Arc<PlayerTuple<Server>>,
    skill: SkillType,
    target: &SkillTarget,
) -> Result<(), &'static str> {
    if skill.activation_kind() == SkillActivationKind::Passive {
        return Err("passive skill cannot be used directly");
    }

    match (skill.target_kind(), target) {
        (SkillTargetKind::None, SkillTarget::None)
        | (SkillTargetKind::Position, SkillTarget::Position(_))
        | (SkillTargetKind::Entity, SkillTarget::Entity(_)) => {}
        _ => return Err("skill target kind mismatch"),
    }

    let player = player_tuple.borrow_player();
    let entity_index = match player.data.status {
        Status::Alive { entity_index, .. } => entity_index,
        _ => return Err("cannot use skill while not alive"),
    };

    let entity = &world.entities[entity_index];
    if !entity.data().has_skill(skill) {
        return Err("entity does not have requested skill");
    }

    Ok(())
}

fn alive_entity_index(
    player_tuple: &Arc<PlayerTuple<Server>>,
    err: &'static str,
) -> Result<EntityIndex, &'static str> {
    let player = player_tuple.borrow_player();
    match player.data.status {
        Status::Alive { entity_index, .. } => Ok(entity_index),
        _ => Err(err),
    }
}

fn warp(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
    target: Vec2,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "cannot warp while not alive")?;

    let entity = &mut world.entities[entity_index];
    let data = entity.data();

    if entity.extension().is_warp_busy() {
        return Err("warp busy");
    }

    let mut target = target;
    let valid_range = -world.radius * 2.0..world.radius * 2.0;
    target.x = sanitize_float(target.x, valid_range.clone())?;
    target.y = sanitize_float(target.y, valid_range)?;

    let max_offset = data.camera_range() * common::skill::WARP_MAX_RANGE_SCALE;
    let current = entity.transform.position;
    let delta = target - current;
    let clamped_target = current + delta.clamp_length_max(max_offset);

    let border_limit = world.radius - data.length.max(100.0);
    let length_sq = clamped_target.length_squared();
    let target = if length_sq > border_limit.powi(2) {
        clamped_target.normalize_or_zero() * border_limit
    } else {
        clamped_target
    };

    entity.extension_mut().start_warp(
        target,
        common::skill::WARP_CHARGE,
        common::skill::WARP_COOLDOWN,
    )?;
    Ok(())
}

fn zero_pulse(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "cannot pulse while not alive")?;

    let entity = &mut world.entities[entity_index];
    entity
        .extension_mut()
        .start_zero_pulse(common::skill::ZERO_PULSE_COOLDOWN)?;

    let center = entity.transform.position;
    let radius = common::skill::ZERO_PULSE_RADIUS;

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
        world.entities[target_index].freeze_for(common::skill::ZERO_PULSE_DURATION);
    }

    world
        .events
        .push(WorldEvent::ZeroPulse { center, radius });

    Ok(())
}

fn iaigiri(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
    target: Vec2,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "cannot iaigiri while not alive")?;

    let entity = &world.entities[entity_index];
    if entity.extension().iaigiri_cooldown_remaining() != Ticks::ZERO {
        return Err("iaigiri on cooldown");
    }

    let data = entity.data();
    let mut target = target;
    let valid_range = -world.radius * 2.0..world.radius * 2.0;
    target.x = sanitize_float(target.x, valid_range.clone())
        .unwrap_or(entity.transform.position.x);
    target.y = sanitize_float(target.y, valid_range).unwrap_or(entity.transform.position.y);

    let max_offset = data.camera_range() * common::skill::IAIGIRI_MAX_RANGE_SCALE;
    let start = entity.transform.position;
    let delta = target - start;
    let end = start + delta.clamp_length_max(max_offset);

    let border_limit = world.radius - data.length.max(100.0);
    let end = if end.length_squared() > border_limit.powi(2) {
        end.normalize_or_zero() * border_limit
    } else {
        end
    };

    let altitude = entity.altitude;

    world.entities[entity_index]
        .extension_mut()
        .start_iaigiri(common::skill::IAIGIRI_COOLDOWN)?;

    let mine_count = common::skill::IAIGIRI_MINE_COUNT as usize;
    for i in 0..mine_count {
        let t = (i as f32 + 0.5) / mine_count as f32;
        let mine_pos = start.lerp(end, t);

        let mut mine = Entity::new(EntityType::IaigiriMine, Some(Arc::clone(player_tuple)));
        mine.transform.position = mine_pos;
        mine.altitude = altitude;
        world.spawn_here_or_nearby(mine, 5.0, None);
    }

    world.entities[entity_index].transform.position = end;
    Ok(())
}

fn engine_boost(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "cannot boost while not alive")?;

    world.entities[entity_index].extension_mut().start_engine_boost(
        common::skill::ENGINE_BOOST_MAX_DURATION,
        common::skill::ENGINE_BOOST_DECEL_DURATION,
        common::skill::ENGINE_BOOST_COOLDOWN,
    )?;

    Ok(())
}

fn sonar_pulse(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "cannot sonar pulse while not alive")?;
    let entity = &world.entities[entity_index];

    if entity.extension().sonar_pulse_cooldown_remaining() != Ticks::ZERO {
        return Err("sonar pulse on cooldown");
    }

    let center = entity.transform.position;
    world.entities[entity_index]
        .extension_mut()
        .start_sonar_pulse(common::skill::SONAR_PULSE_COOLDOWN)?;

    let targets: Vec<_> = world
        .entities
        .iter_radius(center, common::skill::SONAR_PULSE_RADIUS)
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

    for target_index in targets {
        world.entities[target_index].extension_mut().set_active(true);
    }

    world.events.push(WorldEvent::ZeroPulse {
        center,
        radius: common::skill::SONAR_PULSE_RADIUS,
    });

    Ok(())
}

fn depth_charge_barrage(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(
        player_tuple,
        "cannot depth charge barrage while not alive",
    )?;
    let entity = &world.entities[entity_index];

    if entity.extension().depth_charge_barrage_cooldown_remaining() != Ticks::ZERO {
        return Err("depth charge barrage on cooldown");
    }

    let center = entity.transform.position;
    let direction = entity.transform.direction;
    let altitude = entity.altitude;

    world.entities[entity_index]
        .extension_mut()
        .start_depth_charge_barrage(common::skill::DCB_COOLDOWN)?;

    let half_angle = common::skill::DCB_SPREAD_ANGLE / 2.0;
    for i in 0..common::skill::DCB_COUNT {
        let t = if common::skill::DCB_COUNT > 1 {
            (i as f32) / ((common::skill::DCB_COUNT - 1) as f32)
        } else {
            0.5
        };
        let angle_offset_deg = -half_angle + t * common::skill::DCB_SPREAD_ANGLE;
        let angle_offset = Angle::from_degrees(angle_offset_deg);
        let launch_direction = direction + angle_offset;
        let offset = launch_direction.to_vec() * common::skill::DCB_RANGE;
        let spawn_pos = center + offset;

        let mut depth_charge = Entity::new(EntityType::Mark9, Some(Arc::clone(player_tuple)));
        depth_charge.transform.position = spawn_pos;
        depth_charge.transform.direction = launch_direction;
        depth_charge.altitude = altitude;

        world.spawn_here_or_nearby(depth_charge, 10.0, None);
    }

    Ok(())
}

fn air_superiority(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;

    let aircraft_type = {
        let entity = &world.entities[entity_index];
        if entity.extension().air_superiority_cooldown_remaining() != Ticks::ZERO {
            return Err("air superiority on cooldown");
        }

        entity
            .data()
            .armaments
            .iter()
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

    let (center, direction) = {
        let entity = &world.entities[entity_index];
        (entity.transform.position, entity.transform.direction)
    };

    world.entities[entity_index]
        .extension_mut()
        .start_air_superiority(common::skill::AIR_SUPERIORITY_COOLDOWN)?;

    for i in 0..10 {
        let angle_offset = Angle::from_degrees((i as f32 - 5.0) * 20.0);
        let spawn_direction = direction + angle_offset;
        let offset = spawn_direction.to_vec() * 100.0;
        let spawn_pos = center + offset;

        let mut drone = Entity::new(aircraft_type, Some(Arc::clone(player_tuple)));
        drone.transform.position = spawn_pos;
        drone.transform.direction = spawn_direction;
        drone.transform.velocity = Velocity::from_mps(50.0);
        drone.guidance.direction_target = spawn_direction;
        drone.guidance.velocity_target = Velocity::from_mps(50.0);

        world.add(drone);
    }

    Ok(())
}

fn emergency_repair(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];
    if entity.extension().emergency_repair_cooldown_remaining() != Ticks::ZERO {
        return Err("emergency repair on cooldown");
    }

    world.entities[entity_index]
        .extension_mut()
        .start_emergency_repair(
            common::skill::REPAIR_DURATION,
            common::skill::EMERGENCY_REPAIR_COOLDOWN,
        )?;

    Ok(())
}

fn smoke_screen(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];
    if entity.extension().smoke_screen_cooldown_remaining() != Ticks::ZERO {
        return Err("smoke screen on cooldown");
    }

    world.entities[entity_index]
        .extension_mut()
        .start_smoke_screen(
            common::skill::SMOKE_SCREEN_DURATION,
            common::skill::SMOKE_SCREEN_COOLDOWN,
        )?;
    Ok(())
}

fn burst_loading(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];
    if entity.extension().burst_loading_cooldown_remaining() != Ticks::ZERO {
        return Err("burst loading on cooldown");
    }

    world.entities[entity_index]
        .extension_mut()
        .start_burst_loading(
            common::skill::BURST_LOADING_DURATION,
            common::skill::BURST_LOADING_COOLDOWN,
        )?;
    Ok(())
}

fn nuclear_strike(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    log::warn!("NuclearStrike command received!");
    let entity_index = alive_entity_index(player_tuple, "cannot use nuclear strike while not alive")?;

    let entity = &world.entities[entity_index];
    if entity.extension().nuclear_strike_cooldown_remaining() != Ticks::ZERO {
        return Err("nuclear strike is on cooldown");
    }

    let center = entity.transform.position;
    let radius = common::skill::NUCLEAR_STRIKE_RADIUS;

    world.entities[entity_index]
        .extension_mut()
        .start_nuclear_strike(Ticks::ZERO, common::skill::NUCLEAR_STRIKE_COOLDOWN)?;

    let mut targets: Vec<_> = world
        .entities
        .iter_radius(center, radius)
        .filter_map(|(target_index, target)| {
            if target_index == entity_index {
                return None;
            }
            let data = target.data();
            if !matches!(
                data.kind,
                EntityKind::Boat | EntityKind::Aircraft | EntityKind::Weapon
            ) {
                return None;
            }
            if target.is_friendly_to_player(Some(player_tuple)) {
                return None;
            }
            Some(target_index)
        })
        .collect();

    targets.sort_by(|a, b| b.cmp(a));
    log::warn!(
        "NuclearStrike: Found {} combat targets in radius {}",
        targets.len(),
        radius
    );

    for target_index in targets {
        let target_data = world.entities[target_index].data();
        log::warn!(
            "  Killing target: {:?}, kind={:?}",
            target_data.label,
            target_data.kind
        );
        world.remove(target_index, DeathReason::Unknown);
    }

    world
        .events
        .push(WorldEvent::NuclearStrike { center, radius });
    log::warn!("NuclearStrike: events Vec now has {} events", world.events.len());

    Ok(())
}

fn energy_shield(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "cannot use energy shield while not alive")?;

    let entity = &mut world.entities[entity_index];
    entity
        .extension_mut()
        .start_energy_shield(
            common::skill::ENERGY_SHIELD_DURATION,
            common::skill::ENERGY_SHIELD_COOLDOWN,
        )?;

    log::warn!("EnergyShield activated for {:?}", entity.data().label);
    Ok(())
}

fn dredger_sacrifice(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let player = player_tuple.borrow_player();
    let (entity_index, player_alias) = match player.data.status {
        Status::Alive { entity_index, .. } => (entity_index, player.alias()),
        _ => return Err("not alive"),
    };
    drop(player);

    let entity = &world.entities[entity_index];
    let position = entity.transform.position;
    let direction = entity.transform.direction;
    let entity_type = entity.entity_type;

    log::warn!("DredgerSacrifice: {:?} sacrificing at {:?}", player_alias, position);

    world.remove(entity_index, DeathReason::Weapon(player_alias, entity_type));

    use crate::entity::unset_entity_id;

    let oil_platform = Entity {
        player: None,
        transform: Transform {
            position,
            direction,
            velocity: Velocity::ZERO,
        },
        guidance: Guidance {
            velocity_target: Velocity::ZERO,
            direction_target: direction,
        },
        entity_type: EntityType::OilPlatform,
        ticks: Ticks::ZERO,
        id: unset_entity_id(),
        altitude: Altitude::ZERO,
        frozen: Ticks::ZERO,
        altar_blessing: Ticks::ZERO,
    };
    world.add(oil_platform);

    log::warn!("DredgerSacrifice: OilPlatform spawned at {:?}", position);
    Ok(())
}

fn stealth(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];

    if entity.extension().stealth_cooldown_remaining() != Ticks::ZERO {
        return Err("stealth on cooldown");
    }

    world.entities[entity_index]
        .extension_mut()
        .start_stealth(common::skill::STEALTH_DURATION, common::skill::STEALTH_COOLDOWN)?;

    log::info!("Stealth activated for {:?}", player_tuple.borrow_player().alias());
    Ok(())
}

fn unjust_game(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
    target_id: EntityId,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];

    if entity.extension().unjust_game_cooldown_remaining() != Ticks::ZERO {
        return Err("unjust game on cooldown");
    }

    let visual_range = entity.data().sensors.visual.range;
    let center = entity.transform.position;
    let self_id = entity.id;

    let target_index = world
        .entities
        .iter_radius(center, visual_range)
        .find_map(|(idx, e)| {
            if e.id == target_id && e.id != self_id {
                Some(idx)
            } else {
                None
            }
        })
        .ok_or("target entity not found in range")?;

    world.entities[entity_index]
        .extension_mut()
        .start_unjust_game(common::skill::UNJUST_GAME_COOLDOWN)?;

    let src = &world.entities[entity_index];
    let dst = &world.entities[target_index];

    let src_transform = src.transform;
    let src_guidance = src.guidance;
    let src_altitude = src.altitude;

    let dst_transform = dst.transform;
    let dst_guidance = dst.guidance;
    let dst_altitude = dst.altitude;

    {
        let src_entity = &mut world.entities[entity_index];
        src_entity.transform = dst_transform;
        src_entity.guidance = dst_guidance;
        src_entity.altitude = dst_altitude;
    }
    {
        let dst_entity = &mut world.entities[target_index];
        dst_entity.transform = src_transform;
        dst_entity.guidance = src_guidance;
        dst_entity.altitude = src_altitude;
    }

    for idx in [entity_index, target_index] {
        let entity = &mut world.entities[idx];
        if entity.data().kind == EntityKind::Obstacle {
            entity.transform.velocity = Velocity::ZERO;
            entity.guidance.velocity_target = Velocity::ZERO;
            entity.altitude = Altitude::ZERO;
        }
    }

    world.entities.move_sector(entity_index);
    world.entities.move_sector(target_index);

    Ok(())
}

fn ironclad(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &mut world.entities[entity_index];
    entity.extension_mut().start_ironclad(
        common::skill::IRONCLAD_DURATION,
        common::skill::IRONCLAD_COOLDOWN,
    )?;

    log::info!("[SKILL] Ironclad activated");
    Ok(())
}

fn yamato_cannon(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];

    if entity.extension().yamato_cannon_cooldown_remaining() != Ticks::ZERO {
        return Err("yamato cannon on cooldown");
    }

    let center = entity.transform.position;
    let direction = entity.transform.direction;
    let range = common::skill::YAMATO_CANNON_RANGE;
    let width = common::skill::YAMATO_CANNON_WIDTH;

    world.entities[entity_index]
        .extension_mut()
        .start_yamato_cannon(common::skill::YAMATO_CANNON_COOLDOWN)?;

    let dir_vec = Vec2::new(direction.to_radians().cos(), direction.to_radians().sin());
    let perp_vec = Vec2::new(-dir_vec.y, dir_vec.x);
    let half_width = width / 2.0;

    let mut targets: Vec<_> = world
        .entities
        .iter_radius(center, range)
        .filter_map(|(target_index, target)| {
            if target_index == entity_index {
                return None;
            }
            let data = target.data();
            if !matches!(data.kind, EntityKind::Boat | EntityKind::Aircraft) {
                return None;
            }
            if target.is_friendly_to_player(Some(player_tuple)) {
                return None;
            }

            let offset = target.transform.position - center;
            let forward_dist = offset.dot(dir_vec);
            let lateral_dist = offset.dot(perp_vec).abs();

            if forward_dist > 0.0 && forward_dist <= range && lateral_dist <= half_width {
                Some(target_index)
            } else {
                None
            }
        })
        .collect();

    targets.sort_by(|a, b| b.cmp(a));
    for target_index in &targets {
        world.remove(*target_index, DeathReason::Unknown);
    }

    log::info!("[SKILL] Yamato Cannon fired, hit {} targets", targets.len());
    Ok(())
}

fn orbital_bombardment(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];
    let center = entity.transform.position;
    let radius = common::skill::ORBITAL_BOMBARDMENT_RADIUS;

    world.entities[entity_index]
        .extension_mut()
        .start_orbital_bombardment(
            common::skill::ORBITAL_BOMBARDMENT_DURATION,
            common::skill::ORBITAL_BOMBARDMENT_COOLDOWN,
        )?;

    let mut targets: Vec<_> = world
        .entities
        .iter_radius(center, radius)
        .filter_map(|(target_index, target)| {
            if target_index == entity_index {
                return None;
            }
            let data = target.data();
            if !matches!(data.kind, EntityKind::Boat | EntityKind::Aircraft) {
                return None;
            }
            if target.is_friendly_to_player(Some(player_tuple)) {
                return None;
            }
            Some(target_index)
        })
        .collect();

    targets.sort_by(|a, b| b.cmp(a));
    for target_index in &targets {
        world.remove(*target_index, DeathReason::Unknown);
    }

    log::info!(
        "[SKILL] Orbital Bombardment activated, hit {} targets",
        targets.len()
    );
    Ok(())
}

fn rift_storm(
    world: &mut World,
    player_tuple: &Arc<PlayerTuple<Server>>,
) -> Result<(), &'static str> {
    let entity_index = alive_entity_index(player_tuple, "not alive")?;
    let entity = &world.entities[entity_index];
    let center = entity.transform.position;
    let radius = common::skill::RIFT_STORM_RADIUS;

    world.entities[entity_index]
        .extension_mut()
        .start_rift_storm(common::skill::RIFT_STORM_COOLDOWN)?;

    let mut targets: Vec<_> = world
        .entities
        .iter_radius(center, radius)
        .filter_map(|(target_index, target)| {
            if target_index == entity_index {
                return None;
            }
            let data = target.data();
            if !matches!(data.kind, EntityKind::Boat | EntityKind::Aircraft) {
                return None;
            }
            if target.is_friendly_to_player(Some(player_tuple)) {
                return None;
            }
            Some(target_index)
        })
        .collect();

    targets.sort_by(|a, b| b.cmp(a));
    for target_index in &targets {
        world.remove(*target_index, DeathReason::Unknown);
    }

    log::info!("[SKILL] Rift Storm activated, hit {} targets", targets.len());
    Ok(())
}

fn sanitize_float(float: f32, valid: Range<f32>) -> Result<f32, &'static str> {
    if float.is_finite() {
        Ok(float.clamp(valid.start, valid.end))
    } else {
        Err("float not finite")
    }
}
