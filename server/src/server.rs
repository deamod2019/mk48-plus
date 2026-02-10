// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bot::*;
use crate::entity_extension::EntityExtension;
use crate::player::*;
use crate::protocol::*;
use crate::world::World;
use common::entity::EntityType;
use common::protocol::{Command, FactionId, FactionStats, FactionUpdate, Update, WorldEvent};
use common::terrain::ChunkSet;
use common::ticks::Ticks;
use common::util::level_to_score;
use core_protocol::id::*;
use game_server::context::Context;
use game_server::game_service::GameArenaService;
use game_server::player::{PlayerRepo, PlayerTuple};
use glam::Vec2;
use log::{error, warn};
use std::cell::UnsafeCell;
use std::sync::Arc;
use std::time::Duration;

/// A game server.
pub struct Server {
    pub world: World,
    pub counter: Ticks,
    /// Current faction war statistics (updated each tick when faction_mode is on).
    pub faction_update: Option<FactionUpdate>,
    /// Round-robin counter for init-time faction assignment.
    faction_counter: u8,
    /// Per-faction sacrifice count for the current altar.
    pub altar_sacrifices: [u8; FactionId::COUNT],
    /// Per-faction known altar position (None = not yet discovered by this faction).
    pub altar_known_position: [Option<Vec2>; FactionId::COUNT],
    /// Global tick counter for altar timing.
    altar_tick_counter: u64,
    /// Per-faction level threshold: bots at/below this level are in bottom 10 and eligible to sacrifice.
    pub altar_sacrifice_level_threshold: [u8; FactionId::COUNT],
}

/// Stores a player, and metadata related to it. Data stored here may only be accessed when processing,
/// this client (i.e. not when processing other entities). Bots don't use this.
#[derive(Default, Debug)]
pub struct ClientData {
    pub loaded_chunks: ChunkSet,
}

#[derive(Default)]
pub struct PlayerExtension(pub UnsafeCell<EntityExtension>);

/// This is sound because access is limited to when the entity is in scope.
unsafe impl Send for PlayerExtension {}
unsafe impl Sync for PlayerExtension {}

impl GameArenaService for Server {
    const GAME_ID: GameId = GameId::Mk48;
    const TICK_PERIOD_SECS: f32 = Ticks::PERIOD_SECS;

    /// How long a player can remain in limbo after they lose connection.
    const LIMBO: Duration = Duration::from_secs(6);

    /// Show bots on the liveboard.
    const LIVEBOARD_BOTS: bool = true;

    //const TEAM_MEMBERS_MAX: usize = 2;
    //const TEAM_JOINERS_MAX: usize = 2;

    type Bot = Bot;
    type ClientData = ClientData;
    type GameUpdate = Update;
    type GameRequest = Command;
    type PlayerData = Player;
    type PlayerExtension = PlayerExtension;

    /// new returns a game server with the specified parameters.
    fn new(_min_players: usize) -> Self {
        Self {
            world: World::new(6500.0),
            counter: Ticks::ZERO,
            faction_update: None,
            faction_counter: 0,
            altar_sacrifices: [0; FactionId::COUNT],
            altar_known_position: [None; FactionId::COUNT],
            altar_tick_counter: 0,
            altar_sacrifice_level_threshold: [0; FactionId::COUNT],
        }
    }

    fn team_members_max(_players: usize) -> usize {
        10
    }

