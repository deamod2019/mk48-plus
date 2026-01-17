// SPDX-FileCopyrightText: 2025 Softbear, Inc.
// SPDX-License-Identifier: AGPL-3.0-or-later

use crate::ticks::Ticks;

/// 最高速度 (106节 = 54.56 m/s)
pub const ENGINE_BOOST_MAX_SPEED_MPS: f32 = 54.56;

/// 最高速度持续时间 (20秒)
pub const ENGINE_BOOST_MAX_DURATION: Ticks = Ticks::from_whole_secs(20);

/// 减速目标速度 (90节 = 46.30 m/s)
pub const ENGINE_BOOST_DECEL_SPEED_MPS: f32 = 46.30;

/// 减速阶段持续时间 (5秒)
pub const ENGINE_BOOST_DECEL_DURATION: Ticks = Ticks::from_whole_secs(5);

/// 冷却时间 (10秒)
pub const ENGINE_BOOST_COOLDOWN: Ticks = Ticks::from_whole_secs(10);
