"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const express_1 = __importDefault(require("express"));
const cors_1 = __importDefault(require("cors"));
const child_process_1 = require("child_process");
const path_1 = __importDefault(require("path"));
const multer_1 = __importDefault(require("multer"));
const sharp_1 = __importDefault(require("sharp"));
const fs_extra_1 = __importDefault(require("fs-extra"));
const storage_1 = require("./storage");
const validate_1 = require("./validate");
const diff_1 = require("./diff");
const app = (0, express_1.default)();
app.use((0, cors_1.default)());
app.use(express_1.default.json({ limit: '10mb' }));
app.use(express_1.default.urlencoded({ extended: true }));
const PORT = process.env.PORT ? Number(process.env.PORT) : 9000;
const repoRoot = path_1.default.join(process.cwd(), '..', '..');
const upload = (0, multer_1.default)({
    storage: multer_1.default.memoryStorage(),
    limits: { fileSize: 20 * 1024 * 1024 },
});
app.get('/api/health', (_req, res) => {
    res.json({ ok: true });
});
app.get('/api/entities', async (_req, res) => {
    const entities = await (0, storage_1.readEntities)();
    res.json(entities);
});
app.get('/api/entities/draft', async (_req, res) => {
    const entities = await (0, storage_1.readDraftEntities)();
    res.json(entities);
});
app.post('/api/validate', async (req, res) => {
    const payload = req.body;
    if (!payload) {
        return res.status(400).json({ ok: false, error: 'payload required' });
    }
    const result = (0, validate_1.validateEntities)(payload);
    res.json({ ok: result.errors.length === 0, ...result });
});
app.post('/api/preview', async (req, res) => {
    const payload = req.body;
    if (!payload) {
        return res.status(400).json({ ok: false, error: 'payload required' });
    }
    const validation = (0, validate_1.validateEntities)(payload);
    const current = await (0, storage_1.readEntities)();
    const diff = (0, diff_1.diffEntities)(current, payload);
    res.json({
        ok: validation.errors.length === 0,
        validation,
        diff,
    });
});
app.post('/api/commit', async (req, res) => {
    var _a;
    const payload = req.body;
    if (!payload) {
        return res.status(400).json({ ok: false, error: 'payload required' });
    }
    const validation = (0, validate_1.validateEntities)(payload);
    if (validation.errors.length > 0) {
        return res.status(400).json({ ok: false, validation });
    }
    const dryRun = String((_a = req.query.dry_run) !== null && _a !== void 0 ? _a : 'false') === 'true';
    const current = await (0, storage_1.readEntities)();
    const diff = (0, diff_1.diffEntities)(current, payload);
    if (dryRun) {
        return res.json({ ok: true, dry_run: true, diff });
    }
    await (0, storage_1.writeDraftEntities)(payload);
    await (0, storage_1.writeEntities)(payload);
    res.json({ ok: true, diff });
});
// 一键导入：调用 cargo run --bin dump_entities，然后读取生成的 client/src/entities_data.json 写入本地 data/entities.json
app.post('/api/import_existing', async (_req, res) => {
    try {
        const result = await runDumpEntities();
        await (0, storage_1.writeEntities)(result);
        await (0, storage_1.writeDraftEntities)(result);
        res.json({ ok: true, imported: true, counts: { ships: result.ships.length, weapons: result.weapons.length } });
    }
    catch (err) {
        console.error('import_existing failed', err);
        res.status(500).json({ ok: false, error: String(err) });
    }
});
// 素材上传并缩放：支持 main/thumbnail/icon 三种目标
app.post('/api/upload_sprite', upload.single('file'), async (req, res) => {
    const target = String(req.body.target || '').trim();
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
        let destDir;
        let destFile;
        if (target === 'main') {
            destDir = path_1.default.join(repoRoot, 'assets', 'models', 'rendered', entityId);
            destFile = 'color0001.png';
        }
        else if (target === 'thumb') {
            destDir = path_1.default.join(repoRoot, 'assets', 'models', 'rendered', entityId);
            destFile = 'thumb.png';
        }
        else {
            destDir = path_1.default.join(repoRoot, 'assets', 'sprites');
            const name = fileNameRaw || entityId || 'icon';
            destFile = name.endsWith('.png') ? name : `${name}.png`;
        }
        await fs_extra_1.default.ensureDir(destDir);
        const pipeline = (0, sharp_1.default)(req.file.buffer);
        const resized = width || height ? pipeline.resize(width || undefined, height || undefined, { fit: 'inside' }) : pipeline;
        const outputPath = path_1.default.join(destDir, destFile);
        await resized.png().toFile(outputPath);
        return res.json({
            ok: true,
            saved: path_1.default.relative(repoRoot, outputPath),
            info: { target, entityId, width: width || null, height: height || null },
        });
    }
    catch (err) {
        console.error('upload_sprite failed', err);
        return res.status(500).json({ ok: false, error: String(err) });
    }
});
app.listen(PORT, '0.0.0.0', () => {
    console.log(`entity-admin backend listening on ${PORT}`);
});
async function runDumpEntities() {
    const serverDir = path_1.default.join(process.cwd(), '..', '..', 'server');
    const clientEntities = path_1.default.join(process.cwd(), '..', '..', 'client', 'src', 'entities_data.json');
    await new Promise((resolve, reject) => {
        const proc = (0, child_process_1.spawn)('cargo', ['run', '--bin', 'dump_entities'], { cwd: serverDir, stdio: 'inherit' });
        proc.on('exit', (code) => {
            if (code === 0)
                return resolve();
            reject(new Error(`cargo run --bin dump_entities exited with code ${code}`));
        });
        proc.on('error', (err) => reject(err));
    });
    const fs = await Promise.resolve().then(() => __importStar(require('fs/promises')));
    const content = await fs.readFile(clientEntities, 'utf-8');
    const parsed = JSON.parse(content);
    return transformDumpEntities(parsed);
}
function transformDumpEntities(raw) {
    const weapons = (raw.weapons || []).map((w) => {
        var _a, _b, _c;
        return ({
            id: w.id,
            name: w.label,
            kind: w.kind,
            type: w.kind,
            damage: w.damage,
            cooldown: w.reload,
            reload: w.reload,
            speed: w.speed,
            range: (_c = (_b = (_a = w.range) !== null && _a !== void 0 ? _a : w.range_max) !== null && _b !== void 0 ? _b : w.range_min) !== null && _c !== void 0 ? _c : 0,
            sprite: { id: w.id },
        });
    });
    const ships = (raw.ships || []).map((s) => {
        var _a, _b, _c, _d;
        const weaponsList = [];
        if (Array.isArray(s.armaments)) {
            for (const a of s.armaments) {
                if (a === null || a === void 0 ? void 0 : a.weapon_id) {
                    const count = a.count && Number.isFinite(a.count) ? a.count : 1;
                    for (let i = 0; i < count; i++)
                        weaponsList.push(a.weapon_id);
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
            range: (_c = (_b = (_a = s.range) !== null && _a !== void 0 ? _a : s.range_max) !== null && _b !== void 0 ? _b : s.range_min) !== null && _c !== void 0 ? _c : 0,
            depth: s.depth,
            reload: s.reload,
            anti_air: s.anti_aircraft,
            torpedo_resistance: s.torpedo_resistance,
            stealth: s.stealth,
            cost: s.level,
            npc: s.npc,
            weapons: weaponsList,
            sprite: { id: s.id },
            description: `kind:${s.kind || ''} level:${(_d = s.level) !== null && _d !== void 0 ? _d : ''}`,
        };
    });
    return { ships, weapons, sprites: [] };
}