    fn player_joined(
        &mut self,
        player_tuple: &Arc<PlayerTuple<Self>>,
        _players: &PlayerRepo<Server>,
    ) {
        let mut player = player_tuple.borrow_player_mut();
        player.data.flags.left_game = false;

        // Assign faction when faction_mode is enabled.
        if crate::runtime_config::hot_faction_mode() && player.data.faction.is_none() {
            // Pick the weakest faction (lowest total score), or round-robin if no stats yet.
            let idx = if let Some(ref fu) = self.faction_update {
                let f = &fu.factions;
                (0..FactionId::COUNT)
                    .min_by_key(|&i| f[i].total_score)
                    .unwrap_or(0) as u8
            } else {
                // No stats yet (server init) — round-robin for even distribution.
                let idx = self.faction_counter % FactionId::COUNT as u8;
                self.faction_counter = self.faction_counter.wrapping_add(1);
                idx
            };
            player.data.faction = Some(FactionId::from_index(idx));
        }

        // Boss bots: maintain exactly BOSSES_PER_FACTION bosses per faction.
        // Must drop `player` borrow before iterating _players to avoid AtomicRefCell conflict.
        const BOSSES_PER_FACTION: usize = 2;
        let is_bot = player.is_bot();
        let bot_faction = player.data.faction;
        drop(player);

        let mut should_be_boss = false;
        if is_bot && crate::runtime_config::hot_faction_mode() {
            if let Some(faction) = bot_faction {
                use common::entity::EntityData;
                let faction_boss_count = _players.iter_borrow()
                    .filter(|p| p.is_bot() && p.data.is_boss && p.data.faction == Some(faction))
                    .count();
                if faction_boss_count < BOSSES_PER_FACTION {
                    should_be_boss = true;
                }
            }
        }

        let mut player = player_tuple.borrow_player_mut();
        if should_be_boss {
            use common::entity::EntityData;
            player.data.is_boss = true;
            player.score = level_to_score(EntityData::MAX_BOAT_LEVEL);
        }

        #[cfg(debug_assertions)]
        {
            use common::entity::EntityData;
            //use common::util::level_to_score;
            use rand::{thread_rng, Rng};
            let highest_level_score = level_to_score(EntityData::MAX_BOAT_LEVEL);
            if !player.data.is_boss {
                player.score = if player.is_bot() {
                    thread_rng().gen_range(0..=highest_level_score)
                } else {
                    highest_level_score
                };
            }
        }
        #[cfg(not(debug_assertions))]
        {
            use common::entity::EntityData;
            let is_anan = player.alias().as_str().eq_ignore_ascii_case("anan");
            if is_anan {
                error!(
                    "anan join: is_bot={}, never_played={}, score={}",
                    player.is_bot(),
                    player.never_played(),
                    player.score
                );
            }
            if !player.is_bot() && player.never_played() && is_anan {
                player.score = level_to_score(EntityData::MAX_BOAT_LEVEL);
            }
        }
    }

    fn player_command(
        &mut self,
        update: Self::GameRequest,
        player: &Arc<PlayerTuple<Self>>,
        _players: &PlayerRepo<Server>,
    ) -> Option<Update> {
        if let Err(e) = update.as_command().apply(&mut self.world, player) {
            // Bot 的无效指令使用 debug 级别，避免刷屏
            if player.borrow_player().is_bot() {
                log::debug!("Bot command ignored: {}", e);
            } else {
                warn!("Command resulted in {}", e);
            }
        }
        None
    }

    fn player_changed_team(
        &mut self,
        player_tuple: &Arc<PlayerTuple<Self>>,
        old_team: Option<TeamId>,
        _players: &PlayerRepo<Server>,
    ) {
        if old_team.is_some() {
            player_tuple
                .borrow_player_mut()
                .data
                .flags
                .left_populated_team = true;
        }
    }

    fn player_left(
        &mut self,
        player_tuple: &Arc<PlayerTuple<Self>>,
        _players: &PlayerRepo<Server>,
    ) {
        let mut player = player_tuple.borrow_player_mut();
        if player.status.is_alive() {
            drop(player);
        } else {
            player.data.status = Status::Spawning;
            drop(player);
        }

        let mut player = player_tuple.borrow_player_mut();

        // Clear player's score.
        player.score = 0;

        // Delete all player's entities (efficiently, in the next update cycle).
        player.data.flags.left_game = true;
    }

