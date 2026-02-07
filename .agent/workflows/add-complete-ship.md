---
description: How to add a complete new ship with AI-generated sprites and skills in MK48 Plus
---

# 添加完整船只工作流（含AI生成精灵图）

## 使用说明

当用户请求添加新船只时，按以下步骤收集信息、生成精灵图并执行。

---

## 第一步：收集船只参数

向用户确认以下参数：

### 基本属性
| 参数 | 必填 | 示例 |
|------|------|------|
| 名称 (label) | ✅ | "铁甲战神" |
| 代码名 (enum) | ✅ | IronWarrior |
| 等级 (level) | ✅ | 1-15 |
| 子类型 | ✅ | Destroyer/Cruiser/Battleship/Carrier/Starship |
| 长度 (length) | ✅ | 120.0 米 |
| 宽度 (width) | ✅ | 16.0 米 |
| 吃水 (draft) | 可选 | 6.0 米 |
| 桅杆 (mast) | 可选 | 15.0 米 |
| 速度 (speed) | ✅ | 28.0 节 |

### 武器配置
| 类型 | 武器示例 |
|------|----------|
| 炮塔 | OtoMelara76Mm, _458X1980MmR, _127X680MmR, Rim116, TurboLaserIITurret |
| 导弹 | Yj18, Hq9, Tomahawk, P270, P700, Essm |
| 鱼雷 | Yu7, Mark48, Mark54 |
| 飞机 | F35B, Seahawk, Harbin, J20, TieBomber |

### 技能
| 技能 | 说明 |
|------|------|
| `Warp` | 跃迁 |
| `BurstLoading` | 爆发装填 |
| `EmergencyRepair` | 紧急维修 |
| `SmokeScreen` | 烟幕 |
| `EngineBoost` | 引擎加速 |
| `ZeroPulse` | 冰冻脉冲 |
| `Iaigiri` | 居合斩 |
| `SonarPulse` | 声纳脉冲 |
| `NuclearStrike` | 核打击 |
| `EnergyShield` | 能量护盾 |

---

## 第二步：精灵图生成与处理

### 2.1 生成精灵图提示词

根据船只类型生成对应提示词：

**船体提示词模板：**
```
Top-down view of a [ship_type], [length] meters long, aspect ratio [length/width]:1, [hull_description], game sprite, horizontal orientation with bow pointing right, white background, high detail, clean edges, no shadows
```

**炮塔提示词模板：**
```
Top-down view of a [turret_type], dark gray metallic, [barrel_description], game sprite, horizontal orientation pointing right, white background, high detail, clean edges, no shadows, isolated weapon turret
```

### 2.2 调用 Nano Banana Pro 生成图片

使用 `generate_image` 工具调用 Nano Banana Pro 生成 PNG 精灵图。
- 船体：生成1张
- 自定义炮塔（如有）：每种炮塔各1张

### 2.3 精灵图处理步骤 ⚠️ 关键！

> **注意**: 不要拉伸图像！应裁剪空白区域并调整实体尺寸匹配精灵比例。

// turbo
#### 1. 创建目标目录
```bash
mkdir -p assets/models/rendered/[ShipName]
```

#### 2. 格式检查 + RGBA转换 + 裁剪 + 去白色背景 + 方向检测（一步完成）
```bash
source /tmp/imgenv/bin/activate 2>/dev/null || (cd /tmp && python3 -m venv imgenv && source imgenv/bin/activate && pip install pillow numpy --quiet)

python3 -c "
from PIL import Image
import numpy as np

img = Image.open('[源文件路径]')
img = img.convert('RGBA')
data = np.array(img)

# 去除白色背景
r, g, b, a = data[:,:,0], data[:,:,1], data[:,:,2], data[:,:,3]
white_mask = (r > 230) & (g > 230) & (b > 230)
data[:,:,3] = np.where(white_mask, 0, 255)

result = Image.fromarray(data)

# 裁剪空白区域
bbox = result.getbbox()
if bbox:
    cropped = result.crop(bbox)
    w, h = cropped.size
    
    # 自动判断船方向：检测左右两侧的像素密度
    arr = np.array(cropped)
    left_quarter = arr[:, :w//4, 3]
    right_quarter = arr[:, -w//4:, 3]
    
    left_density = np.sum(left_quarter > 0)
    right_density = np.sum(right_quarter > 0)
    
    if left_density < right_density * 0.8:
        print('检测到船头朝左，正在水平翻转...')
        cropped = cropped.transpose(Image.FLIP_LEFT_RIGHT)
    else:
        print('船头朝右，无需翻转')
    
    print(f'裁剪后尺寸: {cropped.size}')
    print(f'实际比例: {cropped.size[0] / cropped.size[1]:.2f}')
    cropped.save('[目标路径]/color0001.png')
    print('保存成功!')
"
```

> **JPEG陷阱**: Nano Banana 有时生成伪PNG（实际是JPEG）。上面的 `Image.open().convert('RGBA').save()` 可自动修复此问题。

#### 3. 根据精灵比例调整实体尺寸 ⚠️ 重要！
- 如果精灵比例为 X:1，实体 length/width 也应为 X
- 例如：精灵 2616×341 (比例 7.67) → 实体 120m × 16m (比例 7.5)

