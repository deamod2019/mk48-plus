// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::server::*;
use crate::world::World;
use common::protocol::*;
use game_server::player::PlayerTuple;
use std::sync::Arc;

/// All client->server commands use this unified interface.
pub trait CommandTrait {
    fn apply(
        &self,
        world: &mut World,
        player_tuple: &Arc<PlayerTuple<Server>>,
    ) -> Result<(), &'static str>;
}

pub trait AsCommandTrait {
    fn as_command(&self) -> &dyn CommandTrait;
}

impl AsCommandTrait for Command {
    fn as_command(&self) -> &dyn CommandTrait {
        match *self {
            Command::Control(ref v) => v as &dyn CommandTrait,
            Command::Spawn(ref v) => v as &dyn CommandTrait,
            Command::Upgrade(ref v) => v as &dyn CommandTrait,
            Command::Warp(ref v) => v as &dyn CommandTrait,
            Command::ZeroPulse(ref v) => v as &dyn CommandTrait,
            Command::Iaigiri(ref v) => v as &dyn CommandTrait,
            Command::EngineBoost(ref v) => v as &dyn CommandTrait,
            Command::SonarPulse(ref v) => v as &dyn CommandTrait,
            Command::DepthChargeBarrage(ref v) => v as &dyn CommandTrait,
            Command::AirSuperiority(ref v) => v as &dyn CommandTrait,
            Command::EmergencyRepair(ref v) => v as &dyn CommandTrait,
            Command::SmokeScreen(ref v) => v as &dyn CommandTrait,
            Command::BurstLoading(ref v) => v as &dyn CommandTrait,
            Command::NuclearStrike(ref v) => v as &dyn CommandTrait,
            Command::EnergyShield(ref v) => v as &dyn CommandTrait,
            Command::DredgerSacrifice(ref v) => v as &dyn CommandTrait,
            Command::Stealth(ref v) => v as &dyn CommandTrait,
        }
    }
}
