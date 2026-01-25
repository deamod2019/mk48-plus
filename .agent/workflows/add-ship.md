---
description: How to add a new ship with skills in MK48 Plus (complete workflow)
---

# 添加新船只完整工作流

## 使用说明

当用户请求添加新船只时，按以下步骤收集信息并执行。

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
| 炮塔 | OtoMelara76Mm, _458X1980MmR, _127X680MmR, Rim116 |
| 导弹 | Yj18, Hq9, Tomahawk, P270, P700, Essm |
| 鱼雷 | Yu7, Mark48, Mark54 |
| 飞机 | F35B, Seahawk, Harbin, J20 |

### 技能
| 技能 | 说明 | 需修改文件 |
|------|------|------------|
| `EmergencyRepair` | 紧急维修 | ship_controls.rs, game.rs |
| `SmokeScreen` | 烟幕 | ship_controls.rs, game.rs |
| `Warp` | 跃迁 | ship_controls.rs, game.rs |
| `EngineBoost` | 引擎加速 | ship_controls.rs, game.rs |
| 其他技能... | | |

---

## 第二步：精灵图处理

### 2.1 生成精灵图提示词模板
使用**白色背景**便于后期处理：

```
Top-down view of a [ship_type], [length] meters long, aspect ratio [length/width]:1, naval warship with [weapon_description], game sprite style, horizontal orientation with bow pointing right, white background, high detail, clean edges, no shadows
```

**示例** (120m × 16m = 7.5:1):
```
Top-down view of a very long and narrow battleship, 120 meters long, aspect ratio 7.5:1, extremely elongated hull shape with 4 main gun turrets, gray hull, game sprite style, horizontal orientation with bow pointing right, white background, high detail, clean edges, no shadows
```

### 2.2 精灵图处理步骤 ⚠️ 关键！

> **注意**: 不要拉伸图像！应裁剪空白区域并调整实体尺寸匹配精灵比例。

// turbo
#### 1. 创建目标目录
```bash
mkdir -p assets/models/rendered/[ShipName]
```

#### 2. 裁剪空白 + 去白色背景 + 自动判断方向 (一步完成)
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
    # 船头（尖端）像素较少，船尾（宽）像素较多
    arr = np.array(cropped)
    left_quarter = arr[:, :w//4, 3]  # 左1/4区域的alpha通道
    right_quarter = arr[:, -w//4:, 3]  # 右1/4区域
    
    left_density = np.sum(left_quarter > 0)
    right_density = np.sum(right_quarter > 0)
    
    # 如果左侧像素密度更低（船头在左），需要水平翻转
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
#[props(speed = [S])]
#[sensors(visual = [V], radar = [R])]
// 炮塔 - 注意 side 值不能超过 width/2
#[turret([Type], forward = [F], fast)]
// 武器 - side 值要适配舰体宽度
#[armament([Type], forward = [F], side = [S], symmetrical, vertical)]
#[exhaust(forward = [F])]
#[skills([Skill1], [Skill2])]  // 如有技能
[ShipName],
```

### 3.2 添加技能UI支持（如有技能）
**文件**: `client/src/ui/ship_controls.rs`

找到对应技能按钮函数，添加新船只：
```rust
// 原始
if status.entity_type != EntityType::ExistingShip {
// 修改为
if status.entity_type != EntityType::ExistingShip
    && status.entity_type != EntityType::NewShip
{
```

### 3.3 更新游戏逻辑（如有技能）
**文件**: `client/src/game.rs`

**A. 技能定时器 (~line 1660)**:
```rust
let has_new_ship = player_contact.view.entity_type() == Some(EntityType::NewShip);
if has_new_ship {
    self.update_[skill]_timers(elapsed_seconds);
}
```

**B. 技能触发函数** (搜索 `try_[skill]`):
```rust
if (contact.entity_type() == Some(EntityType::ExistingShip)
    || contact.entity_type() == Some(EntityType::NewShip))
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
cd ../server && cargo build
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
- [ ] 精灵图处理
  - [ ] **裁剪空白区域**（不拉伸！）
  - [ ] **去除白色背景**
  - [ ] **根据精灵比例调整实体尺寸**
- [ ] `_type.rs` - 实体定义（注意 side 值适配宽度）
- [ ] `ship_controls.rs` - 技能按钮（如有）
- [ ] `game.rs` - 技能逻辑（如有）
- [ ] 精灵表打包
- [ ] 客户端构建
- [ ] 服务器构建
- [ ] 验证精灵 aspect 与实体比例匹配