---

## 第三步：代码修改

### 3.1 添加实体定义
**文件**: `common/src/entity/_type.rs`

```rust
// ============ Level [N] [ShipName] ============
#[info(label = "[显示名称]")]
#[entity(Boat, [SubKind], level = [N])]
#[size(length = [L], width = [W], draft = [D], mast = [M])]
#[props(speed = [S], damage = [DMG])]
#[sensors(visual = [V], radar = [R])]
#[turret([Type], forward = [F], fast)]
#[armament([Type], forward = [F], side = [S], symmetrical, vertical)]
#[exhaust(forward = [F])]
#[skills([Skill1], [Skill2])]
[ShipName],
```

> **side 值不能超过 width/2**

### 3.2 添加技能硬编码支持 ⚠️ 必须！

每个技能都有**多处硬编码**需要添加新实体，否则技能UI不会显示或无法触发。

#### A. UI 按钮显示
**文件**: `client/src/ui/ship_controls.rs`

每个技能对应一个 `xxx_button()` 函数，需要将新船只加入白名单：

| 技能 | 函数 | 位置 |
|------|------|------|
| Warp | `warp_button()` | ~211 行 |
| ZeroPulse | `zero_pulse_button()` | ~250 行 |
| Iaigiri | `iaigiri_button()` | ~274 行 |
| EngineBoost | `engine_boost_button()` | ~306 行 |
| SonarPulse | `sonar_pulse_button()` | ~338 行 |
| BurstLoading | `burst_loading_button()` | 搜索 `BurstLoading` |
| EmergencyRepair | `emergency_repair_button()` | 搜索 `EmergencyRepair` |
| SmokeScreen | `smoke_screen_button()` | 搜索 `SmokeScreen` |
| NuclearStrike | `nuclear_strike_button()` | 搜索 `NuclearStrike` |
| EnergyShield | `energy_shield_button()` | 搜索 `EnergyShield` |

**修改示例**:
```rust
if status.entity_type != EntityType::StarDestroyer
    && status.entity_type != EntityType::XystonStarDestroyer
    && status.entity_type != EntityType::YourNewShip  // ← 添加
{
    return Html::default();
}
```

#### B. 游戏定时器更新
**文件**: `client/src/game.rs` (~1608 行区域)

修改 `is_xxx` 或 `has_xxx` 变量，将新船只添加到 `matches!()` 宏：

| 技能 | 变量名 |
|------|--------|
| Warp | `is_star_destroyer` |
| ZeroPulse | `has_zero_pulse` |
| Iaigiri | `has_iaigiri` |
| BurstLoading | `has_burst_loading` |

```rust
let is_star_destroyer = matches!(
    player_contact.view.entity_type(),
    Some(EntityType::StarDestroyer | EntityType::XystonStarDestroyer | EntityType::YourNewShip)
);
```

#### C. UI 事件处理
**文件**: `client/src/game.rs` (~1951 行区域)

处理按钮点击事件的 `match` 分支，将新船只添加到 `matches!()` 宏：

| 技能 | 事件 |
|------|------|
| Warp | `UiEvent::WarpToggle` |
| ZeroPulse | `UiEvent::ZeroPulse` |
| Iaigiri | `UiEvent::IaigiriToggle` |
| BurstLoading | `UiEvent::BurstLoading` |

```rust
UiEvent::WarpToggle => {
    if matches!(
        contact.entity_type(),
        Some(EntityType::StarDestroyer | EntityType::XystonStarDestroyer | EntityType::YourNewShip)
    )
```

#### 搜索技能硬编码位置
// turbo
```bash
grep -rn "StarDestroyer.*XystonStarDestroyer" client/src/
```

---

## 第四步：编译和构建

// turbo-all
```bash
# 1. 打包精灵表
cd sprite_sheet_packer && cargo run --release

# 2. 构建客户端
cd ../client && trunk build --release

# 3. 构建服务器
cd ../server && cargo build --release
```

---

## 第五步：验证

// turbo
```bash
# 检查精灵比例
grep -o '"[ShipName]"[^}]*' client/src/sprites_webgl.json
```

确保 `aspect` 值与实体 `length/width` 比例接近。

---

## 快速检查清单

- [ ] 收集参数
- [ ] 精灵图生成
  - [ ] 生成提示词
  - [ ] 调用 Nano Banana Pro 生成 PNG
  - [ ] **RGBA 格式转换**（防止JPEG陷阱）
  - [ ] **裁剪空白区域**（不拉伸！）
  - [ ] **去除白色背景**
  - [ ] **根据精灵比例调整实体尺寸**
- [ ] `_type.rs` - 实体定义（注意 side 值适配宽度）
- [ ] **技能硬编码**（⚠️ 以下每个都要检查）：
  - [ ] `ship_controls.rs` - 技能按钮显示条件
  - [ ] `game.rs` ~1608 - 技能定时器变量
  - [ ] `game.rs` ~1951 - 技能事件处理
- [ ] 精灵表打包
- [ ] 客户端构建
- [ ] 服务器构建
- [ ] 验证精灵 aspect 与实体比例匹配
