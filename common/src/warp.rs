// SPDX-FileCopyrightText: 2024 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// 耗时 3 秒的跃迁充能。
pub const WARP_CHARGE: Ticks = Ticks::from_whole_secs(3);
/// 跃迁完成后的主冷却（20 秒）。
pub const WARP_COOLDOWN: Ticks = Ticks::from_whole_secs(20);
/// 允许跃迁的最大偏移距离系数（相对可视距离）。
pub const WARP_MAX_RANGE_SCALE: f32 = 1.0;
