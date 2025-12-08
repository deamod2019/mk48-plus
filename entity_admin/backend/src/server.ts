/// <reference path="./types-ext.d.ts" />
import express from 'express';
import cors from 'cors';
import { spawn } from 'child_process';
import path from 'path';
import multer from 'multer';
import sharp from 'sharp';
import fse from 'fs-extra';
import { readEntities, readDraftEntities, writeDraftEntities, writeEntities } from './storage';
import { validateEntities } from './validate';
import { diffEntities } from './diff';
import type { Entities, Ship, Weapon } from './types';

const app = express();
app.use(cors());
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true }));

// 默认与前端约定：端口 9000，HOST 0.0.0.0，若需可通过 env 覆盖
const PORT = process.env.PORT ? Number(process.env.PORT) : 9000;
const HOST = process.env.HOST || '0.0.0.0';
const repoRoot = path.join(process.cwd(), '..', '..');
const upload = multer({
  storage: multer.memoryStorage(),
  limits: { fileSize: 20 * 1024 * 1024 },
});

app.get('/api/health', (_req, res) => {
  res.json({ ok: true });
});

app.get('/api/entities', async (_req, res) => {
  const entities = await readEntities();
  res.json(entities);
});

app.get('/api/entities/draft', async (_req, res) => {
  const entities = await readDraftEntities();
  res.json(entities);
});

app.post('/api/validate', async (req, res) => {
  const payload = req.body as Entities | undefined;
  if (!payload) {
    return res.status(400).json({ ok: false, error: 'payload required' });
  }
  const result = validateEntities(payload);
  res.json({ ok: result.errors.length === 0, ...result });
});

app.post('/api/preview', async (req, res) => {
  const payload = req.body as Entities | undefined;
  if (!payload) {
    return res.status(400).json({ ok: false, error: 'payload required' });
  }
  const validation = validateEntities(payload);
  const current = await readEntities();
  const diff = diffEntities(current, payload);
  res.json({
    ok: validation.errors.length === 0,
    validation,
    diff,
  });
});

app.post('/api/commit', async (req, res) => {
  const payload = req.body as Entities | undefined;
  if (!payload) {
    return res.status(400).json({ ok: false, error: 'payload required' });
  }
  const validation = validateEntities(payload);
  if (validation.errors.length > 0) {
    return res.status(400).json({ ok: false, validation });
  }

  const dryRun = String(req.query.dry_run ?? 'false') === 'true';
  const current = await readEntities();
  const diff = diffEntities(current, payload);

  if (dryRun) {
    return res.json({ ok: true, dry_run: true, diff });
  }

  await writeDraftEntities(payload);
  await writeEntities(payload);
  res.json({ ok: true, diff });
});

// 一键导入：调用 cargo run --bin dump_entities，然后读取生成的 client/src/entities_data.json 写入本地 data/entities.json
app.post('/api/import_existing', async (_req, res) => {
  try {
    const result = await runDumpEntities();
    await writeEntities(result);
    await writeDraftEntities(result);
    res.json({ ok: true, imported: true, counts: { ships: result.ships.length, weapons: result.weapons.length } });
  } catch (err: any) {
    console.error('import_existing failed', err);
    res.status(500).json({ ok: false, error: String(err) });
  }
});

