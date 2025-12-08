"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.readEntities = readEntities;
exports.readDraftEntities = readDraftEntities;
exports.writeEntities = writeEntities;
exports.writeDraftEntities = writeDraftEntities;
const promises_1 = __importDefault(require("fs/promises"));
const path_1 = __importDefault(require("path"));
const dataDir = path_1.default.join(process.cwd(), '..', 'data');
const prodFile = path_1.default.join(dataDir, 'entities.json');
const draftFile = path_1.default.join(dataDir, 'entities.draft.json');
async function readEntities() {
    return readJson(prodFile);
}
async function readDraftEntities() {
    try {
        return await readJson(draftFile);
    }
    catch {
        return readEntities();
    }
}
async function writeEntities(data) {
    await ensureDir();
    await promises_1.default.writeFile(prodFile, JSON.stringify(data, null, 2), 'utf-8');
}
async function writeDraftEntities(data) {
    await ensureDir();
    await promises_1.default.writeFile(draftFile, JSON.stringify(data, null, 2), 'utf-8');
}
async function readJson(file) {
    try {
        const content = await promises_1.default.readFile(file, 'utf-8');
        return JSON.parse(content);
    }
    catch {
        return { ships: [], weapons: [], sprites: [] };
    }
}
async function ensureDir() {
    await promises_1.default.mkdir(dataDir, { recursive: true });
}
