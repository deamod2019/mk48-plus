# MK48 Plus — 完整工具链安装、构建与运行指南

## 1. 工具链安装

### 1.1 Rust (nightly-2022-08-14)

```bash
# 安装 rustup（如未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装指定 nightly 版本
rustup toolchain install nightly-2022-08-14

# 设为项目默认（在项目目录下）
rustup override set nightly-2022-08-14

# 添加 WASM 编译目标
rustup target add wasm32-unknown-unknown --toolchain nightly-2022-08-14
```

> [!IMPORTANT]
> 项目依赖多个 `#![feature(...)]`，必须使用 **nightly-2022-08-14**，其他版本可能无法编译。

### 1.2 Trunk (WASM 打包工具)

```bash
cargo install trunk --version 0.21.14
```

### 1.3 wasm-bindgen-cli

```bash
cargo install wasm-bindgen-cli --version 0.2.83
```

### 1.4 Python + Pillow（精灵图处理）

```bash
cd /tmp && python3 -m venv imgenv
source /tmp/imgenv/bin/activate
pip install pillow numpy
```

### 1.5 验证安装

```bash
rustup show                    # 应显示 nightly-2022-08-14
rustup target list --installed # 应包含 wasm32-unknown-unknown
trunk --version                # trunk 0.21.14
wasm-bindgen --version         # wasm-bindgen 0.2.83
```

---

## 2. 项目结构

```
mk48-plus/
├── common/          # 共享代码（实体定义、协议）
├── macros/          # 过程宏（实体属性解析）
├── client/          # 前端 (Rust → WASM)
├── server/          # 后端 (Rust native)
├── engine/          # 游戏引擎框架
├── sprite_sheet_packer/  # 精灵表打包器
├── assets/
│   ├── models/rendered/  # 各实体精灵图 (color0001.png)
│   └── sprites/          # 原始素材
├── admin/           # 管理后台
└── entity_admin/    # 实体管理工具
```

---

## 3. 构建流程（4步）

### 3.1 打包精灵表

将 `assets/models/rendered/` 下所有实体精灵打包为精灵表：

```bash
cd sprite_sheet_packer && cargo run --release
```

输出：
- `client/sprites_css.png` — CSS 精灵表
- `client/sprites_webgl.png` — WebGL 精灵表
- `client/src/sprites_webgl.json` — 精灵坐标

### 3.2 构建客户端 (WASM)

```bash
cd client && trunk build --release
```

输出到 `client/dist/`，包含编译后的 `.wasm` 文件和 `index.html`。

> [!TIP]
> 开发时可用 `trunk build`（不加 `--release`）加速编译，但运行较慢。

### 3.3 构建服务器

```bash
cd server && cargo build --release
```

输出 `target/release/server`。

> [!NOTE]
> 服务器通过 `minicdn` 宏将 `client/dist/` 嵌入二进制文件，因此**必须先构建客户端**。

### 3.4 一键构建脚本

```bash
# 在项目根目录执行
cd sprite_sheet_packer && cargo run --release && \
cd ../client && trunk build --release && \
cd ../server && cargo build --release
```

---

## 4. 运行

### 4.1 启动服务器

```bash
cd server && cargo run --release
```

服务器启动后默认监听 **http://localhost:8443**

### 4.2 访问游戏

打开浏览器访问：`http://localhost:8443`

### 4.3 强制刷新

修改代码重新构建后：**Cmd+Shift+R**（Mac）强制刷新浏览器缓存。

---

## 5. 开发工作流

### 添加新实体

1. 生成精灵图 → 处理（去白背景、裁剪）→ 放入 `assets/models/rendered/[Name]/color0001.png`
2. 编辑 `common/src/entity/_type.rs` 添加实体定义
3. 如有技能硬编码，更新 `client/src/game.rs` 和 `client/src/ui/ship_controls.rs`
4. 执行完整构建流程（3.1 → 3.2 → 3.3）

### 常见问题

| 问题 | 解决 |
|------|------|
| `minicdn` 编译失败 | 先构建客户端 `trunk build --release` |
| 精灵表打包 panic | 检查 color/normal 图片尺寸是否一致 |
| unused import 警告 | 仅为 warning，不影响编译 |
| Python PIL 找不到 | `source /tmp/imgenv/bin/activate` |
| 浏览器看不到更新 | **Cmd+Shift+R** 强制刷新 |
