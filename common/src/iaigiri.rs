// SPDX-FileCopyrightText: 2025 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// 蓄力时间 (1 秒)
pub const IAIGIRI_CHARGE: Ticks = Ticks::from_whole_secs(1);

/// 冷却时间 (20 秒)
pub const IAIGIRI_COOLDOWN: Ticks = Ticks::from_whole_secs(20);

/// 最大跃迁距离系数 (相对视野范围)
pub const IAIGIRI_MAX_RANGE_SCALE: f32 = 0.8;

/// 路径上布置的水雷数量
pub const IAIGIRI_MINE_COUNT: u8 = 20;
