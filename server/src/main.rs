// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

#![feature(drain_filter)]
#![feature(new_uninit)]
#![feature(get_mut_unchecked)]
#![feature(async_closure)]
#![feature(hash_drain_filter)]
#![feature(type_alias_impl_trait)]
#![feature(generic_associated_types)]

//! The game server has authority over all game logic. Clients are served the client, which connects
//! via websocket.

use crate::server::Server;
use common::entity::EntityType;

mod arena;
mod bot;
mod collision;
mod complete_ref;
mod contact_ref;
mod entities;
mod entity;
mod entity_extension;
mod noise;
mod player;
mod protocol;
mod runtime_config;
mod server;
mod world;
mod world_inbound;
mod world_mutation;
mod world_outbound;
mod world_physics;
mod world_physics_radius;
mod world_spawn;
#[cfg(test)]
mod world_test;

fn main() {
    unsafe {
        noise::init();

        for typ in EntityType::iter() {
            rustrict::add_word(typ.as_str(), rustrict::Type::SAFE);
        }
    }

    // Start hot-reload config file watcher.
    runtime_config::start_config_watcher(None);

    // Env var override: set FACTION_MODE=1 to enable faction mode on startup.
    if std::env::var("FACTION_MODE").map_or(false, |v| v == "1" || v == "true") {
        runtime_config::set_faction_mode(true);
        eprintln!("[ENV] Faction mode enabled via FACTION_MODE=1");
    }

    game_server::entry_point::entry_point::<Server>(
        minicdn::release_include_mini_cdn!("../../client/dist/"),
        true,
    );
}
