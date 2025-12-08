import React, { useEffect, useMemo, useState } from 'react';
import './style.css';
import {
  Entities,
  ValidationResult,
  DiffResult,
  apiGet,
  apiPost,
  Ship,
  Weapon,
  SpriteRef,
  importExisting,
} from './api';

type Tab = 'browse' | 'ships' | 'weapons' | 'sprites' | 'preview' | 'assets';

const emptyEntities: Entities = { ships: [], weapons: [], sprites: [] };

export default function App() {
  const defaultBackend = useMemo(() => {
    // 尝试同源，若需要跨端口再手动修改
    return window.location.origin;
  }, []);
  const [backend, setBackend] = useState(defaultBackend);
  const [tab, setTab] = useState<Tab>('browse');
  const [entities, setEntities] = useState<Entities>(emptyEntities);
  const [log, setLog] = useState<string>('ready');
  const [validation, setValidation] = useState<ValidationResult | null>(null);
  const [diff, setDiff] = useState<DiffResult | null>(null);
  const [dryRun, setDryRun] = useState(true);
  const [selectedShipId, setSelectedShipId] = useState<string | null>(null);
  const [uploadTarget, setUploadTarget] = useState<'main' | 'thumb' | 'icon'>('main');
  const [uploadEntityId, setUploadEntityId] = useState('039A');
  const [uploadWidth, setUploadWidth] = useState<number | ''>(420);
  const [uploadHeight, setUploadHeight] = useState<number | ''>(90);
  const [uploadFileName, setUploadFileName] = useState('');
  const [uploadFile, setUploadFile] = useState<File | null>(null);
  const [uploadMsg, setUploadMsg] = useState('');
  const [mainFile, setMainFile] = useState<File | null>(null);
  const [thumbFile, setThumbFile] = useState<File | null>(null);
  const [iconYu6File, setIconYu6File] = useState<File | null>(null);
  const [iconYu7File, setIconYu7File] = useState<File | null>(null);
  const [iconYj82File, setIconYj82File] = useState<File | null>(null);
  const [iconTurboFile, setIconTurboFile] = useState<File | null>(null);
  const [uploading, setUploading] = useState(false);
  const [toast, setToast] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState<'ship' | 'icons'>('ship');
  const spriteSheet = useSpriteSheet();
  const selectedShip = useMemo(
    () => entities.ships.find((s) => s.id === selectedShipId) || null,
    [entities.ships, selectedShipId]
  );

  useEffect(() => {
    // 初始加载正式数据
    load('/api/entities');
  }, []);

  const weaponsById = useMemo(() => {
    const map = new Map<string, Weapon>();
    entities.weapons.forEach((w) => w.id && map.set(w.id, w));
    return map;
  }, [entities.weapons]);

  const spritesById = useMemo(() => {
    const map = new Map<string, SpriteRef>();
    entities.sprites.forEach((s) => s.id && map.set(s.id, s));
    return map;
  }, [entities.sprites]);

  async function load(path: string) {
    try {
      const data = await apiGet<Entities>(backend, path);
      setEntities(normalize(data));
      if (!selectedShipId && data.ships && data.ships.length > 0) {
        setSelectedShipId(data.ships[0].id);
      }
      setLog(`loaded ${path}`);
    } catch (err) {
      setLog(String(err));
    }
  }

  async function doValidate() {
    try {
      const result = await apiPost<{ ok: boolean; errors: string[]; warnings: string[] }>(
        backend,
        '/api/validate',
        entities
      );
      setValidation({ errors: result.errors, warnings: result.warnings });
      setLog(result.ok ? '校验通过' : '校验有错误');
    } catch (err) {
      setLog(String(err));
    }
  }

  async function doPreview() {
    try {
      const result = await apiPost<{ ok: boolean; validation: ValidationResult; diff: DiffResult }>(
        backend,
        '/api/preview',
        entities
      );
      setValidation(result.validation);
      setDiff(result.diff);
      setLog(result.ok ? '预览完成（无错误）' : '预览完成（校验有错误）');
    } catch (err) {
      setLog(String(err));
    }
  }

  async function doCommit() {
    try {
      const path = dryRun ? '/api/commit?dry_run=true' : '/api/commit';
      const result = await apiPost<{ ok: boolean; diff: DiffResult; validation?: ValidationResult }>(
        backend,
        path,
        entities
      );
      setDiff(result.diff);
      setLog(dryRun ? 'dry-run 完成' : '已提交写入');
    } catch (err) {
      setLog(String(err));
    }
  }

  async function doImport() {
    try {
      const data = await importExisting(backend);
      setEntities(normalize(data));
      if (data.ships && data.ships.length > 0) setSelectedShipId(data.ships[0].id);
      setLog('导入成功');
    } catch (err) {
      setLog(String(err));
    }
  }

  function updateShip(id: string, patch: Partial<Ship>) {
    setEntities((prev) => ({
      ...prev,
      ships: prev.ships.map((s) => (s.id === id ? { ...s, ...patch } : s)),
    }));
  }

  function addShip() {
    const id = prompt('新建 ship id?');
    if (!id) return;
    setEntities((prev) => ({ ...prev, ships: [...prev.ships, { id }] }));
  }

  function removeShip(id: string) {
    setEntities((prev) => ({ ...prev, ships: prev.ships.filter((s) => s.id !== id) }));
  }

  function updateWeapon(id: string, patch: Partial<Weapon>) {
    setEntities((prev) => ({
      ...prev,
      weapons: prev.weapons.map((w) => (w.id === id ? { ...w, ...patch } : w)),
    }));
  }

  function addWeapon() {
    const id = prompt('新建 weapon id?');
    if (!id) return;
    setEntities((prev) => ({ ...prev, weapons: [...prev.weapons, { id }] }));
  }

  function removeWeapon(id: string) {
    setEntities((prev) => ({ ...prev, weapons: prev.weapons.filter((w) => w.id !== id) }));
  }

  function addSprite() {
    const id = prompt('新建 sprite id?（可留空，仅引用文件）');
    const file = prompt('文件名/路径？（可留空）') || undefined;
    setEntities((prev) => ({ ...prev, sprites: [...prev.sprites, { id: id || undefined, file }] }));
  }

  function removeSprite(idx: number) {
    setEntities((prev) => {
      const next = [...prev.sprites];
      next.splice(idx, 1);
      return { ...prev, sprites: next };
    });
  }

  function loadFromTextarea(ev: React.ChangeEvent<HTMLTextAreaElement>) {
    try {
      const parsed = JSON.parse(ev.target.value);
      setEntities(normalize(parsed));
      setLog('已从文本载入 JSON');
    } catch (err) {
      setLog('JSON 解析失败: ' + String(err));
    }
  }

  const jsonText = JSON.stringify(entities, null, 2);

  async function handleUpload(opts: {
    target: 'main' | 'thumb' | 'icon';
    entityId?: string;
    fileName?: string;
    width?: number;
    height?: number;
    file: File | null;
  }) {
    const { target, entityId, fileName, width, height, file } = opts;
    if (target !== 'icon' && !entityId?.trim()) {
      setUploadMsg('请填写 entityId');
      return;
    }
    if (!file) {
      setUploadMsg('请选择文件');
      return;
    }

    const form = new FormData();
    form.append('file', file);
    form.append('target', target);
    if (target !== 'icon' && entityId) {
      form.append('entityId', entityId.trim());
    }
    if (fileName?.trim()) {
      form.append('fileName', fileName.trim());
    }
    if (width && width > 0) form.append('targetWidth', String(width));
    if (height && height > 0) form.append('targetHeight', String(height));

    try {
      setUploading(true);
      const resp = await fetch(`${backend}/api/upload_sprite`, { method: 'POST', body: form });
      const data = await resp.json();
      if (!resp.ok || !data.ok) throw new Error(data.error || `上传失败 status=${resp.status}`);
      const msg = `已保存到 ${data.saved}`;
      setUploadMsg(msg);
      setToast(msg);
      setTimeout(() => setToast(null), 3000);
    } catch (err) {
      const msg = String(err);
      setUploadMsg(msg);
      setToast(msg);
      setTimeout(() => setToast(null), 4000);
    } finally {
      setUploading(false);
    }
  }

  return (
    <div className="page">
      <div className="header">
        <h2 style={{ margin: 0 }}>Entity Admin (React)</h2>
        <div className="toolbar">
          <label>
            后端：
            <input
              type="text"
              value={backend}
              onChange={(e) => setBackend(e.target.value)}
              style={{ width: 220, marginLeft: 6 }}
            />
          </label>
          <button onClick={() => load('/api/entities')}>加载正式</button>
          <button onClick={() => load('/api/entities/draft')}>加载草稿</button>
          <button onClick={doValidate}>校验</button>
          <button onClick={doPreview}>预览</button>
          <button onClick={doCommit}>{dryRun ? '提交(dry-run)' : '提交'}</button>
          <label style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
            <input type="checkbox" checked={dryRun} onChange={(e) => setDryRun(e.target.checked)} />
            dry-run
          </label>
        </div>
      </div>

      <div className="tabs">
        {(['browse', 'ships', 'weapons', 'sprites', 'preview', 'assets'] as Tab[]).map((t) => (
          <div key={t} className={`tab ${tab === t ? 'active' : ''}`} onClick={() => setTab(t)}>
            {t}
          </div>
        ))}
      </div>

      {tab === 'browse' && (
        <div className="panel">
          <div className="list-actions">
            <button onClick={() => load('/api/entities')}>刷新</button>
            <button onClick={doImport}>从现有 rs 导入</button>
            <span className="monospace">ships: {entities.ships.length}, weapons: {entities.weapons.length}</span>
          </div>
          <div className="grid">
            {entities.ships.map((s) => (
              <div
                key={s.id}
                className="card"
                style={{ cursor: 'pointer', borderColor: selectedShipId === s.id ? '#4b6aa8' : '#1f2835' }}
                onClick={() => setSelectedShipId(s.id)}
              >
                <SpriteThumb spriteId={s.sprite?.id || s.id} sheet={spriteSheet} height={64} />
                <div className="field">
                  <strong>{s.name || s.id}</strong>
                  <div style={{ color: '#9fb0d0', fontSize: 12 }}>{s.class || s.kind}</div>
                </div>
                <div className="field">
                  <label>HP</label>
                  <span>{s.hp ?? '-'}</span>
                </div>
                <div className="field">
                  <label>速度</label>
                  <span>{s.speed ?? '-'}</span>
                </div>
                <div className="field">
                  <label>武器数</label>
                  <span>{s.weapons?.length ?? 0}</span>
                </div>
              </div>
            ))}
          </div>
          <div style={{ marginTop: 12 }}>
            {selectedShip && (
              <div className="card">
                <h3 style={{ marginTop: 0 }}>{selectedShip.name || selectedShip.id}</h3>
                <div style={{ color: '#9fb0d0', marginBottom: 6 }}>
                  {selectedShip.class || selectedShip.kind} {selectedShip.npc ? '(NPC)' : ''}
                </div>
                <SpriteThumb spriteId={selectedShip.sprite?.id || selectedShip.id} sheet={spriteSheet} height={120} />
                <div className="grid" style={{ gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))' }}>
                  <div className="field">
                    <label>HP</label>
                    <span>{selectedShip.hp ?? '-'}</span>
                  </div>
                  <div className="field">
                    <label>速度</label>
                    <span>{selectedShip.speed ?? '-'}</span>
                  </div>
                  <div className="field">
                    <label>质量/长度</label>
                    <span>{selectedShip.mass ?? selectedShip.length ?? '-'}</span>
                  </div>
                  <div className="field">
                    <label>吃水/深度</label>
                    <span>
                      draft {selectedShip.draft ?? '-'} / depth {selectedShip.depth ?? '-'}
                    </span>
                  </div>
                  <div className="field">
                    <label>射程/冷却</label>
                    <span>
                      range {selectedShip.range ?? '-'} / reload {selectedShip.reload ?? '-'}
                    </span>
                  </div>
                  <div className="field">
                    <label>防空/鱼雷抗性/隐身</label>
                    <span>
                      AA {selectedShip.anti_air ?? '-'} / torp {selectedShip.torpedo_resistance ?? '-'} / stealth{' '}
                      {selectedShip.stealth ?? '-'}
                    </span>
                  </div>
                  <div className="field">
                    <label>武器</label>
                    <div>
                      {(selectedShip.weapons || []).map((wid: string) => {
                        const w = weaponsById.get(wid);
                        return (
                          <div key={wid}>
                            {w?.name || wid}
                            {w ? ` (伤害${w.damage ?? '-'} 射程${w.range ?? '-'})` : ''}
                          </div>
                        );
                      })}
                    </div>
                  </div>
                  <div className="field">
                    <label>描述</label>
                    <div>{selectedShip.description || '-'}</div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {tab === 'ships' && (
        <div className="panel">
          <div className="list-actions">
            <button onClick={addShip}>新增 Ship</button>
            <span className="monospace">共 {entities.ships.length} 艘</span>
          </div>
          <div className="grid">
            {entities.ships.map((s) => (
              <div className="card" key={s.id}>
                <div className="field">
                  <label>ID</label>
                  <input value={s.id} onChange={(e) => updateShip(s.id, { id: e.target.value })} />
                </div>
                <div className="field">
                  <label>名称</label>
                  <input value={s.name || ''} onChange={(e) => updateShip(s.id, { name: e.target.value })} />
                </div>
                <div className="field">
                  <label>HP</label>
                  <input
                    type="number"
                    value={s.hp ?? ''}
                    onChange={(e) => updateShip(s.id, { hp: numOrUndefined(e.target.value) })}
                  />
                </div>
                <div className="field">
                  <label>速度</label>
                  <input
                    type="number"
                    value={s.speed ?? ''}
                    onChange={(e) => updateShip(s.id, { speed: numOrUndefined(e.target.value) })}
                  />
                </div>
                <div className="field">
                  <label>武器（逗号分隔 ID）</label>
                  <input
                    value={(s.weapons || []).join(',')}
                    onChange={(e) => updateShip(s.id, { weapons: splitIds(e.target.value) })}
                  />
                  <small style={{ color: '#9fb0d0' }}>
                    解析后：{(s.weapons || []).map((id) => weaponsById.get(id)?.name || id).join(', ')}
                  </small>
                </div>
                <div className="field">
                  <label>贴图 sprite id</label>
                  <input
                    value={s.sprite?.id || ''}
                    onChange={(e) => updateShip(s.id, { sprite: { ...(s.sprite || {}), id: e.target.value } })}
                  />
                  <small style={{ color: '#9fb0d0' }}>
                    {(s.sprite?.id && spritesById.get(s.sprite.id)?.atlas) || ''}
                  </small>
                </div>
                <div className="field">
                  <label>描述</label>
                  <textarea value={s.description || ''} onChange={(e) => updateShip(s.id, { description: e.target.value })} />
                </div>
                <button className="secondary" onClick={() => removeShip(s.id)}>
                  删除
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === 'weapons' && (
        <div className="panel">
          <div className="list-actions">
            <button onClick={addWeapon}>新增 Weapon</button>
            <span className="monospace">共 {entities.weapons.length} 个</span>
          </div>
          <div className="grid">
            {entities.weapons.map((w) => (
              <div className="card" key={w.id}>
                <div className="field">
                  <label>ID</label>
                  <input value={w.id} onChange={(e) => updateWeapon(w.id, { id: e.target.value })} />
                </div>
                <div className="field">
                  <label>名称</label>
                  <input value={w.name || ''} onChange={(e) => updateWeapon(w.id, { name: e.target.value })} />
                </div>
                <div className="field">
                  <label>类型</label>
                  <input value={w.type || ''} onChange={(e) => updateWeapon(w.id, { type: e.target.value })} />
                </div>
                <div className="field">
                  <label>伤害</label>
                  <input
                    type="number"
                    value={w.damage ?? ''}
                    onChange={(e) => updateWeapon(w.id, { damage: numOrUndefined(e.target.value) })}
                  />
                </div>
                <div className="field">
                  <label>射程</label>
                  <input
                    type="number"
                    value={w.range ?? ''}
                    onChange={(e) => updateWeapon(w.id, { range: numOrUndefined(e.target.value) })}
                  />
                </div>
                <div className="field">
                  <label>冷却</label>
                  <input
                    type="number"
                    step="0.01"
                    value={w.cooldown ?? ''}
                    onChange={(e) => updateWeapon(w.id, { cooldown: numOrUndefined(e.target.value) })}
                  />
                </div>
                <div className="field">
                  <label>贴图 sprite id</label>
                  <input
                    value={w.sprite?.id || ''}
                    onChange={(e) => updateWeapon(w.id, { sprite: { ...(w.sprite || {}), id: e.target.value } })}
                  />
                  <small style={{ color: '#9fb0d0' }}>
                    {(w.sprite?.id && spritesById.get(w.sprite.id)?.atlas) || ''}
                  </small>
                </div>
                <div className="field">
                  <label>描述</label>
                  <textarea value={w.description || ''} onChange={(e) => updateWeapon(w.id, { description: e.target.value })} />
                </div>
                <button className="secondary" onClick={() => removeWeapon(w.id)}>
                  删除
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === 'sprites' && (
        <div className="panel">
          <div className="list-actions">
            <button onClick={addSprite}>新增 Sprite</button>
            <span className="monospace">共 {entities.sprites.length} 条</span>
          </div>
          <div className="grid">
            {entities.sprites.map((sp, idx) => (
              <div className="card" key={`${sp.id || 'sprite'}-${idx}`}>
                <div className="field">
                  <label>ID</label>
                  <input
                    value={sp.id || ''}
                    onChange={(e) => updateSprite(idx, { ...sp, id: e.target.value || undefined })}
                  />
                </div>
                <div className="field">
                  <label>文件/atlas</label>
                  <input
                    value={sp.file || sp.atlas || ''}
                    onChange={(e) => updateSprite(idx, { ...sp, file: e.target.value || undefined })}
                  />
                </div>
                <div className="field">
                  <label>坐标 x,y,w,h</label>
                  <input
                    value={[sp.x, sp.y, sp.w, sp.h].map((v) => (v ?? '')).join(',')}
                    onChange={(e) => updateSprite(idx, parseCoords(sp, e.target.value))}
                  />
                </div>
                <button className="secondary" onClick={() => removeSprite(idx)}>
                  删除
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {tab === 'assets' && (
        <div className="panel">
          <div className="list-actions" style={{ flexDirection: 'column', alignItems: 'flex-start', gap: 6 }}>
            <div>上传并缩放素材：main/thumb 写入 <code>assets/models/rendered/&lt;id&gt;/</code>，icon 写入 <code>assets/sprites/</code>。</div>
            <div className="monospace">
              推荐：039A main 约 420x90，thumb 96x54 或 128x72；图标 64x64 透明背景，保留 2px padding。
            </div>
            <div className="pill-tabs">
              <div className={`pill ${activeSection === 'ship' ? 'active' : ''}`} onClick={() => setActiveSection('ship')}>
                039A 主图/缩略
              </div>
              <div className={`pill ${activeSection === 'icons' ? 'active' : ''}`} onClick={() => setActiveSection('icons')}>
                武器图标（含 TurbolaserBeam）
              </div>
            </div>
          </div>
          <div className="asset-grid" style={{ display: activeSection === 'ship' ? 'grid' : 'none' }}>
            <div className="card" style={{ gridColumn: '1 / -1' }}>
              <h4 style={{ marginTop: 0 }}>039A 顶视 / 缩略图</h4>
              <ul className="tips">
                <li>主图：<code>assets/models/rendered/039A/color0001.png</code>，建议 420x90，透明背景。</li>
                <li>缩略：<code>assets/models/rendered/039A/thumb.png</code> 或 <code>assets/sprites/039A_thumb.png</code>，建议 128x72（或 96x54）。</li>
              </ul>
              <div className="upload-row">
                <div>
                  <div className="label">主图 420x90</div>
                  <input type="file" accept=".png" onChange={(e) => setMainFile(e.target.files?.[0] || null)} />
                  <button
                    disabled={uploading}
                    onClick={() =>
                      handleUpload({ target: 'main', entityId: '039A', width: 420, height: 90, file: mainFile })
                    }
                  >
                    {uploading ? '上传中…' : '上传主图'}
                  </button>
                </div>
                {mainFile && (
                  <div className="preview">
                    <div className="label">预览</div>
                    <img
                      src={URL.createObjectURL(mainFile)}
                      alt="main preview"
                      style={{ width: 420, height: 90, objectFit: 'contain', background: '#0a0e14' }}
                    />
                  </div>
                )}
              </div>
              <div className="upload-row">
                <div>
                  <div className="label">缩略 128x72</div>
                  <input type="file" accept=".png" onChange={(e) => setThumbFile(e.target.files?.[0] || null)} />
                  <button
                    disabled={uploading}
                    onClick={() =>
                      handleUpload({
                        target: 'thumb',
                        entityId: '039A',
                        width: 128,
                        height: 72,
                        fileName: 'thumb.png',
                        file: thumbFile,
                      })
                    }
                  >
                    {uploading ? '上传中…' : '上传缩略'}
                  </button>
                </div>
                {thumbFile && (
                  <div className="preview">
                    <div className="label">预览</div>
                    <img
                      src={URL.createObjectURL(thumbFile)}
                      alt="thumb preview"
                      style={{ width: 128, height: 72, objectFit: 'contain', background: '#0a0e14' }}
                    />
                  </div>
                )}
              </div>
            </div>
          </div>
          <div className="asset-grid" style={{ display: activeSection === 'icons' ? 'grid' : 'none' }}>
            <div className="card" style={{ gridColumn: '1 / -1' }}>
              <h4 style={{ marginTop: 0 }}>武器图标（64x64，透明）</h4>
              <ul className="tips">
                <li>鱼-6：<code>assets/sprites/Yu6.png</code></li>
                <li>鱼-7：<code>assets/sprites/Yu7.png</code></li>
                <li>YJ-82：<code>assets/sprites/YJ82.png</code></li>
                <li>TurbolaserBeam：<code>assets/sprites/TurbolaserBeam.png</code></li>
              </ul>
              <div className="icon-grid">
                <IconUpload
                  label="Yu6"
                  file={iconYu6File}
                  onFile={setIconYu6File}
                  uploading={uploading}
                  onUpload={() =>
                    handleUpload({ target: 'icon', fileName: 'Yu6.png', width: 64, height: 64, file: iconYu6File })
                  }
                />
                <IconUpload
                  label="Yu7"
                  file={iconYu7File}
                  onFile={setIconYu7File}
                  uploading={uploading}
                  onUpload={() =>
                    handleUpload({ target: 'icon', fileName: 'Yu7.png', width: 64, height: 64, file: iconYu7File })
                  }
                />
                <IconUpload
                  label="YJ82"
                  file={iconYj82File}
                  onFile={setIconYj82File}
                  uploading={uploading}
                  onUpload={() =>
                    handleUpload({ target: 'icon', fileName: 'YJ82.png', width: 64, height: 64, file: iconYj82File })
                  }
                />
                <IconUpload
                  label="TurbolaserBeam"
                  file={iconTurboFile}
                  onFile={setIconTurboFile}
                  uploading={uploading}
                  onUpload={() =>
                    handleUpload({
                      target: 'icon',
                      fileName: 'TurbolaserBeam.png',
                      width: 64,
                      height: 64,
                      file: iconTurboFile,
                    })
                  }
                />
              </div>
              <div className="monospace" style={{ marginTop: 6 }}>{uploadMsg}</div>
            </div>
          </div>
          {toast && <div className="toast">{toast}</div>}
        </div>
      )}

      {tab === 'preview' && (
        <div className="panel">
          <div className="field">
            <label>直接编辑 JSON</label>
            <textarea className="monospace" value={jsonText} onChange={loadFromTextarea} />
          </div>
          <div className="field">
            <label>校验 / Diff 结果</label>
            <pre className="pre">{JSON.stringify({ validation, diff, log }, null, 2)}</pre>
          </div>
        </div>
      )}
    </div>
  );

  function updateSprite(idx: number, value: SpriteRef) {
    setEntities((prev) => {
      const next = [...prev.sprites];
      next[idx] = value;
      return { ...prev, sprites: next };
    });
  }
}

function numOrUndefined(v: string): number | undefined {
  if (v === '') return undefined;
  const n = Number(v);
  return Number.isFinite(n) ? n : undefined;
}

function splitIds(v: string): string[] {
  return v
    .split(',')
    .map((s) => s.trim())
    .filter(Boolean);
}

function parseCoords(sp: SpriteRef, raw: string): SpriteRef {
  const parts = raw.split(',').map((s) => s.trim());
  const [x, y, w, h] = parts.map((p) => (p === '' ? undefined : Number(p)));
  return { ...sp, x, y, w, h };
}

function normalize(data: Partial<Entities>): Entities {
  return {
    ships: Array.isArray(data.ships) ? data.ships : [],
    weapons: Array.isArray(data.weapons) ? data.weapons : [],
    sprites: Array.isArray(data.sprites) ? data.sprites : [],
  };
}

type SpriteSheetData = {
  sprites: Record<string, { uvs: [number, number][]; aspect: number }>;
};

function useSpriteSheet() {
  const [sheet, setSheet] = useState<SpriteSheetData | null>(null);
  const [image, setImage] = useState<HTMLImageElement | null>(null);

  useEffect(() => {
    fetch('/sprites_webgl.json')
      .then((res) => res.json())
      .then((data) => setSheet(data))
      .catch(() => {});
    const img = new Image();
    img.src = '/sprites_webgl.png';
    img.onload = () => setImage(img);
  }, []);

  return { sheet, image };
}

function SpriteThumb({ spriteId, sheet, height = 64 }: { spriteId?: string; sheet: ReturnType<typeof useSpriteSheet>; height?: number }) {
  const [url, setUrl] = useState<string | null>(null);

  useEffect(() => {
    if (!spriteId || !sheet.sheet || !sheet.image) return;
    const sprite = sheet.sheet.sprites[spriteId];
    if (!sprite) return;
    const img = sheet.image;
    const u = sprite.uvs.map((v) => v[0]);
    const v = sprite.uvs.map((v) => v[1]);
    const uMin = Math.min(...u);
    const uMax = Math.max(...u);
    const vMin = Math.min(...v);
    const vMax = Math.max(...v);
    const sx = Math.floor(uMin * img.naturalWidth);
    const sy = Math.floor(vMin * img.naturalHeight);
    const sw = Math.ceil((uMax - uMin) * img.naturalWidth);
    const sh = Math.ceil((vMax - vMin) * img.naturalHeight);
    const safeSw = Math.max(1, Math.min(sw, img.naturalWidth - sx));
    const safeSh = Math.max(1, Math.min(sh, img.naturalHeight - sy));
    if (sw <= 0 || sh <= 0) return;
    const canvas = document.createElement('canvas');
    canvas.width = safeSw;
    canvas.height = safeSh;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.drawImage(img, sx, sy, safeSw, safeSh, 0, 0, safeSw, safeSh);
    setUrl(canvas.toDataURL());
  }, [spriteId, sheet.sheet, sheet.image]);

  if (!url) return null;
  return (
    <img src={url} alt={spriteId} style={{ width: '100%', maxHeight: height, objectFit: 'contain', display: 'block' }} />
  );
}

type IconUploadProps = {
  label: string;
  file: File | null;
  uploading: boolean;
  onFile: (f: File | null) => void;
  onUpload: () => void;
};

function IconUpload({ label, file, uploading, onFile, onUpload }: IconUploadProps) {
  return (
    <div className="icon-upload">
      <div className="label">{label}</div>
      <input type="file" accept=".png" onChange={(e) => onFile(e.target.files?.[0] || null)} />
      <button disabled={uploading} onClick={onUpload}>
        {uploading ? '上传中…' : `上传 ${label}`}
      </button>
      {file && <img src={URL.createObjectURL(file)} alt={`${label} preview`} className="icon-preview" />}
    </div>
  );
}
