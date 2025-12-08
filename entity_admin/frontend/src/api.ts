export interface SpriteRef {
  id?: string;
  atlas?: string;
  file?: string;
  x?: number;
  y?: number;
  w?: number;
  h?: number;
}

export interface Weapon {
  id: string;
  name?: string;
  kind?: string;
  type?: string;
  speed?: number;
  range?: number;
  damage?: number;
  cooldown?: number;
  reload?: number;
  sprite?: SpriteRef;
  description?: string;
}

export interface Ship {
  id: string;
  name?: string;
  class?: string;
  kind?: string;
  sub_kind?: string;
  hp?: number;
  speed?: number;
  length?: number;
  draft?: number;
  range?: number;
  depth?: number;
  reload?: number;
  anti_air?: number;
  torpedo_resistance?: number;
  stealth?: number;
  mass?: number;
  npc?: boolean;
  collision?: unknown;
  cost?: number;
  weapons?: string[];
  sprite?: SpriteRef;
  description?: string;
}

export interface Entities {
  ships: Ship[];
  weapons: Weapon[];
  sprites: SpriteRef[];
}

export interface ValidationResult {
  errors: string[];
  warnings: string[];
}

export interface DiffResult {
  ships: DiffList;
  weapons: DiffList;
  sprites: DiffList;
}

export interface DiffList {
  added: string[];
  removed: string[];
  changed: string[];
}

export async function importExisting(baseUrl: string): Promise<Entities> {
  const res = await fetch(join(baseUrl, '/api/import_existing'), { method: 'POST' });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`import_existing failed: ${text}`);
  }
  const data = await res.json();
  // 后端会同时写入文件，返回不一定包含全量数据；这里重新 GET 一次确保最新
  return apiGet<Entities>(baseUrl, '/api/entities');
}

export async function apiGet<T>(baseUrl: string, path: string): Promise<T> {
  const res = await fetch(join(baseUrl, path));
  if (!res.ok) throw new Error(`GET ${path} ${res.status}`);
  return res.json();
}

export async function apiPost<T>(baseUrl: string, path: string, body: unknown): Promise<T> {
  const res = await fetch(join(baseUrl, path), {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const text = await res.text();
    throw new Error(`POST ${path} ${res.status}: ${text}`);
  }
  return res.json();
}

function join(base: string, path: string) {
  return `${base.replace(/\/$/, '')}${path}`;
}
