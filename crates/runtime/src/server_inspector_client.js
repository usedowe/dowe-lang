(() => {
  const root = document.getElementById('dowe-server-inspector');
  const state = { manifest: null, tab: 'overview', selected: null, sourceOpen: false, endpoint: { id: null, modalOpen: false, response: null, error: null, running: false }, websocket: { id: null, socket: null, connected: false, query: '', message: '', logs: [] }, data: { kind: 'database', name: null, table: null, key: null, queue: null, id: null, payload: null, catalog: {} } };
  const base = '/_dowe/dev/server';
  const esc = (value) => String(value ?? '').replace(/[&<>"']/g, (char) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[char]));
  const copy = async (value) => { try { await navigator.clipboard.writeText(value); } catch (_) {} };
  document.addEventListener('keydown', (event) => { if (event.key !== 'Escape') return; if (state.endpoint.modalOpen) { event.preventDefault(); closeEndpointModal(); } else if (state.sourceOpen) { event.preventDefault(); closeSourceModal(); } });

  async function load() {
    try {
      const response = await fetch(`${base}/manifest.json`, { cache: 'no-store' });
      if (!response.ok) throw new Error('Server inspector is only available from dowe dev.');
      state.manifest = await response.json();
      render();
    } catch (error) {
      root.innerHTML = `<main class="main"><div class="card full"><h2>Dowe Server Inspector</h2><div class="error">${esc(error.message)}</div></div></main>`;
    }
  }

  function navIcon(tab) {
    const paths = {
      overview: '<rect x="3" y="3" width="7" height="7" rx="1"/><rect x="14" y="3" width="7" height="7" rx="1"/><rect x="3" y="14" width="7" height="7" rx="1"/><rect x="14" y="14" width="7" height="7" rx="1"/>',
      endpoints: '<path d="M4 7h16M4 12h10M4 17h16"/><circle cx="18" cy="12" r="2"/>',
      websockets: '<path d="M5 7c3-3 5-3 7 0s4 3 7 0M5 17c3-3 5-3 7 0s4 3 7 0"/><path d="M12 4v16"/>',
      flow: '<circle cx="5" cy="12" r="2"/><circle cx="19" cy="6" r="2"/><circle cx="19" cy="18" r="2"/><path d="m7 11 10-4M7 13l10 4"/>',
      resources: '<ellipse cx="12" cy="5" rx="7" ry="3"/><path d="M5 5v7c0 1.7 3.1 3 7 3s7-1.3 7-3V5"/><path d="M5 12v7c0 1.7 3.1 3 7 3s7-1.3 7-3v-7"/>',
      data: '<path d="M5 7h14M5 12h14M5 17h14"/><circle cx="3" cy="7" r="1"/><circle cx="3" cy="12" r="1"/><circle cx="3" cy="17" r="1"/>',
      jobs: '<circle cx="12" cy="12" r="8"/><path d="M12 7v5l3 2"/>'
    };
    return `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">${paths[tab]}</svg>`;
  }

  function actionIcon(action) {
    const paths = {
      source: '<path d="m9 7-5 5 5 5M15 7l5 5-5 5"/>',
      try: '<path d="m9 6 9 6-9 6z"/>',
      close: '<path d="m6 6 12 12M18 6 6 18"/>'
    };
    return `<svg viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="1.9" stroke-linecap="round" stroke-linejoin="round">${paths[action]}</svg>`;
  }

  function nav() {
    const items = [
      ['overview', 'Overview', null],
      ['endpoints', 'Endpoints', state.manifest.routes.length],
      ['websockets', 'WebSockets', state.manifest.websockets.length],
      ['flow', 'Flow', state.manifest.nodes.length],
      ['resources', 'Resources', state.manifest.resources.length],
      ['data', 'Data', null],
      ['jobs', 'Jobs', state.manifest.jobs.length]
    ];
    return items.map(([tab, label, count]) => `<button class="${state.tab === tab ? 'active' : ''}" data-tab="${tab}" aria-current="${state.tab === tab ? 'page' : 'false'}"><span class="nav-icon">${navIcon(tab)}</span><span class="nav-text">${label}</span>${count === null ? '' : `<span class="nav-count">${count}</span>`}</button>`).join('');
  }

  function layout(content) {
    const m = state.manifest;
    return `<div class="shell"><aside class="side"><div class="brand"><div class="brand-name">Dowe</div><span>Server Inspector</span></div><div class="side-divider"></div><div class="nav-section"><div class="nav-heading">Workspace<span>DEV</span></div><div class="nav">${nav()}</div></div><div class="side-footer"><div class="environment"><span class="status-dot"></span><div><strong>Development</strong><span>read-only runtime</span></div></div><div class="backend"><span>Backend</span><strong>:${esc(m.port)}</strong></div></div></aside><main class="main">${content}</main></div>`;
  }

  function card(title, body, extra = '') { return `<section class="card ${extra}"><h2>${title}</h2>${body}</section>`; }
  function list(items, render, empty = 'No items found') { return `<div class="list">${items.length ? items.map(render).join('') : `<div class="empty">${empty}</div>`}</div>`; }
  function sourceButton(item, label) { return item.source ? `<button class="item" data-source="${esc(item.id)}"><strong>${esc(label)}</strong><span class="muted">${esc(item.source.path)}:${item.source.line}-${item.source.end_line}</span></button>` : `<div class="item"><strong>${esc(label)}</strong></div>`; }

  function overview() {
    const m = state.manifest;
    return `<div class="grid">${card('Endpoints', `<div class="metric">${m.routes.length}</div><div class="muted">HTTP entrypoints</div>`)}${card('WebSockets', `<div class="metric">${m.websockets.length}</div><div class="muted">declared channels</div>`)}${card('Resources', `<div class="metric">${m.resources.length}</div><div class="muted">database · cache · vector · queue</div>`)}${card('Entities', `<div class="metric">${m.entities.length}</div><div class="muted">compiled database entities</div>`)}${card('Jobs', `<div class="metric">${m.jobs.length}</div><div class="muted">tasks and crons</div>`)}${card('Services', `<div class="metric">${m.services.filter((service) => service.enabled).length}</div><div class="muted">enabled local service surfaces</div>`)}${card('Flow overview', `<div class="flow">${list(m.routes.slice(0, 12), (route) => `<div class="item" data-source="${esc(route.id)}"><strong><span class="pill">${esc(route.method)}</span>${esc(route.path)}</strong><span class="muted">${esc(route.behavior)}</span></div>`)}</div>`, 'wide')}${card('Dev contract', `<div class="muted">This dashboard is mounted on the backend listener only while <code>dowe dev</code> is running. It is not part of deploy output.</div>`, 'wide')}</div>`;
  }

  function methodClass(method) { return `method-${String(method || '').toLowerCase()}`; }

  function endpointForm(route) {
    const pathFields = (route.parameters || []).filter((parameter) => parameter.location === 'path');
    const queryFields = (route.parameters || []).filter((parameter) => parameter.location === 'query');
    const headerFields = route.headers || [];
    const bodyFields = route.body?.fields || [];
    const bodyField = (field) => {
      const label = `${esc(field.name)}${field.required ? ' *' : ''}`;
      const type = esc(field.field_type);
      let input = `<input data-body-field="${esc(field.name)}" type="${field.field_type === 'number' ? 'number' : 'text'}" ${field.required ? 'required' : ''} placeholder="${type}"/>`;
      if (field.field_type === 'boolean') input = `<select data-body-field="${esc(field.name)}"><option value="">Select…</option><option value="true">true</option><option value="false">false</option></select>`;
      if (field.field_type === 'object' || field.field_type === 'array') input = `<textarea data-body-field="${esc(field.name)}" rows="2" placeholder="JSON value"></textarea>`;
      return `<label class="request-field"><span>${label}<small>${type}</small></span>${input}</label>`;
    };
    const body = route.body ? (bodyFields.length ? `<div class="request-fields">${bodyFields.map(bodyField).join('')}</div>` : `<textarea data-request-body rows="7" placeholder="${route.body.content_type === 'application/octet-stream' ? 'Raw request body' : 'JSON object'}"></textarea>`) : '';
    return `<div class="endpoint-form"><div class="request-section"><div class="request-section-title">Path</div>${pathFields.length ? pathFields.map((field) => `<label class="request-field"><span>${esc(field.name)}<small>${esc(field.field_type)} · required</small></span><input data-path-param="${esc(field.name)}" required placeholder="${esc(field.name)}"/></label>`).join('') : '<div class="muted">No path parameters.</div>'}</div><div class="request-section"><div class="request-section-title">Search params${queryFields.length ? ` <span>${esc(queryFields.map((field) => field.name).join(', '))}</span>` : ''}</div>${queryFields.length ? `<input data-request-query placeholder="page=1&search=term"/>` : '<div class="muted">No query parameters declared.</div>'}</div><div class="request-section"><div class="request-section-title">Headers</div>${headerFields.length ? headerFields.map((header) => `<label class="request-field"><span>${esc(header.name)}${header.required ? ' *' : ''}<small>${header.sensitive ? 'sensitive' : 'request header'}</small></span><input data-request-header="${esc(header.name)}" type="${header.sensitive ? 'password' : 'text'}" placeholder="${header.required ? 'Required' : 'Optional'}"/></label>`).join('') : '<div class="muted">No custom headers declared.</div>'}</div>${route.body ? `<div class="request-section"><div class="request-section-title">Body <span>${esc(route.body.content_type)}</span></div>${body}</div>` : ''}<div class="endpoint-actions"><button class="execute-button" data-endpoint-execute="${esc(route.id)}" ${state.endpoint.running ? 'disabled' : ''}>${state.endpoint.running ? 'Running…' : 'Try it'}</button><button class="copy" data-copy-endpoint="${esc(route.id)}">Copy request</button><button class="copy" data-source="${esc(route.id)}">View source</button></div>${state.endpoint.error ? `<div class="error">${esc(state.endpoint.error)}</div>` : ''}${state.endpoint.response ? responsePanel(state.endpoint.response) : ''}</div>`;
  }

  function responsePanel(response) {
    return `<div class="response-panel"><div class="response-head"><strong>Response</strong><span class="status-badge ${response.status >= 400 ? 'status-error' : 'status-ok'}">${esc(response.status)} ${esc(response.statusText || '')}</span></div><div class="response-meta">${Object.entries(response.headers || {}).map(([name, value]) => `<span><b>${esc(name)}</b>: ${esc(value)}</span>`).join('')}</div><pre class="source response-body">${esc(response.body || '')}</pre></div>`;
  }

  function endpointModal(route) {
    if (!state.endpoint.modalOpen || !route) return '';
    return `<div class="modal-backdrop" data-endpoint-modal-backdrop><section class="modal-surface endpoint-modal" role="dialog" aria-modal="true" aria-labelledby="endpoint-modal-title"><div class="modal-header"><div><div class="eyebrow">Try endpoint</div><h2 id="endpoint-modal-title">${esc(route.method)} ${esc(route.path)}</h2><div class="modal-subtitle">${esc(route.behavior)}</div></div><div class="modal-header-actions"><span class="method-tag ${methodClass(route.method)}">${esc(route.method)}</span><button type="button" class="modal-close" data-endpoint-modal-close aria-label="Close Try endpoint" title="Close">${actionIcon('close')}</button></div></div>${endpointForm(route)}</section></div>`;
  }

  function sourceModal() {
    if (!state.sourceOpen || !state.selected) return '';
    return `<div class="modal-backdrop" data-source-modal-backdrop><section class="modal-surface source-modal" role="dialog" aria-modal="true" aria-labelledby="source-modal-title"><div class="modal-header"><div><div class="eyebrow">Source selection</div><h2 id="source-modal-title">Compiled source</h2><div class="modal-subtitle">The selected source span is bounded to the current manifest node.</div></div><button type="button" class="modal-close" data-source-modal-close aria-label="Close source viewer" title="Close">${actionIcon('close')}</button></div><div class="source" id="source-view">Loading source…</div></section></div>`;
  }

  function endpoints() {
    const m = state.manifest;
    const route = m.routes.find((item) => item.id === state.endpoint.id) || m.routes[0];
    if (route && state.endpoint.id !== route.id) { state.endpoint.id = route.id; state.endpoint.response = null; state.endpoint.error = null; }
    return `<div class="endpoint-page"><section class="endpoint-catalog"><div class="card-head"><h2>Endpoints</h2><span class="nav-count">${m.routes.length}</span></div><div class="endpoint-list">${list(m.routes, (item) => `<div class="endpoint-row-shell ${route?.id === item.id ? 'active' : ''}"><button type="button" class="endpoint-select" data-endpoint-select="${esc(item.id)}" aria-pressed="${route?.id === item.id ? 'true' : 'false'}"><span class="method-tag ${methodClass(item.method)}">${esc(item.method)}</span><span><strong>${esc(item.path)}</strong><small>${esc(item.behavior)}</small></span></button><div class="endpoint-row-actions">${item.source ? `<button type="button" class="icon-action" data-endpoint-source="${esc(item.id)}" aria-label="View source for ${esc(item.method)} ${esc(item.path)}" title="View source">${actionIcon('source')}</button>` : ''}<button type="button" class="icon-action try-action" data-endpoint-try="${esc(item.id)}" aria-label="Try ${esc(item.method)} ${esc(item.path)}" title="Try endpoint">${actionIcon('try')}</button></div></div>`, 'No HTTP endpoints declared.')}</div></section>${endpointModal(route)}${sourceModal()}</div>`;
  }

  function websockets() {
    const m = state.manifest;
    const socket = m.websockets.find((item) => item.id === state.websocket.id) || m.websockets[0];
    if (socket && state.websocket.id !== socket.id) { state.websocket.id = socket.id; state.websocket.logs = []; state.websocket.connected = false; state.websocket.query = ''; state.websocket.message = ''; }
    const logs = state.websocket.logs.map((entry) => `<div class="ws-log ws-${esc(entry.kind)}"><span>${esc(entry.time)}</span><strong>${esc(entry.kind)}</strong><pre>${esc(entry.text)}</pre></div>`).join('') || '<div class="empty">Connect to see WebSocket events.</div>';
    return `<div class="endpoint-layout"><section class="card endpoint-catalog"><div class="card-head"><h2>WebSockets</h2><span class="nav-count">${m.websockets.length}</span></div><div class="endpoint-list">${list(m.websockets, (item) => `<button class="endpoint-row ${socket?.id === item.id ? 'active' : ''}" data-websocket-select="${esc(item.id)}"><span class="method-tag method-ws">WS</span><span><strong>${esc(item.path)}</strong><small>${esc(item.message_format || 'text')} messages</small></span><span class="endpoint-arrow">›</span></button>`, 'No WebSockets declared.')}</div></section><section class="card endpoint-detail">${socket ? `<div class="endpoint-detail-head"><div><div class="eyebrow">WebSocket channel</div><h2>${esc(socket.path)}</h2><div class="muted">${socket.middleware?.length ? `Middleware: ${socket.middleware.map(esc).join(', ')}` : 'No middleware declared'} · ${esc(socket.message_format || 'text')} messages</div></div><span class="method-tag method-ws">WS</span></div><div class="ws-controls"><input data-ws-query value="${esc(state.websocket.query)}" placeholder="Query string for middleware, e.g. token=…"/><textarea data-ws-message rows="3" placeholder="Message to send">${esc(state.websocket.message)}</textarea><div class="endpoint-actions"><button class="execute-button" data-ws-connect ${state.websocket.connected ? 'data-connected="true"' : ''}>${state.websocket.connected ? 'Disconnect' : 'Connect'}</button><button class="execute-button" data-ws-send ${state.websocket.connected ? '' : 'disabled'}>Send</button><button class="copy" data-copy-websocket="${esc(socket.id)}">Copy channel</button></div></div><div class="ws-log-list">${logs}</div>` : '<div class="empty">Select a WebSocket to test it.</div>'}</section></div>`;
  }

  function flow() {
    const m = state.manifest;
    return `<div class="grid">${card('Compiled graph', list(m.nodes, (node) => `<button class="item" data-source="${esc(node.id)}"><strong><span class="pill">${esc(node.kind)}</span>${esc(node.label)}</strong><span class="muted">${node.source ? `${esc(node.source.path)}:${node.source.line}-${node.source.end_line}` : 'generated/runtime node'}</span></button>`), 'wide')}${card('Edges', list(m.edges, (edge) => `<div class="item"><strong>${esc(edge.relation)}</strong><span class="muted">${esc(edge.from)} → ${esc(edge.to)}</span></div>`), '')}</div>`;
  }

  function resources() {
    const m = state.manifest;
    return `<div class="grid">${card('Resources', list(m.resources, (resource) => `<div class="item"><strong><span class="pill">${esc(resource.kind)}</span>${esc(resource.binding)}</strong><span class="muted">${esc(resource.provider)} · ${resource.operations.map(esc).join(', ') || 'no operations'}</span></div>`), 'wide')}${card('Entities', list(m.entities, (entity) => `<div class="item"><strong>${esc(entity.binding)}.${esc(entity.table)}</strong><span class="muted">${esc(entity.provider)} · ${entity.fields.map(esc).join(', ')}</span></div>`), '')}${card('Services', list(m.services, (service) => `<div class="item"><strong>${service.enabled ? '●' : '○'} ${esc(service.kind)}</strong><span class="muted">${esc(service.endpoint)}</span></div>`), 'full')}</div>`;
  }

  const dataProviders = [
    ['database', 'Database', 'Tables and records'],
    ['cache', 'Cache', 'Keys and values'],
    ['queue', 'Queue', 'Queues and messages'],
    ['vector', 'Vector', 'Embeddings and metadata']
  ];

  function resetDataSelection(kind = state.data.kind) {
    state.data = { kind, name: null, table: null, key: null, queue: null, id: null, payload: null, catalog: state.data.catalog || {} };
  }

  function dataParams(selection) {
    const params = new URLSearchParams();
    ['name', 'table', 'key', 'queue', 'id'].forEach((field) => { if (selection[field]) params.set(field, selection[field]); });
    params.set('limit', '100');
    return params.toString();
  }

  async function loadData(selection) {
    const response = await fetch(`${base}/data/${selection.kind}?${dataParams(selection)}`, { cache: 'no-store' });
    if (!response.ok) throw new Error('Data is unavailable');
    selection.payload = await response.json();
    if (!selection.name && !selection.table && !selection.key && !selection.queue && !selection.id) {
      selection.catalog[selection.kind] = selection.payload;
    }
  }

  function dataConnectionSummary(kind, item) {
    if (kind === 'database') return `${item.tables?.length || 0} tables · ${item.tables?.reduce((total, table) => total + (table.records || 0), 0) || 0} records`;
    if (kind === 'cache') return `${item.keys?.length || 0} keys · ${item.persistent ? 'persistent' : 'memory'}`;
    if (kind === 'vector') return `${item.embeddings || 0} embeddings · ${item.dimensions || 'dynamic'} dimensions`;
    return `${item.queues?.length || 0} queues · ${item.queues?.reduce((total, queue) => total + (queue.ready || 0), 0) || 0} ready`;
  }

  function formatDataCell(value) {
    if (value === undefined) return '—';
    if (value === null) return 'null';
    return typeof value === 'object' ? JSON.stringify(value) : String(value);
  }

  function dataRowsTable(rows, columns, options = {}) {
    const safeRows = Array.isArray(rows) ? rows : [];
    const safeColumns = columns?.length ? columns : [...new Set(safeRows.flatMap((row) => Object.keys(row || {})))];
    if (!safeRows.length) return '<div class="empty data-empty">No records in this view.</div>';
    return `<div class="data-table-wrap"><table class="data-table"><thead><tr>${safeColumns.map((column) => `<th>${esc(column)}</th>`).join('')}</tr></thead><tbody>${safeRows.map((row) => `<tr ${options.idField && row[options.idField] ? `data-data-id="${esc(row[options.idField])}"` : ''}>${safeColumns.map((column) => `<td title="${esc(formatDataCell(row?.[column]))}">${esc(formatDataCell(row?.[column]))}</td>`).join('')}</tr>`).join('')}</tbody></table></div>`;
  }

  function entityStructure(databaseName, tableName = null) {
    const entities = (state.manifest.entities || []).filter((entity) => (entity.database === databaseName || entity.binding === databaseName) && (!tableName || entity.table === tableName));
    if (!entities.length) return '';
    return `<div class="entity-structure"><div class="entity-structure-head"><span>Entity structure</span><small>${entities.length} associated</small></div><div class="entity-cards">${entities.map((entity) => {
      const fields = entity.field_details?.length ? entity.field_details : (entity.fields || []).map((name) => ({ name, field_type: 'unknown' }));
      return `<article class="entity-card"><div class="entity-card-head"><div><strong>${esc(entity.binding)}.${esc(entity.table)}</strong><small>${esc(entity.provider)} · ${fields.length} fields</small></div><span class="entity-badge">Entity</span></div><div class="entity-fields">${fields.map((field) => `<div class="entity-field"><code>${esc(field.name)}</code><span class="entity-type">${esc(field.field_type || 'unknown')}</span><span class="entity-flags">${[field.primary && 'PK', field.required && 'required', field.unique && 'unique', field.index && 'index'].filter(Boolean).join(' · ')}</span></div>`).join('')}</div></article>`;
    }).join('')}</div></div>`;
  }

  function dataWorkspace(selection) {
    const payload = selection.payload || {};
    if (payload.error) return `<div class="error">${esc(payload.error)}</div>`;
    if (!selection.name) return '<div class="data-welcome"><div class="data-welcome-icon">⌘</div><h3>Select a connection</h3><p>Choose a database, cache, queue or vector space to browse its read-only development data.</p></div>';
    if (selection.kind === 'database') {
      const item = payload.item || {};
      if (selection.table) return `<div class="data-workspace-head"><button class="data-back" data-data-clear="table">← ${esc(selection.name)}</button><span class="data-path">Table / ${esc(selection.table)}</span></div>${entityStructure(selection.name, selection.table)}${dataRowsTable(payload.rows, payload.columns)}<div class="data-footnote">${payload.total || 0} total records · showing up to ${payload.limit || 100}</div>`;
      return `<div class="data-workspace-head"><button class="data-back" data-data-clear="name">← Connections</button><span class="data-path">Database / ${esc(selection.name)}</span><span class="data-count">${item.tables?.length || 0} tables</span></div>${entityStructure(selection.name)}<div class="schema-grid">${(item.tables || []).map((table) => `<button class="schema-card" data-data-table="${esc(table.name)}"><span class="schema-icon">▦</span><span><strong>${esc(table.name)}</strong><small>${table.records || 0} records · ${table.indexes?.length || 0} indexes</small></span><span class="schema-arrow">→</span></button>`).join('') || '<div class="empty data-empty">No tables found.</div>'}</div>`;
    }
    if (selection.kind === 'cache') {
      const item = payload.item || {};
      if (selection.key) return `<div class="data-workspace-head"><button class="data-back" data-data-clear="key">← ${esc(selection.name)}</button><span class="data-path">Key / ${esc(selection.key)}</span></div><div class="value-view"><div class="value-label">Value</div><pre class="source">${esc(JSON.stringify(payload.value, null, 2) ?? 'null')}</pre></div>`;
      return `<div class="data-workspace-head"><button class="data-back" data-data-clear="name">← Connections</button><span class="data-path">Cache / ${esc(selection.name)}</span><span class="data-count">${item.keys?.length || 0} keys</span></div><div class="key-list">${(item.keys || []).map((key) => key === '[redacted]' ? `<div class="key-row masked"><span class="key-icon">⌁</span><code>${esc(key)}</code><span>•</span></div>` : `<button class="key-row" data-data-key="${esc(key)}"><span class="key-icon">⌁</span><code>${esc(key)}</code><span>→</span></button>`).join('') || '<div class="empty data-empty">No keys found.</div>'}</div>`;
    }
    if (selection.kind === 'vector') {
      if (selection.id) return `<div class="data-workspace-head"><button class="data-back" data-data-clear="id">← ${esc(selection.name)}</button><span class="data-path">Embedding / ${esc(selection.id)}</span></div><pre class="source">${esc(JSON.stringify(payload.item, null, 2))}</pre>`;
      return `<div class="data-workspace-head"><button class="data-back" data-data-clear="name">← Connections</button><span class="data-path">Vector / ${esc(selection.name)}</span><span class="data-count">${payload.item?.embeddings || 0} embeddings</span></div>${dataRowsTable(payload.rows, ['id', 'dimensions', 'metadata'], { idField: 'id' })}`;
    }
    const item = payload.item || {};
    if (selection.queue) return `<div class="data-workspace-head"><button class="data-back" data-data-clear="queue">← ${esc(selection.name)}</button><span class="data-path">Queue / ${esc(selection.queue)}</span></div>${dataRowsTable(payload.rows, ['id', 'topic', 'value', 'publishedAt', 'redelivered'])}<div class="data-footnote">Read-only peek · showing up to ${payload.limit || 100} messages</div>`;
    return `<div class="data-workspace-head"><button class="data-back" data-data-clear="name">← Connections</button><span class="data-path">Queue / ${esc(selection.name)}</span><span class="data-count">${item.queues?.length || 0} queues</span></div><div class="schema-grid">${(item.queues || []).map((queue) => `<button class="schema-card" data-data-queue="${esc(queue.queue)}"><span class="schema-icon">≋</span><span><strong>${esc(queue.queue)}</strong><small>${queue.ready || 0} ready · ${queue.inFlight || 0} in flight</small></span><span class="schema-arrow">→</span></button>`).join('') || '<div class="empty data-empty">No queues found.</div>'}</div>`;
  }

  async function data() {
    const selection = state.data;
    if (!selection.payload) {
      try { await loadData(selection); } catch (error) { selection.payload = { kind: selection.kind, error: error.message }; }
    }
    const catalog = selection.catalog[selection.kind] || { items: [] };
    const tabs = dataProviders.map(([kind, label, description]) => `<button class="provider-tab ${selection.kind === kind ? 'active' : ''}" data-data-kind="${kind}" title="${esc(description)}" aria-label="${esc(`${label}: ${description}`)}"><strong>${label}</strong></button>`).join('');
    const connectionOptions = (catalog.items || []).map((item) => `<option value="${esc(item.name)}" ${selection.name === item.name ? 'selected' : ''}>${esc(item.name)} · ${esc(dataConnectionSummary(selection.kind, item))}</option>`).join('');
    return `<div class="data-studio"><div class="data-studio-toolbar"><div><div class="eyebrow">Runtime data</div><div class="data-studio-title">Data studio</div></div><span class="read-only-icon" role="img" aria-label="Read-only bounded data" title="Read-only · bounded">✓</span></div><div class="provider-tabs">${tabs}</div><div class="data-controls"><label class="data-select"><span class="sr-only">Connection</span><select aria-label="Connection" data-data-name-select><option value="">Select a connection…</option>${connectionOptions}</select></label><button class="data-refresh" data-data-refresh type="button" aria-label="Refresh data" title="Refresh data">↻</button></div><section class="card data-workspace">${dataWorkspace(selection)}</section></div>`;
  }

  function jobs() {
    const m = state.manifest;
    return `<div class="grid">${card('Tasks and crons', list(m.jobs, (job) => sourceButton(job, `${job.kind} · ${job.target || job.id}${job.schedule ? ` · ${job.schedule}` : ''}`)), 'wide')}${card('Runtime events', '<div class="muted">Build and reload events are available from the existing <code>/_dowe/dev/ws</code> channel.</div>')}</div>`;
  }

  function selectedRoute(id) { return state.manifest.routes.find((route) => route.id === id); }

  function endpointPath(route, target) {
    let path = route.path;
    target.querySelectorAll('[data-path-param]').forEach((input) => { path = path.replace(`:${input.dataset.pathParam}`, encodeURIComponent(input.value)); path = path.replace(`*${input.dataset.pathParam}`, input.value.split('/').map(encodeURIComponent).join('/')); });
    return path;
  }

  function endpointRequest(route, target) {
    const query = {};
    const queryText = target.querySelector('[data-request-query]')?.value || '';
    new URLSearchParams(queryText).forEach((value, key) => { query[key] = value; });
    const headers = {};
    target.querySelectorAll('[data-request-header]').forEach((input) => { if (input.value) headers[input.dataset.requestHeader] = input.value; });
    let body;
    if (route.body) {
      const fields = target.querySelectorAll('[data-body-field]');
      if (fields.length) {
        body = {};
        fields.forEach((input) => { if (!input.value) return; if (input.tagName === 'SELECT') body[input.dataset.bodyField] = input.value === 'true' ? true : input.value === 'false' ? false : input.value; else if (['object', 'array'].includes((route.body.fields.find((field) => field.name === input.dataset.bodyField) || {}).field_type)) { try { body[input.dataset.bodyField] = JSON.parse(input.value); } catch (_) { body[input.dataset.bodyField] = input.value; } } else if ((route.body.fields.find((field) => field.name === input.dataset.bodyField) || {}).field_type === 'number') body[input.dataset.bodyField] = Number(input.value); else body[input.dataset.bodyField] = input.value; });
      } else {
        const raw = target.querySelector('[data-request-body]')?.value || '';
        if (raw) { try { body = route.body.content_type === 'application/json' ? JSON.parse(raw) : raw; } catch (_) { throw new Error('Body must be valid JSON'); } }
      }
      if (!Object.keys(headers).some((name) => name.toLowerCase() === 'content-type')) headers['Content-Type'] = route.body.content_type;
    }
    return { id: route.id, method: route.method, path: endpointPath(route, target), query, headers, body };
  }

  async function executeEndpoint(route, target) {
    let request;
    try { request = endpointRequest(route, target); } catch (error) { state.endpoint.error = error.message; await render(); return; }
    state.endpoint.running = true; state.endpoint.error = null; state.endpoint.response = null; await render();
    try {
      const response = await fetch(`${base}/execute`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify(request) });
      const payload = await response.json().catch(() => ({ error: 'Invalid inspector response' }));
      if (!response.ok) throw new Error(payload.error || `Request failed (${response.status})`);
      state.endpoint.response = payload;
    } catch (error) { state.endpoint.error = error.message; }
    state.endpoint.running = false; await render();
  }

  function websocketLog(kind, text) { state.websocket.logs.push({ kind, text: typeof text === 'string' ? text : JSON.stringify(text), time: new Date().toLocaleTimeString() }); state.websocket.logs = state.websocket.logs.slice(-100); }

  function disconnectWebsocket() { if (state.websocket.socket) state.websocket.socket.close(); state.websocket.socket = null; state.websocket.connected = false; }

  function connectWebsocket(socket) {
    disconnectWebsocket();
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const query = root.querySelector('[data-ws-query]')?.value || state.websocket.query;
    state.websocket.query = query;
    const suffix = query ? `?${query.replace(/^\?/, '')}` : '';
    const websocket = new WebSocket(`${protocol}//${window.location.host}${socket.path}${suffix}`);
    state.websocket.socket = websocket;
    websocket.onopen = () => { state.websocket.connected = true; websocketLog('open', 'Connected'); render(); };
    websocket.onmessage = (event) => { websocketLog('message', event.data); render(); };
    websocket.onerror = () => { websocketLog('error', 'WebSocket error'); render(); };
    websocket.onclose = (event) => { state.websocket.connected = false; websocketLog('close', `${event.code} ${event.reason || ''}`.trim()); state.websocket.socket = null; render(); };
    render();
  }

  function closeEndpointModal() { state.endpoint.modalOpen = false; render(); }
  function closeSourceModal() { state.sourceOpen = false; render(); }

  async function openEndpointModal(id) {
    state.endpoint.id = id;
    state.endpoint.modalOpen = true;
    state.endpoint.response = null;
    state.endpoint.error = null;
    await render();
    root.querySelector('[data-endpoint-modal-close]')?.focus();
  }

  async function render() {
    if (!state.manifest) return;
    let content = state.tab === 'endpoints' ? endpoints() : state.tab === 'websockets' ? websockets() : state.tab === 'flow' ? flow() : state.tab === 'resources' ? resources() : state.tab === 'data' ? await data() : state.tab === 'jobs' ? jobs() : overview();
    root.innerHTML = layout(content);
    document.body.classList.toggle('modal-open', state.endpoint.modalOpen || state.sourceOpen);
    root.querySelectorAll('[data-tab]').forEach((button) => button.onclick = () => { state.tab = button.dataset.tab; state.selected = null; state.sourceOpen = false; state.endpoint.modalOpen = false; if (state.tab === 'data' && !state.data.payload) resetDataSelection(); render(); });
    root.querySelectorAll('[data-copy-endpoint]').forEach((button) => button.onclick = () => { const route = selectedRoute(button.dataset.copyEndpoint); if (route) copy(JSON.stringify({ route, request: endpointRequest(route, document.querySelector('.endpoint-form')) }, null, 2)); });
    root.querySelectorAll('[data-copy-websocket]').forEach((button) => button.onclick = () => { const socket = state.manifest.websockets.find((item) => item.id === button.dataset.copyWebsocket); if (socket) copy(JSON.stringify(socket, null, 2)); });
    root.querySelectorAll('[data-source]').forEach((button) => button.addEventListener('click', () => selectSource(button.dataset.source)));
    root.querySelectorAll('[data-endpoint-select]').forEach((button) => button.addEventListener('click', () => { state.endpoint.id = button.dataset.endpointSelect; state.endpoint.response = null; state.endpoint.error = null; render(); }));
    root.querySelectorAll('[data-endpoint-source]').forEach((button) => button.addEventListener('click', () => selectSource(button.dataset.endpointSource)));
    root.querySelectorAll('[data-endpoint-try]').forEach((button) => button.addEventListener('click', () => openEndpointModal(button.dataset.endpointTry)));
    root.querySelector('[data-endpoint-modal-close]')?.addEventListener('click', closeEndpointModal);
    root.querySelector('[data-source-modal-close]')?.addEventListener('click', closeSourceModal);
    root.querySelector('[data-endpoint-modal-backdrop]')?.addEventListener('click', (event) => { if (event.target === event.currentTarget) closeEndpointModal(); });
    root.querySelector('[data-source-modal-backdrop]')?.addEventListener('click', (event) => { if (event.target === event.currentTarget) closeSourceModal(); });
    root.querySelector('[data-endpoint-execute]')?.addEventListener('click', (event) => { const route = selectedRoute(event.currentTarget.dataset.endpointExecute); if (route) executeEndpoint(route, document.querySelector('.endpoint-form')); });
    root.querySelectorAll('[data-websocket-select]').forEach((button) => button.addEventListener('click', () => { disconnectWebsocket(); state.websocket.id = button.dataset.websocketSelect; state.websocket.logs = []; render(); }));
    root.querySelector('[data-ws-message]')?.addEventListener('input', (event) => { state.websocket.message = event.target.value; });
    root.querySelector('[data-ws-query]')?.addEventListener('input', (event) => { state.websocket.query = event.target.value; });
    root.querySelector('[data-ws-connect]')?.addEventListener('click', () => { const socket = state.manifest.websockets.find((item) => item.id === state.websocket.id); if (!socket) return; if (state.websocket.connected) disconnectWebsocket(); else connectWebsocket(socket); render(); });
    root.querySelector('[data-ws-send]')?.addEventListener('click', () => { const input = root.querySelector('[data-ws-message]'); if (!input || !state.websocket.socket || !state.websocket.connected) return; state.websocket.socket.send(input.value); websocketLog('sent', input.value); input.value = ''; state.websocket.message = ''; render(); });
    root.querySelector('[data-ws-connect]')?.addEventListener('dblclick', (event) => event.preventDefault());
    root.querySelector('[data-ws-message]')?.addEventListener('keydown', (event) => { if (event.key === 'Enter' && (event.metaKey || event.ctrlKey)) { event.preventDefault(); if (state.websocket.socket && state.websocket.connected) { state.websocket.socket.send(event.target.value); websocketLog('sent', event.target.value); event.target.value = ''; state.websocket.message = ''; render(); } } });
    root.querySelectorAll('[data-data-kind]').forEach((button) => button.addEventListener('click', () => { resetDataSelection(button.dataset.dataKind); render(); }));
    root.querySelector('[data-data-name-select]')?.addEventListener('change', (event) => { state.data.name = event.target.value || null; state.data.table = null; state.data.key = null; state.data.queue = null; state.data.id = null; state.data.payload = null; render(); });
    root.querySelector('[data-data-refresh]')?.addEventListener('click', () => { state.data.payload = null; render(); });
    root.querySelectorAll('[data-data-table]').forEach((button) => button.addEventListener('click', () => { state.data.table = button.dataset.dataTable; state.data.payload = null; render(); }));
    root.querySelectorAll('[data-data-key]').forEach((button) => button.addEventListener('click', () => { state.data.key = button.dataset.dataKey; state.data.payload = null; render(); }));
    root.querySelectorAll('[data-data-queue]').forEach((button) => button.addEventListener('click', () => { state.data.queue = button.dataset.dataQueue; state.data.payload = null; render(); }));
    root.querySelectorAll('[data-data-id]').forEach((button) => button.addEventListener('click', () => { state.data.id = button.dataset.dataId; state.data.payload = null; render(); }));
    root.querySelectorAll('[data-data-clear]').forEach((button) => button.addEventListener('click', () => { const level = button.dataset.dataClear; if (level === 'name') { state.data.name = null; state.data.table = null; state.data.key = null; state.data.queue = null; state.data.id = null; } if (level === 'table') state.data.table = null; if (level === 'key') state.data.key = null; if (level === 'queue') state.data.queue = null; if (level === 'id') state.data.id = null; state.data.payload = null; render(); }));
    if (state.selected && (state.tab === 'endpoints' || state.tab === 'flow' || state.tab === 'jobs')) await selectSource(state.selected, true);
  }

  async function selectSource(id, preserve = false) {
    state.selected = id;
    state.sourceOpen = true;
    fetch(`${base}/selection`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ id }) }).catch(() => {});
    if (!preserve) await render();
    if (!preserve) root.querySelector('[data-source-modal-close]')?.focus();
    const target = document.getElementById('source-view');
    if (!target) return;
    try {
      const response = await fetch(`${base}/source/${encodeURIComponent(id)}`, { cache: 'no-store' });
      const source = await response.json();
      target.textContent = `${source.path}:${source.startLine}-${source.endLine}\n\n${source.code}`;
    } catch (_) { target.textContent = 'Source unavailable'; }
  }

  load();
})();
