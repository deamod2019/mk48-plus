// SPDX-FileCopyrightText: 2021 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// Repair duration in seconds
pub const REPAIR_DURATION: Ticks = Ticks::from_whole_secs(15);

/// Percentage of max HP restored (0.20 = 20%)
pub const REPAIR_AMOUNT: f32 = 0.20;

/// Speed penalty during repair (0.25 = 25% slower)
pub const REPAIR_SPEED_PENALTY: f32 = 0.25;

/// Skill cooldown
pub const EMERGENCY_REPAIR_COOLDOWN: Ticks = Ticks::from_whole_secs(60);
