import type { Entities } from './types';

export function diffEntities(base: Entities, target: Entities) {
  return {
    ships: diffList(base.ships, target.ships),
    weapons: diffList(base.weapons, target.weapons),
    sprites: diffList(base.sprites, target.sprites),
  };
}

function diffList<T extends { id?: string }>(base: T[], target: T[]) {
  const baseMap = new Map<string, T>();
  const tgtMap = new Map<string, T>();
  for (const item of base || []) {
    if (item.id) baseMap.set(item.id, item);
  }
  for (const item of target || []) {
    if (item.id) tgtMap.set(item.id, item);
  }

  const added: string[] = [];
  const removed: string[] = [];
  const changed: string[] = [];

  for (const [id] of tgtMap) {
    if (!baseMap.has(id)) added.push(id);
  }
  for (const [id] of baseMap) {
    if (!tgtMap.has(id)) removed.push(id);
  }
  for (const [id, item] of tgtMap) {
    const prev = baseMap.get(id);
    if (prev && JSON.stringify(prev) !== JSON.stringify(item)) {
      changed.push(id);
    }
  }

  return { added, removed, changed };
}
