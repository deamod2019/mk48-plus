export async function importExisting(baseUrl) {
    const res = await fetch(join(baseUrl, '/api/import_existing'), { method: 'POST' });
    if (!res.ok) {
        const text = await res.text();
        throw new Error(`import_existing failed: ${text}`);
    }
    const data = await res.json();
    // 后端会同时写入文件，返回不一定包含全量数据；这里重新 GET 一次确保最新
    return apiGet(baseUrl, '/api/entities');
}
export async function apiGet(baseUrl, path) {
    const res = await fetch(join(baseUrl, path));
    if (!res.ok)
        throw new Error(`GET ${path} ${res.status}`);
    return res.json();
}
export async function apiPost(baseUrl, path, body) {
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
function join(base, path) {
    return `${base.replace(/\/$/, '')}${path}`;
}
