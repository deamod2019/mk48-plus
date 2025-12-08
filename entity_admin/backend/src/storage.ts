import fs from 'fs/promises';
import path from 'path';
import type { Entities } from './types';

const dataDir = path.join(process.cwd(), '..', 'data');
const prodFile = path.join(dataDir, 'entities.json');
const draftFile = path.join(dataDir, 'entities.draft.json');

export async function readEntities(): Promise<Entities> {
  return readJson(prodFile);
}

export async function readDraftEntities(): Promise<Entities> {
  try {
    return await readJson(draftFile);
  } catch {
    return readEntities();
  }
}

export async function writeEntities(data: Entities) {
  await ensureDir();
  await fs.writeFile(prodFile, JSON.stringify(data, null, 2), 'utf-8');
}

export async function writeDraftEntities(data: Entities) {
  await ensureDir();
  await fs.writeFile(draftFile, JSON.stringify(data, null, 2), 'utf-8');
}

async function readJson(file: string): Promise<Entities> {
  try {
    const content = await fs.readFile(file, 'utf-8');
    return JSON.parse(content);
  } catch {
    return { ships: [], weapons: [], sprites: [] };
  }
}

async function ensureDir() {
  await fs.mkdir(dataDir, { recursive: true });
}
