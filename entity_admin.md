## entity_admin 设计与进度快照

### 目录与运行
- 位置：`entity_admin/`（独立于主项目）。
- 运行脚本：`run.sh`（启动后端端口 9000、前端端口 5179，前端 Vite dev 代理 `/api` 至后端）。
- 默认后端基址：同源（前端通过代理访问 `/api`）。

### 后端（Node/Express，CJS）
- 文件：`backend/src/server.ts`。
- 主要接口：
  - `GET /api/health` 简单健康检查。
  - `GET /api/entities` 返回当前正式 JSON。
  - `GET /api/entities/draft` 返回草稿 JSON。
  - `POST /api/validate` 校验提交的 JSON（当前严格：range=0 报错；挂载缺失武器报错）。
  - `POST /api/preview` 仅返回 payload。
  - `POST /api/commit?dry_run=bool` 落盘或 dry-run。
  - `POST /api/import_existing` 调用 server/bin/dump_entities 获取最新实体/武器/sprite 数据。
- dump_entities（Rust）增强点：
  - 从 `_type.rs` 解析武器属性（range、speed、reload、damage），覆盖 EntityData。
  - 收集 armaments + turrets，生成挂载列表。
  - ship.range = 若实体有 range>0 用之；否则取武器最大 range；否则取视觉传感器 range。
  - TurbolaserBeam 解析到 range=200000。
  - 许多武器仍为 0，因为 `_type.rs` 原始定义即为 0（如 Harpoon/Mark48 等）。

### 前端（React/Vite）
- 入口：`frontend/src/App.tsx`。
- 标签页：browse（列表 + 详情）、ships/weapons/sprites 表单、preview JSON、直接编辑 JSON。
- 详情布局参考 ships/levels 页；SpriteThumb 使用 `sprites_webgl.json/png` UV 裁剪（修复为 full width + maxHeight）。
- 预览/校验/提交按钮调用后端接口；默认加载正式或草稿。

### 资源
- `frontend/public/` 包含 `sprites_webgl.png` 与 `sprites_webgl.json`（来自 client，UV 用于裁剪）。

### 当前问题/待办
- 校验仍然对 `range==0`、挂载缺失武器报错；缺失武器多数是飞机/直升机/炮塔等非 weapon 定义（Seahawk、Type96 等），需放宽或补映射。
- 某些武器（_type.rs 未给 range）仍为 0；若需非 0，需从其他源推断或补充表。
- 浏览详情中已解析到非零 range（如 TurbolaserBeam 200000）；Olympias 没武器因为 _type.rs 中无 armament。

### 关键信息留存
- 不修改 `_type.rs`；允许从其他源文件解析补充。
- 构建已通过（Node 后端 + React 前端）；可以从 `http://10.0.1.234:5179` 访问 UI（通过 run.sh 启动）。 
