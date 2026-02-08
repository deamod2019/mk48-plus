---
description: Full build pipeline — pack sprites, build client, then build server
---

# Build All

Run these three steps in order from the project root `c:\Users\deamo\Documents\src\mk48-plus`.

// turbo-all

## 1. Pack sprite sheets

```shell
cargo run --release -p sprite_sheet_packer
```

Cwd: `c:\Users\deamo\Documents\src\mk48-plus`

This regenerates `client/sprites_*.png` and related assets from `assets/sprites/`.

## 2. Build client (trunk)

```shell
trunk build --release
```

Cwd: `c:\Users\deamo\Documents\src\mk48-plus\client`

Must complete **before** building the server.

## 3. Build server

```shell
cargo build --release
```

Cwd: `c:\Users\deamo\Documents\src\mk48-plus\server`

## Verification

After all three steps, confirm zero errors and zero warnings in the output.