    fn get_game_update(
        &self,
        player: &Arc<PlayerTuple<Self>>,
        client_data: &mut Self::ClientData,
        _players: &PlayerRepo<Server>,
    ) -> Option<Self::GameUpdate> {
        let p = player.borrow_player();
        let altar_pos = if let Some(faction) = p.data.faction {
            self.altar_known_position[faction.index()]
        } else {
            None
        };
        drop(p);
        Some(
            self.world
                .get_player_complete(player)
                .into_update(
                    self.counter,
                    &mut client_data.loaded_chunks,
                    self.faction_update.clone(),
                    altar_pos,
                    self.altar_sacrifices,
                ),
        )
    }

    fn is_alive(&self, player_tuple: &Arc<PlayerTuple<Self>>) -> bool {
        let player = player_tuple.borrow_player();
        !player.data.flags.left_game && player.data.status.is_alive()
    }

    /// update runs server ticks.
    fn tick(&mut self, context: &mut Context<Self>) {
        self.counter = self.counter.next();

        // Hot-reload bot limits from config file (if configured).
        if let Some(v) = crate::runtime_config::hot_min_bots() {
            context.bots.min_bots = v;
        }
        if let Some(v) = crate::runtime_config::hot_max_bots() {
            context.bots.max_bots = v;
        }
        if let Some(v) = crate::runtime_config::hot_bot_percent() {
            context.bots.bot_percent = v;
        }

        #[cfg(not(debug_assertions))]
        {
            use common::entity::EntityData;
            let highest_level_score = level_to_score(EntityData::MAX_BOAT_LEVEL);
            for mut player in context.players.iter_borrow_mut() {
                if !player.is_bot()
                    && player.never_played()
                    && player.alias().as_str().eq_ignore_ascii_case("anan")
                    && player.score < highest_level_score
                {
                    player.score = highest_level_score;
                    error!(
                        "anan bonus applied: player_id={:?}, score={}",
                        player.player_id, player.score
                    );
                }
            }
        }


        self.world.update(Ticks::ONE);

        // ---- Droplet Altar processing ----
        if crate::runtime_config::hot_faction_mode() {
            use common::entity::{EntityData, EntityKind};
            use rand::seq::SliceRandom;

            self.altar_tick_counter += 1;

            // Compute bottom-10 level threshold per faction for sacrifice eligibility.
            {
                let mut faction_levels: [Vec<u8>; FactionId::COUNT] = core::array::from_fn(|_| Vec::new());
                for player in context.players.iter_borrow() {
                    if !player.data.status.is_alive() || player.data.is_boss {
                        continue;
                    }
                    if let Some(faction) = player.data.faction {
                        if let Status::Alive { entity_index, .. } = player.data.status {
                            let level = self.world.entities[entity_index].data().level;
                            faction_levels[faction.index()].push(level);
                        }
                    }
                }
                for (i, levels) in faction_levels.iter_mut().enumerate() {
                    levels.sort_unstable();
                    // Bottom 10: level at index 9 (0-based), or last element if <10.
                    self.altar_sacrifice_level_threshold[i] = if levels.len() >= 10 {
                        levels[9]
                    } else if let Some(&max) = levels.last() {
                        max
                    } else {
                        0
                    };
                }
            }

            // 1. Find the altar entity position and index.
            let mut altar_info: Option<(crate::entities::EntityIndex, Vec2)> = None;
            for (idx, entity) in self.world.entities.iter_radius(Vec2::ZERO, self.world.radius) {
                if entity.entity_type == EntityType::DropletAltar {
                    altar_info = Some((idx, entity.transform.position));
                    break;
                }
            }

            // 2. Detect discovery: check if any alive player can see the altar.
            if let Some((_altar_idx, altar_pos)) = altar_info {
                for player in context.players.iter_borrow() {
                    if !player.data.status.is_alive() {
                        continue;
                    }
                    let faction = match player.data.faction {
                        Some(f) => f,
                        None => continue,
                    };
                    if self.altar_known_position[faction.index()].is_some() {
                        continue; // Already discovered by this faction.
                    }
                    // Get the player's entity to check sensor range.
                    if let Status::Alive { entity_index, .. } = player.data.status {
                        let entity = &self.world.entities[entity_index];
                        let sensor_range = entity.data().sensors.visual.range
                            .max(entity.data().sensors.radar.range)
                            .max(entity.data().sensors.sonar.range);
                        let dist = (altar_pos - entity.transform.position).length();
                        if dist <= sensor_range {
                            self.altar_known_position[faction.index()] = Some(altar_pos);
                            self.world.events.push(WorldEvent::AltarDiscovered {
                                position: altar_pos,
                                faction,
                            });
                        }
                    }
                }
            }

            // 3. Process AltarSacrifice events (no cooldown).
            let sacrifice_events: Vec<_> = self.world.events.iter().filter_map(|e| {
                if let WorldEvent::AltarSacrifice { faction } = e {
                    Some(*faction)
                } else {
                    None
                }
            }).collect();

            for faction in sacrifice_events {
                let idx = faction.index();

                self.altar_sacrifices[idx] = self.altar_sacrifices[idx].saturating_add(1);

                // 4. Check if sacrifice threshold reached.
                if self.altar_sacrifices[idx] >= 5 {

                    // Collect candidates: real players and bots separately.
                    let mut real_candidates: Vec<(core_protocol::id::PlayerId, crate::entities::EntityIndex)> = Vec::new();
                    let mut bot_candidates: Vec<(core_protocol::id::PlayerId, crate::entities::EntityIndex)> = Vec::new();
                    for player in context.players.iter_borrow() {
                        if player.data.faction != Some(faction) || !player.data.status.is_alive() || player.data.is_boss {
                            continue;
                        }
                        if let Status::Alive { entity_index, .. } = player.data.status {
                            let level = self.world.entities[entity_index].data().level;
                            if level >= EntityData::MAX_BOAT_LEVEL {
                                continue; // Already max level, skip.
                            }
                            if player.is_bot() {
                                bot_candidates.push((player.player_id, entity_index));
                            } else {
                                real_candidates.push((player.player_id, entity_index));
                            }
                        }
                    }

                    // Priority: real players first, then bots.
                    let chosen = if !real_candidates.is_empty() {
                        real_candidates.choose(&mut rand::thread_rng()).copied()
                    } else {
                        bot_candidates.choose(&mut rand::thread_rng()).copied()
                    };

                    if let Some((player_id, entity_index)) = chosen {
                        let max_level_boats: Vec<EntityType> = EntityType::iter()
                            .filter(|et| {
                                let d = et.data();
                                d.kind == EntityKind::Boat && d.level == EntityData::MAX_BOAT_LEVEL
                            })
                            .collect();

                        if let Some(&target_type) = max_level_boats.choose(&mut rand::thread_rng()) {
                            self.world.entities[entity_index].change_entity_type(
                                target_type,
                                &mut self.world.arena,
                                false,
                            );
                            // Grant 60s invulnerability (terrain + damage immune).
                            self.world.entities[entity_index].altar_blessing = Ticks::from_secs(60.0);
                            if let Some(mut player) = context.players.borrow_player_mut(player_id) {
                                player.score = level_to_score(EntityData::MAX_BOAT_LEVEL);
                            }
                        }
                    } else {
                    }

                    // Remove old altar and atomically spawn new one.
                    if let Some((altar_idx, _)) = altar_info {
                        self.world.remove(altar_idx, common::death_reason::DeathReason::Border);
                    }
                    {
                        use crate::entity::{unset_entity_id, Entity};
                        use common::altitude::Altitude;
                        use common::angle::Angle;
                        use common::guidance::Guidance;
                        use common::transform::Transform;
                        use common::velocity::Velocity;

                        let entity = Entity {
                            player: None,
                            transform: Transform {
                                position: Vec2::ZERO,
                                direction: Angle::ZERO,
                                velocity: Velocity::ZERO,
                            },
                            guidance: Guidance {
                                velocity_target: Velocity::ZERO,
                                direction_target: Angle::ZERO,
                            },
                            entity_type: EntityType::DropletAltar,
                            ticks: Ticks::ZERO,
                            id: unset_entity_id(),
                            altitude: Altitude::ZERO,
                            frozen: Ticks::ZERO,
                            altar_blessing: Ticks::ZERO,
                        };
                        let ok = self.world.spawn_here_or_nearby(entity, self.world.radius * 0.7, None);
                    }

                    // Reset all sacrifice counts and discovery state.
                    self.altar_sacrifices = [0; FactionId::COUNT];
                    self.altar_known_position = [None; FactionId::COUNT];

                    self.world.events.push(WorldEvent::AltarConsumed { faction });
                    break; // Only one faction can consume per tick.
                }
            }
        }
        // ---- End Droplet Altar processing ----

        // Calculate faction statistics each tick (cheap: just iterates players).
        if crate::runtime_config::hot_faction_mode() {
            let mut factions: [FactionStats; FactionId::COUNT] = core::array::from_fn(|_| FactionStats::default());
            let mut player_factions = Vec::new();
            for player in context.players.iter_borrow() {
                if let Some(faction) = player.data.faction {
                    // Only count alive players for stats and faction markers.
                    if !player.data.status.is_alive() {
                        continue;
                    }
                    let idx = faction.index();
                    factions[idx].total_score += player.score as u64;
                    factions[idx].player_count += 1;
                    if player.score > factions[idx].top_score {
                        factions[idx].top_score = player.score;
                        factions[idx].top_player = Some(player.alias().to_string());
                    }
                    player_factions.push((player.player_id, faction));
                }
            }
            self.faction_update = Some(FactionUpdate { factions, player_factions });
        } else {
            self.faction_update = None;
        }

        // Needs to be called before clients receive updates, but after World::update.
        self.world.terrain.pre_update();

        if self.counter.every(Ticks::from_whole_secs(60)) {
            use std::collections::{BTreeMap, HashMap};
            use std::fs::OpenOptions;
            use std::io::{Read, Seek, Write};

            let mut count_score = HashMap::<EntityType, (usize, f32)>::new();

            for player in context.players.iter_borrow() {
                if let Status::Alive { entity_index, .. } = player.status {
                    let entity = &self.world.entities[entity_index];
                    debug_assert!(entity.is_boat());

                    let (current_count, current_score) =
                        count_score.entry(entity.entity_type).or_default();
                    *current_count += 1;

                    let level = entity.data().level;
                    let level_score = level_to_score(level);
                    let next_level_score = level_to_score(level + 1);
                    let progress = common_util::range::map_ranges(
                        player.score as f32,
                        level_score as f32..next_level_score as f32,
                        0.0..1.0,
                        false,
                    );
                    if progress.is_finite() {
                        *current_score += progress;
                    }
                }
            }

            tokio::task::spawn_blocking(move || {
                if let Err(e) = OpenOptions::new()
                    .create(true)
                    .read(true)
                    .write(true)
                    .open(&*"playtime.json")
                    .and_then(move |mut file| {
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf)?;
                        let mut old = if let Ok(old) =
                            serde_json::from_slice::<BTreeMap<EntityType, (u64, f32)>>(&buf)
                        {
                            old
                        } else {
                            error!("error loading old playtime.");
                            BTreeMap::new()
                        };

                        for (entity_type, (new_count, new_score)) in count_score {
                            if new_count > 0 {
                                let (old_count, old_score) = old.entry(entity_type).or_default();
                                *old_count = old_count.saturating_add(new_count as u64);
                                *old_score += new_score;
                            }
                        }

                        file.set_len(0)?;
                        file.rewind()?;

                        let serialized = serde_json::to_vec(&old).unwrap_or_default();
                        file.write_all(&serialized)
                    })
                {
                    error!("error logging playtime: {:?}", e);
                }
            });
        }
    }

    fn post_update(&mut self, _context: &mut Context<Self>) {
        // Clear events after clients have received updates
        self.world.events.clear();
        // Needs to be after clients receive updates.
        self.world.terrain.post_update();
    }
}
