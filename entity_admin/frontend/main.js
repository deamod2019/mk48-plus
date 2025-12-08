const backendInput = document.getElementById('backend-url');
const healthStatus = document.getElementById('health-status');
const output = document.getElementById('output');
const jsonInput = document.getElementById('json-input');

document.getElementById('btn-health').addEventListener('click', async () => {
  const url = `${baseUrl()}/api/health`;
  try {
    const res = await fetch(url);
    const data = await res.json();
    healthStatus.textContent = data.ok ? 'OK' : 'FAIL';
  } catch (err) {
    healthStatus.textContent = 'error';
    log(err);
  }
});

document.getElementById('btn-load').addEventListener('click', async () => {
  await loadJson('/api/entities');
});

document.getElementById('btn-load-draft').addEventListener('click', async () => {
  await loadJson('/api/entities/draft');
});

document.getElementById('btn-validate').addEventListener('click', async () => {
  const payload = parseInput();
  if (!payload) return;
  await postJson('/api/validate', payload);
});

document.getElementById('btn-preview').addEventListener('click', async () => {
  const payload = parseInput();
  if (!payload) return;
  await postJson('/api/preview', payload);
});

document.getElementById('btn-commit').addEventListener('click', async () => {
  const payload = parseInput();
  if (!payload) return;
  const dryRun = document.getElementById('dry-run').checked;
  const query = dryRun ? '?dry_run=true' : '';
  await postJson(`/api/commit${query}`, payload);
});

function baseUrl() {
  return backendInput.value.replace(/\\/$/, '');
}

async function loadJson(path) {
  const url = `${baseUrl()}${path}`;
  try {
    const res = await fetch(url);
    const data = await res.json();
    jsonInput.value = JSON.stringify(data, null, 2);
    log({ source: path, loaded: true });
  } catch (err) {
    log(err);
  }
}

async function postJson(path, payload) {
  const url = `${baseUrl()}${path}`;
  try {
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(payload),
    });
    const data = await res.json();
    log(data);
  } catch (err) {
    log(err);
  }
}

function parseInput() {
  try {
    return JSON.parse(jsonInput.value);
  } catch (err) {
    log({ error: 'JSON 解析失败', detail: String(err) });
    return null;
  }
}

function log(data) {
  output.textContent = JSON.stringify(data, null, 2);
}
