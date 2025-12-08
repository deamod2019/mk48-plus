"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.validateEntities = validateEntities;
function validateEntities(data) {
    const errors = [];
    const warnings = [];
    if (!Array.isArray(data.ships))
        errors.push('ships must be array');
    if (!Array.isArray(data.weapons))
        errors.push('weapons must be array');
    if (!Array.isArray(data.sprites))
        errors.push('sprites must be array');
    const shipIds = new Set();
    const weaponIds = new Set();
    const spriteIds = new Set();
    for (const s of data.ships || []) {
        if (!s.id)
            errors.push('ship missing id');
        if (s.id) {
            if (shipIds.has(s.id))
                errors.push(`duplicate ship id: ${s.id}`);
            shipIds.add(s.id);
        }
        if (s.hp !== undefined && s.hp <= 0)
            errors.push(`ship ${s.id} hp must be > 0`);
        if (s.speed !== undefined && s.speed <= 0)
            errors.push(`ship ${s.id} speed must be > 0`);
    }
    for (const w of data.weapons || []) {
        if (!w.id)
            errors.push('weapon missing id');
        if (w.id) {
            if (weaponIds.has(w.id))
                errors.push(`duplicate weapon id: ${w.id}`);
            weaponIds.add(w.id);
        }
        if (w.damage !== undefined && w.damage <= 0)
            errors.push(`weapon ${w.id} damage must be > 0`);
        if (w.range !== undefined && w.range <= 0)
            errors.push(`weapon ${w.id} range must be > 0`);
        if (w.cooldown !== undefined && w.cooldown <= 0)
            errors.push(`weapon ${w.id} cooldown must be > 0`);
    }
    for (const sp of data.sprites || []) {
        if (sp.id) {
            if (spriteIds.has(sp.id))
                errors.push(`duplicate sprite id: ${sp.id}`);
            spriteIds.add(sp.id);
        }
    }
    for (const s of data.ships || []) {
        if (s.weapons) {
            for (const wid of s.weapons) {
                if (!weaponIds.has(wid))
                    errors.push(`ship ${s.id} references missing weapon ${wid}`);
            }
        }
    }
    return { errors, warnings };
}
