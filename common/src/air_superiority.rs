// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// Number of drones to deploy
pub const DRONE_COUNT: u32 = 10;

/// How long drones survive
pub const DRONE_DURATION: Ticks = Ticks::from_whole_secs(20);

/// Drone movement speed (m/s)
pub const DRONE_SPEED: f32 = 50.0;

/// Drone damage per hit
pub const DRONE_DAMAGE: f32 = 0.2;

/// Skill cooldown
pub const AIR_SUPERIORITY_COOLDOWN: Ticks = Ticks::from_whole_secs(45);
