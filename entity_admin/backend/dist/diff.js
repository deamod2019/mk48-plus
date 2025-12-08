"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.diffEntities = diffEntities;
function diffEntities(base, target) {
    return {
        ships: diffList(base.ships, target.ships),
        weapons: diffList(base.weapons, target.weapons),
        sprites: diffList(base.sprites, target.sprites),
    };
}
function diffList(base, target) {
    const baseMap = new Map();
    const tgtMap = new Map();
    for (const item of base || []) {
        if (item.id)
            baseMap.set(item.id, item);
    }
    for (const item of target || []) {
        if (item.id)
            tgtMap.set(item.id, item);
    }
    const added = [];
    const removed = [];
    const changed = [];
    for (const [id] of tgtMap) {
        if (!baseMap.has(id))
            added.push(id);
    }
    for (const [id] of baseMap) {
        if (!tgtMap.has(id))
            removed.push(id);
    }
    for (const [id, item] of tgtMap) {
        const prev = baseMap.get(id);
        if (prev && JSON.stringify(prev) !== JSON.stringify(item)) {
            changed.push(id);
        }
    }
    return { added, removed, changed };
}
