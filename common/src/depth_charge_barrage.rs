// SPDX-FileCopyrightText: 2025 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// 深弹数量
pub const DCB_COUNT: u32 = 12;

/// 扇形角度 (度)
pub const DCB_SPREAD_ANGLE: f32 = 120.0;

/// 抛射距离 (米)
pub const DCB_RANGE: f32 = 200.0;

/// 冷却时间 (25秒)
pub const DCB_COOLDOWN: Ticks = Ticks::from_whole_secs(25);
