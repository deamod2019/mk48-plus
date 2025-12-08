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
  mass?: number;
  length?: number;
  draft?: number;
  range?: number;
  depth?: number;
  reload?: number;
  anti_air?: number;
  torpedo_resistance?: number;
  stealth?: number;
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
