---
description: How to add an existing skill to a new ship in MK48 Plus
---

# 船只技能添加完整清单

## 概述

在 MK48 Plus 中为船只添加技能时，需要修改**多个文件**的**多处位置**。

---

## 1. 实体定义

**文件**: `common/src/entity/_type.rs`

```rust
#[skills(Warp)]  // 在船只定义中添加技能
YourNewShip,
```

---

## 2. UI 按钮显示

**文件**: `client/src/ui/ship_controls.rs`

每个技能都有一个 `xxx_button()` 函数，包含实体类型检查：

| 技能 | 函数 | 位置 |
|------|------|------|
| Warp | `warp_button()` | ~211 行 |
| ZeroPulse | `zero_pulse_button()` | ~250 行 |
| Iaigiri | `iaigiri_button()` | ~274 行 |
| EngineBoost | `engine_boost_button()` | ~306 行 |
| SonarPulse | `sonar_pulse_button()` | ~338 行 |

**修改示例**:
```rust
if status.entity_type != EntityType::StarDestroyer
    && status.entity_type != EntityType::XystonStarDestroyer
    && status.entity_type != EntityType::YourNewShip  // ← 添加
{
    return Html::default();
}
```

---

## 3. 游戏定时器更新

**文件**: `client/src/game.rs` (~1608 行区域)

修改 `is_xxx` 或 `has_xxx` 变量：

| 技能 | 变量名 |
|------|--------|
| Warp | `is_star_destroyer` |
| ZeroPulse | `has_zero_pulse` |
| Iaigiri | `has_iaigiri` |

**修改示例**:
```rust
let is_star_destroyer = matches!(
    player_contact.view.entity_type(),
    Some(EntityType::StarDestroyer | EntityType::XystonStarDestroyer | EntityType::YourNewShip)
);
```

---

## 4. UI 事件处理

**文件**: `client/src/game.rs` (~1951 行区域)

处理按钮点击事件的 `match` 分支：

| 技能 | 事件 |
|------|------|
| Warp | `UiEvent::WarpToggle` |
| ZeroPulse | `UiEvent::ZeroPulse` |
| Iaigiri | `UiEvent::IaigiriToggle` |

**修改示例**:
```rust
UiEvent::WarpToggle => {
    if matches!(
        contact.entity_type(),
        Some(EntityType::StarDestroyer | EntityType::XystonStarDestroyer | EntityType::YourNewShip)
    )
```

---

## 快速检查清单

- [ ] `common/src/entity/_type.rs` - `#[skills(...)]` 宏
- [ ] `client/src/ui/ship_controls.rs` - 按钮显示条件
- [ ] `client/src/game.rs` - 定时器更新 (~1608 行)
- [ ] `client/src/game.rs` - 事件处理 (~1951 行)

---

## 搜索命令

// turbo
```bash
grep -rn "StarDestroyer.*XystonStarDestroyer" client/src/
```
