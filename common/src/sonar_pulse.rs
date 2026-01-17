// SPDX-FileCopyrightText: 2025 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// 主动声纳探测半径 (1500m)
pub const SONAR_PULSE_RADIUS: f32 = 1500.0;

/// 标记持续时间 (10秒)
pub const SONAR_PULSE_DURATION: Ticks = Ticks::from_whole_secs(10);

/// 冷却时间 (30秒)
pub const SONAR_PULSE_COOLDOWN: Ticks = Ticks::from_whole_secs(30);