// 素材上传并缩放：支持 main/thumbnail/icon 三种目标
app.post('/api/upload_sprite', upload.single('file'), async (req, res) => {
  const target = String(req.body.target || '').trim() as 'main' | 'thumb' | 'icon';
  const entityId = String(req.body.entityId || '').trim();
  const fileNameRaw = String(req.body.fileName || '').trim();
  const width = req.body.targetWidth ? Number(req.body.targetWidth) : undefined;
  const height = req.body.targetHeight ? Number(req.body.targetHeight) : undefined;

  if (!req.file) {
    return res.status(400).json({ ok: false, error: '缺少文件 file' });
  }
  if (!target) {
    return res.status(400).json({ ok: false, error: '缺少 target' });
  }
  if (target !== 'icon' && !entityId) {
    return res.status(400).json({ ok: false, error: '缺少 entityId' });
  }

  try {
    let destDir: string;
    let destFile: string;
    if (target === 'main') {
      destDir = path.join(repoRoot, 'assets', 'models', 'rendered', entityId);
      destFile = 'color0001.png';
    } else if (target === 'thumb') {
      destDir = path.join(repoRoot, 'assets', 'models', 'rendered', entityId);
      destFile = 'thumb.png';
    } else {
      destDir = path.join(repoRoot, 'assets', 'sprites');
      const name = fileNameRaw || entityId || 'icon';
      destFile = name.endsWith('.png') ? name : `${name}.png`;
    }

    await fse.ensureDir(destDir);
    const pipeline = sharp(req.file.buffer);
    const resized = width || height ? pipeline.resize(width || undefined, height || undefined, { fit: 'inside' }) : pipeline;
    const outputPath = path.join(destDir, destFile);
    await resized.png().toFile(outputPath);

    return res.json({
      ok: true,
      saved: path.relative(repoRoot, outputPath),
      info: { target, entityId, width: width || null, height: height || null },
    });
  } catch (err: any) {
    console.error('upload_sprite failed', err);
    return res.status(500).json({ ok: false, error: String(err) });
  }
});

app.listen(PORT, HOST as any, () => {
  console.log(`entity-admin backend listening on http://${HOST}:${PORT}`);
});

async function runDumpEntities(): Promise<Entities> {
  const serverDir = path.join(process.cwd(), '..', '..', 'server');
  const clientEntities = path.join(process.cwd(), '..', '..', 'client', 'src', 'entities_data.json');

  await new Promise<void>((resolve, reject) => {
    const proc = spawn('cargo', ['run', '--bin', 'dump_entities'], { cwd: serverDir, stdio: 'inherit' });
    proc.on('exit', (code) => {
      if (code === 0) return resolve();
      reject(new Error(`cargo run --bin dump_entities exited with code ${code}`));
    });
    proc.on('error', (err) => reject(err));
  });

  const fs = await import('fs/promises');
  const content = await fs.readFile(clientEntities, 'utf-8');
  const parsed = JSON.parse(content) as any;
  return transformDumpEntities(parsed);
}

function transformDumpEntities(raw: any): Entities {
  const weapons: Weapon[] = (raw.weapons || []).map((w: any) => ({
    id: w.id,
    name: w.label,
    kind: w.kind,
    type: w.kind,
    damage: w.damage,
    cooldown: w.reload,
    reload: w.reload,
    speed: w.speed,
    range: w.range ?? w.range_max ?? w.range_min ?? 0,
    sprite: { id: w.id },
  }));

  const ships: Ship[] = (raw.ships || []).map((s: any) => {
    const weaponsList: string[] = [];
    if (Array.isArray(s.armaments)) {
      for (const a of s.armaments) {
        if (a?.weapon_id) {
          const count = a.count && Number.isFinite(a.count) ? a.count : 1;
          for (let i = 0; i < count; i++) weaponsList.push(a.weapon_id);
        }
      }
    }
    return {
      id: s.id,
      name: s.label,
      kind: s.kind,
      sub_kind: s.sub_kind,
      class: s.sub_kind || s.kind,
      hp: s.health,
      speed: s.speed,
      mass: s.length,
      length: s.length,
      draft: s.draft,
      range: s.range ?? s.range_max ?? s.range_min ?? 0,
      depth: s.depth,
      reload: s.reload,
      anti_air: s.anti_aircraft,
      torpedo_resistance: s.torpedo_resistance,
      stealth: s.stealth,
      cost: s.level,
      npc: s.npc,
      weapons: weaponsList,
      sprite: { id: s.id },
      description: `kind:${s.kind || ''} level:${s.level ?? ''}`,
    };
  });

  return { ships, weapons, sprites: [] };
}
