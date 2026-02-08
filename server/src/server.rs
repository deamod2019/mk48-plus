// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::bot::*;
use crate::entity_extension::EntityExtension;
use crate::player::*;
use crate::protocol::*;
use crate::world::World;
use common::entity::EntityType;
use common::protocol::{Command, FactionId, FactionStats, FactionUpdate, Update};
use common::terrain::ChunkSet;
use common::ticks::Ticks;
use common::util::level_to_score;
use core_protocol::id::*;
use game_server::context::Context;
use game_server::game_service::GameArenaService;
use game_server::player::{PlayerRepo, PlayerTuple};
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

        // Boss bots: first 2*FACTION_COUNT bots get max score and is_boss flag.
        const BOSSES_PER_FACTION: usize = 2;
        if player.is_bot() && crate::runtime_config::hot_faction_mode() {
            use common::entity::EntityData;
            // Bot IDs are sequential starting from 1; use faction_counter as proxy for bot index.
            let boss_total = BOSSES_PER_FACTION * FactionId::COUNT;
            if (self.faction_counter as usize) <= boss_total {
                player.data.is_boss = true;
                player.score = level_to_score(EntityData::MAX_BOAT_LEVEL);
            }
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
        Some(
            self.world
                .get_player_complete(player)
                .into_update(self.counter, &mut client_data.loaded_chunks, self.faction_update.clone()),
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
