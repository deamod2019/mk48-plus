// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// Smoke screen radius in meters
pub const SMOKE_SCREEN_RADIUS: f32 = 200.0;

/// Smoke screen duration (30 seconds)
pub const SMOKE_SCREEN_DURATION: Ticks = Ticks::from_whole_secs(30);

/// Smoke screen cooldown (60 seconds - includes 30s duration + 30s post-effect cooldown)
pub const SMOKE_SCREEN_COOLDOWN: Ticks = Ticks::from_whole_secs(60);
