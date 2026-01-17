// SPDX-FileCopyrightText: 2025 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

pub const ZERO_PULSE_RADIUS: f32 = 1000.0;
pub const ZERO_PULSE_DURATION: Ticks = Ticks::from_whole_secs(10);
pub const ZERO_PULSE_COOLDOWN: Ticks = Ticks::from_whole_secs(30);
