fn server_inspector_html() -> String {
    let script = include_str!("../server_inspector_client.js");
    format!(
        r#"<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Dowe Server Inspector</title><style>{}</style></head><body><div id="dowe-server-inspector"></div><script>{}</script></body></html>"#,
        SERVER_INSPECTOR_CSS, script
    )
}

const SERVER_INSPECTOR_CSS: &str = r#"
:root {
  color-scheme: light;
  font-family: Manrope, Inter, ui-sans-serif, system-ui, sans-serif;
  background: #fff;
  color: #17263a;
  --dowe-primary: #1f3a5f;
  --dowe-primary-text: #fff;
  --dowe-secondary: #6bc670;
  --dowe-secondary-text: #102a15;
  --dowe-background: #fff;
  --dowe-background-text: #17263a;
  --dowe-background-title: #17263e;
  --dowe-surface: #f7f9fc;
  --dowe-muted: #526274;
  --dowe-soft-primary: #ccfbf3;
  --dowe-soft-secondary: #e3f5e5;
  --dowe-soft-muted: #e6ebf0;
  --dowe-soft-danger: #f7e3e6;
  --dowe-line: #dce4ec;
  --dowe-shadow: 0 10px 28px rgba(23, 38, 58, .09);
  --dowe-nav-active: #56687a;
  --dowe-nav-active-text: #fff;
  --dowe-nav-hover: #f2f5f7;
}
* { box-sizing: border-box; scrollbar-width: thin; scrollbar-color: #b8c5d2 transparent; }
*::-webkit-scrollbar { width: 10px; height: 10px; }
*::-webkit-scrollbar-track { background: transparent; }
*::-webkit-scrollbar-thumb { min-height: 36px; border: 3px solid transparent; border-radius: 999px; background: #b8c5d2; background-clip: content-box; }
*::-webkit-scrollbar-thumb:hover { border-width: 2px; background: var(--dowe-primary); background-clip: padding-box; }
*::-webkit-scrollbar-corner { background: transparent; }
html, body { min-height: 100%; }
body { margin: 0; min-height: 100vh; overflow: hidden; background: var(--dowe-background); color: var(--dowe-background-text); font-size: 14px; }
button { font: inherit; }
.shell { min-height: 100vh; }
.side { position: fixed; inset: 0 auto 0 0; z-index: 10; display: flex; flex-direction: column; width: 260px; height: 100vh; overflow-y: auto; padding: 28px 18px 20px; border-right: 1px solid var(--dowe-line); background: var(--dowe-background); }
.brand { margin: 0; padding: 0 2px 18px; color: var(--dowe-primary); }
.brand-name { font-size: 21px; font-weight: 850; letter-spacing: -.035em; line-height: 1.1; }
.brand span { display: block; margin-top: 5px; color: var(--dowe-muted); font-size: 12px; font-weight: 750; letter-spacing: .025em; }
.side-divider { height: 1px; background: var(--dowe-line); margin: 0 2px 22px; }
.nav-section { display: grid; gap: 10px; }
.nav-heading { display: flex; align-items: center; justify-content: space-between; padding: 0 4px; color: var(--dowe-muted); font-size: 10px; font-weight: 850; letter-spacing: .11em; text-transform: uppercase; }
.nav-heading span { color: var(--dowe-primary); font-size: 9px; letter-spacing: .08em; }
.nav { display: grid; gap: 6px; }
.nav button, .copy { border: 1px solid var(--dowe-line); background: var(--dowe-background); color: var(--dowe-primary); border-radius: 9px; padding: 9px 11px; text-align: left; cursor: pointer; font-weight: 700; transition: background .15s ease, border-color .15s ease, box-shadow .15s ease, transform .15s ease; }
.nav button { display: flex; align-items: center; gap: 10px; min-height: 43px; border-color: transparent; background: transparent; color: var(--dowe-muted); border-radius: 8px; padding: 8px 10px; font-weight: 650; }
.nav button:hover, .copy:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.nav button:hover { border-color: transparent; background: var(--dowe-nav-hover); color: var(--dowe-primary); transform: none; }
.nav button:focus-visible, .copy:focus-visible, .item:focus-visible { outline: 3px solid var(--dowe-soft-primary); outline-offset: 2px; }
.nav button.active { background: var(--dowe-nav-active); color: var(--dowe-nav-active-text); border-color: var(--dowe-nav-active); box-shadow: none; }
.nav-icon { display: grid; flex: 0 0 25px; place-items: center; width: 25px; height: 25px; border-radius: 7px; background: transparent; color: currentColor; }
.nav-icon svg { width: 15px; height: 15px; }
.nav button.active .nav-icon { background: transparent; color: inherit; }
.nav-text { flex: 1; }
.nav-count { min-width: 25px; padding: 3px 6px; border-radius: 999px; background: var(--dowe-soft-muted); color: var(--dowe-muted); font-size: 11px; line-height: 1; text-align: center; }
.nav button.active .nav-count { background: rgba(255, 255, 255, .16); color: var(--dowe-nav-active-text); }
.side-footer { display: grid; gap: 12px; margin-top: auto; padding-top: 20px; }
.environment { display: flex; align-items: center; gap: 10px; padding: 11px 12px; border: 1px solid var(--dowe-line); border-radius: 10px; background: rgba(255, 255, 255, .82); }
.status-dot { width: 9px; height: 9px; flex: 0 0 9px; border-radius: 50%; background: var(--dowe-secondary); box-shadow: 0 0 0 4px var(--dowe-soft-secondary); }
.environment strong, .environment span { display: block; }
.environment strong { color: var(--dowe-primary); font-size: 12px; }
.environment span { margin-top: 2px; color: var(--dowe-muted); font-size: 11px; }
.backend { display: flex; align-items: center; justify-content: space-between; padding: 0 3px; color: var(--dowe-muted); font-size: 11px; }
.backend strong { color: var(--dowe-primary); font-weight: 800; }
.main { width: calc(100% - 260px); max-width: none; height: 100vh; min-height: 100vh; margin-left: 260px; overflow: auto; padding: 32px 38px; }
.eyebrow { color: var(--dowe-primary); font-size: 12px; font-weight: 800; text-transform: uppercase; letter-spacing: .08em; }
.muted { color: var(--dowe-muted); }
.grid { display: grid; grid-template-columns: repeat(12, minmax(0, 1fr)); gap: 16px; }
.card { grid-column: span 4; min-width: 0; padding: 18px; border: 1px solid var(--dowe-line); border-radius: 12px; background: var(--dowe-surface); box-shadow: var(--dowe-shadow); }
.card.wide { grid-column: span 8; }
.card.full { grid-column: 1 / -1; }
.card h2 { margin: 0 0 12px; color: var(--dowe-background-title); font-size: 15px; }
.metric { color: var(--dowe-primary); font-size: 30px; font-weight: 800; }
.list { display: grid; gap: 8px; max-height: 430px; overflow: auto; }
.data-studio { display: grid; gap: 16px; }
.data-studio-toolbar { display: flex; align-items: center; justify-content: space-between; gap: 18px; }
.data-studio-title { margin: 4px 0 5px; color: var(--dowe-background-title); font-size: 30px; line-height: 1.1; letter-spacing: -.035em; }
.read-only-icon { display: grid; place-items: center; width: 25px; height: 25px; border: 1px solid #b9ddbd; border-radius: 50%; background: var(--dowe-soft-secondary); color: #245d33; font-size: 13px; font-weight: 900; cursor: help; }
.provider-tabs { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 8px; }
.provider-tab { display: flex; align-items: center; min-height: 42px; padding: 10px 13px; border: 1px solid var(--dowe-line); border-radius: 9px; background: var(--dowe-background); color: var(--dowe-primary); cursor: pointer; text-align: left; transition: background .15s ease, border-color .15s ease, transform .15s ease; }
.provider-tab:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); transform: translateY(-1px); }
.provider-tab.active { border-color: var(--dowe-primary); background: var(--dowe-primary); color: var(--dowe-primary-text); }
.provider-tab strong { font-size: 13px; }
.data-controls { display: flex; align-items: center; gap: 8px; padding: 8px; border: 1px solid var(--dowe-line); border-radius: 10px; background: var(--dowe-surface); }
.data-select { display: grid; flex: 1; min-width: 0; }
.data-select select { width: 100%; min-height: 38px; padding: 8px 32px 8px 10px; border: 1px solid var(--dowe-line); border-radius: 8px; background: var(--dowe-background); color: var(--dowe-primary); font: inherit; font-size: 12px; font-weight: 700; }
.data-select select:focus { outline: 3px solid var(--dowe-soft-primary); outline-offset: 1px; border-color: var(--dowe-primary); }
.data-refresh { display: grid; place-items: center; width: 38px; height: 38px; padding: 0; border: 1px solid var(--dowe-line); border-radius: 8px; background: var(--dowe-background); color: var(--dowe-primary); cursor: pointer; font-size: 18px; font-weight: 750; line-height: 1; }
.data-refresh:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.sr-only { position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0, 0, 0, 0); white-space: nowrap; border: 0; }
.data-layout { display: grid; grid-template-columns: 270px minmax(0, 1fr); gap: 16px; min-width: 0; }
.data-sidebar, .data-workspace { min-width: 0; min-height: 0; }
.data-sidebar { padding: 14px; }
.data-sidebar-head { display: flex; align-items: center; justify-content: space-between; padding: 3px 2px 12px; border-bottom: 1px solid var(--dowe-line); color: var(--dowe-primary); font-size: 12px; font-weight: 850; letter-spacing: .02em; }
.data-sidebar-head span:last-child { min-width: 22px; padding: 3px 6px; border-radius: 999px; background: var(--dowe-soft-muted); color: var(--dowe-muted); font-size: 10px; text-align: center; }
.connection-list { display: grid; gap: 6px; padding-top: 10px; }
.connection-row { display: flex; align-items: center; gap: 9px; width: 100%; padding: 9px 8px; border: 1px solid transparent; border-radius: 8px; background: transparent; color: var(--dowe-primary); cursor: pointer; text-align: left; }
.connection-row:hover { border-color: var(--dowe-line); background: var(--dowe-background); }
.connection-row.active { border-color: var(--dowe-secondary); background: var(--dowe-soft-secondary); }
.connection-dot { width: 8px; height: 8px; flex: 0 0 8px; border-radius: 50%; background: var(--dowe-secondary); box-shadow: 0 0 0 3px var(--dowe-soft-secondary); }
.connection-row span:nth-child(2) { min-width: 0; flex: 1; }
.connection-row strong, .connection-row small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.connection-row strong { font-size: 12px; }
.connection-row small { margin-top: 2px; color: var(--dowe-muted); font-size: 10px; }
.connection-arrow { color: var(--dowe-muted); font-size: 18px; line-height: 1; }
.data-workspace { padding: 20px; }
.data-workspace-head { display: flex; align-items: center; justify-content: space-between; gap: 12px; min-height: 28px; margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid var(--dowe-line); }
.data-path { flex: 1; color: var(--dowe-primary); font-size: 13px; font-weight: 800; }
.data-count { color: var(--dowe-muted); font-size: 11px; }
.data-back { padding: 0; border: 0; background: transparent; color: var(--dowe-primary); cursor: pointer; font-size: 12px; font-weight: 800; }
.data-back:hover { color: #294d77; text-decoration: underline; }
.entity-structure { display: grid; gap: 8px; margin-bottom: 16px; }
.entity-structure-head { display: flex; align-items: center; justify-content: space-between; color: var(--dowe-primary); font-size: 11px; font-weight: 850; letter-spacing: .06em; text-transform: uppercase; }
.entity-structure-head small { color: var(--dowe-muted); font-size: 10px; font-weight: 700; letter-spacing: 0; text-transform: none; }
.entity-cards { display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 10px; }
.entity-card { min-width: 0; overflow: hidden; border: 1px solid var(--dowe-line); border-radius: 9px; background: var(--dowe-background); }
.entity-card-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; padding: 10px 12px; border-bottom: 1px solid var(--dowe-line); background: var(--dowe-surface); }
.entity-card-head strong, .entity-card-head small { display: block; }
.entity-card-head strong { overflow: hidden; color: var(--dowe-primary); font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.entity-card-head small { margin-top: 3px; color: var(--dowe-muted); font-size: 10px; }
.entity-badge { flex: 0 0 auto; padding: 3px 6px; border-radius: 999px; background: var(--dowe-soft-primary); color: var(--dowe-primary); font-size: 9px; font-weight: 850; text-transform: uppercase; }
.entity-fields { max-height: 220px; overflow: auto; }
.entity-field { display: grid; grid-template-columns: minmax(90px, 1.2fr) minmax(60px, .8fr) minmax(0, 1.4fr); gap: 8px; align-items: center; padding: 7px 12px; border-bottom: 1px solid var(--dowe-line); font-size: 11px; }
.entity-field:last-child { border-bottom: 0; }
.entity-field code { overflow: hidden; color: var(--dowe-background-text); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; text-overflow: ellipsis; white-space: nowrap; }
.entity-type { color: var(--dowe-primary); font-size: 10px; font-weight: 800; }
.entity-flags { overflow: hidden; color: var(--dowe-muted); font-size: 9px; text-align: right; text-overflow: ellipsis; white-space: nowrap; }
.schema-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 10px; }
.schema-card { display: flex; align-items: center; gap: 10px; min-height: 72px; padding: 12px; border: 1px solid var(--dowe-line); border-radius: 9px; background: var(--dowe-background); color: var(--dowe-primary); cursor: pointer; text-align: left; transition: border-color .15s ease, background .15s ease, transform .15s ease; }
.schema-card:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); transform: translateY(-1px); }
.schema-icon { display: grid; flex: 0 0 28px; place-items: center; width: 28px; height: 28px; border-radius: 7px; background: var(--dowe-soft-muted); color: var(--dowe-primary); font-size: 18px; }
.schema-card > span:nth-child(2) { min-width: 0; flex: 1; }
.schema-card strong, .schema-card small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.schema-card strong { font-size: 12px; }
.schema-card small { margin-top: 4px; color: var(--dowe-muted); font-size: 10px; }
.schema-arrow { color: var(--dowe-muted); font-size: 18px; }
.data-table-wrap { max-width: 100%; max-height: calc(100vh - 300px); overflow: auto; border: 1px solid var(--dowe-line); border-radius: 8px; }
.data-table { width: 100%; min-width: 620px; border-collapse: separate; border-spacing: 0; color: var(--dowe-background-text); font-size: 12px; }
.data-table th { position: sticky; top: 0; z-index: 1; padding: 10px 12px; border-bottom: 1px solid var(--dowe-line); background: var(--dowe-surface); color: var(--dowe-primary); font-size: 10px; font-weight: 850; letter-spacing: .06em; text-align: left; text-transform: uppercase; }
.data-table td { max-width: 320px; padding: 10px 12px; border-bottom: 1px solid var(--dowe-line); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; vertical-align: top; }
.data-table tr:last-child td { border-bottom: 0; }
.data-table tbody tr:hover { background: var(--dowe-soft-primary); }
.data-table tbody tr[data-data-id] { cursor: pointer; }
.data-footnote { margin-top: 10px; color: var(--dowe-muted); font-size: 11px; }
.key-list { display: grid; gap: 6px; }
.key-row { display: flex; align-items: center; gap: 10px; width: 100%; padding: 10px 11px; border: 1px solid var(--dowe-line); border-radius: 8px; background: var(--dowe-background); color: var(--dowe-primary); cursor: pointer; text-align: left; }
.key-row:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.key-row.masked { cursor: not-allowed; opacity: .7; }
.key-row code { flex: 1; overflow: hidden; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 12px; text-overflow: ellipsis; white-space: nowrap; }
.key-row span:last-child { color: var(--dowe-muted); font-size: 17px; }
.key-icon { display: grid; flex: 0 0 23px; place-items: center; width: 23px; height: 23px; border-radius: 6px; background: var(--dowe-soft-muted); color: var(--dowe-primary); }
.value-view { display: grid; gap: 8px; }
.value-label { color: var(--dowe-muted); font-size: 11px; font-weight: 800; text-transform: uppercase; letter-spacing: .08em; }
.data-welcome { display: grid; place-items: center; min-height: 440px; padding: 40px; color: var(--dowe-muted); text-align: center; }
.data-welcome-icon { display: grid; place-items: center; width: 46px; height: 46px; margin-bottom: 12px; border-radius: 12px; background: var(--dowe-soft-primary); color: var(--dowe-primary); font-size: 24px; font-weight: 850; }
.data-welcome h3 { margin: 0 0 6px; color: var(--dowe-primary); font-size: 16px; }
.data-welcome p { max-width: 360px; margin: 0; line-height: 1.5; }
.data-empty { padding: 28px 8px; text-align: center; }
.item { border: 1px solid var(--dowe-line); background: var(--dowe-background); color: var(--dowe-background-text); border-radius: 8px; padding: 10px; cursor: pointer; text-align: left; box-shadow: 0 2px 8px rgba(23, 38, 58, .05); }
.item:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.item strong { display: block; margin-bottom: 3px; color: var(--dowe-background-title); }
.pill { display: inline-block; margin-right: 5px; padding: 2px 7px; border-radius: 9999px; background: var(--dowe-soft-secondary); color: #245d33; font-size: 11px; }
.source { max-height: 340px; overflow: auto; padding: 14px; border: 1px solid var(--dowe-line); border-radius: 8px; background: var(--dowe-background); color: var(--dowe-background-text); font-family: ui-monospace, SFMono-Regular, Menlo, monospace; line-height: 1.5; white-space: pre-wrap; }
.empty { padding: 16px 0; color: var(--dowe-muted); }
.flow { display: grid; gap: 8px; }
.edge { padding-left: 16px; color: var(--dowe-muted); font-size: 12px; }
.error { padding: 10px; border: 1px solid #edc5cb; border-radius: 8px; background: var(--dowe-soft-danger); color: #7b2c36; }
.card-head, .endpoint-detail-head, .response-head { display: flex; align-items: flex-start; justify-content: space-between; gap: 12px; }
.card-head h2, .endpoint-detail-head h2 { margin: 0; }
.endpoint-page { display: grid; gap: 16px; }
.endpoint-layout { display: grid; grid-template-columns: minmax(240px, .8fr) minmax(0, 1.8fr); gap: 16px; align-items: start; }
.endpoint-catalog, .endpoint-detail { min-width: 0; }
.endpoint-catalog:not(.card) .card-head { padding-bottom: 10px; }
.endpoint-list { display: grid; gap: 7px; max-height: calc(100vh - 240px); overflow: hidden; }
.endpoint-list > .list { max-height: calc(100vh - 280px); overflow: auto; }
.endpoint-row-shell { display: flex; align-items: center; gap: 8px; width: 100%; min-width: 0; padding: 9px 10px; border: 1px solid var(--dowe-line); border-radius: 9px; background: var(--dowe-background); color: var(--dowe-primary); }
.endpoint-row-shell:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.endpoint-row-shell.active { border-color: var(--dowe-secondary); background: var(--dowe-soft-secondary); }
.endpoint-select { display: flex; align-items: center; gap: 9px; min-width: 0; flex: 1; padding: 0; border: 0; background: transparent; color: inherit; cursor: pointer; text-align: left; }
.endpoint-select > span:nth-child(2) { min-width: 0; flex: 1; }
.endpoint-row-shell strong, .endpoint-row-shell small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.endpoint-row-shell strong { font-size: 12px; }
.endpoint-row-shell small { margin-top: 3px; color: var(--dowe-muted); font-size: 10px; }
.endpoint-row-actions { display: flex; align-items: center; gap: 2px; flex: 0 0 auto; }
.icon-action { display: grid; place-items: center; width: 30px; height: 30px; padding: 0; border: 0; border-radius: 7px; background: transparent; color: var(--dowe-muted); cursor: pointer; }
.icon-action svg { width: 16px; height: 16px; }
.icon-action:hover { background: rgba(255, 255, 255, .7); color: var(--dowe-primary); }
.icon-action.try-action:hover { background: var(--dowe-secondary); color: var(--dowe-secondary-text); }
.endpoint-select:focus-visible, .icon-action:focus-visible { outline: 3px solid var(--dowe-soft-primary); outline-offset: 2px; }
.endpoint-row { display: flex; align-items: center; gap: 9px; width: 100%; min-width: 0; padding: 10px; border: 1px solid var(--dowe-line); border-radius: 9px; background: var(--dowe-background); color: var(--dowe-primary); cursor: pointer; text-align: left; }
.endpoint-row:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.endpoint-row.active { border-color: var(--dowe-secondary); background: var(--dowe-soft-secondary); }
.endpoint-row > span:nth-child(2) { min-width: 0; flex: 1; }
.endpoint-row strong, .endpoint-row small { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.endpoint-row strong { font-size: 12px; }
.endpoint-row small { margin-top: 3px; color: var(--dowe-muted); font-size: 10px; }
.endpoint-arrow { color: var(--dowe-muted); font-size: 18px; }
.method-tag { display: inline-grid; flex: 0 0 auto; place-items: center; min-width: 43px; padding: 4px 7px; border-radius: 6px; font-size: 10px; font-weight: 900; letter-spacing: .04em; }
.method-get { background: #d9f2e1; color: #23623b; }
.method-post { background: #d9e9fb; color: #1f4d7c; }
.method-put { background: #fff0c7; color: #795316; }
.method-patch { background: #f0e2ff; color: #6a3d8d; }
.method-delete { background: #f9dede; color: #8b3131; }
.method-ws { background: var(--dowe-soft-primary); color: var(--dowe-primary); }
.endpoint-detail { padding: 20px; }
.endpoint-detail-head { padding-bottom: 16px; border-bottom: 1px solid var(--dowe-line); }
.endpoint-detail-head h2 { margin-top: 5px; color: var(--dowe-background-title); font-size: 23px; }
.endpoint-form { display: grid; gap: 14px; margin-top: 16px; }
.request-section { display: grid; gap: 8px; padding: 12px; border: 1px solid var(--dowe-line); border-radius: 9px; background: var(--dowe-background); }
.request-section-title { display: flex; align-items: center; justify-content: space-between; color: var(--dowe-primary); font-size: 11px; font-weight: 850; letter-spacing: .06em; text-transform: uppercase; }
.request-section-title span { color: var(--dowe-muted); font-size: 10px; font-weight: 700; letter-spacing: 0; text-transform: none; }
.request-fields { display: grid; grid-template-columns: repeat(auto-fit, minmax(210px, 1fr)); gap: 9px; }
.request-field { display: grid; gap: 5px; }
.request-field > span { display: flex; align-items: baseline; justify-content: space-between; gap: 8px; color: var(--dowe-primary); font-size: 11px; font-weight: 800; }
.request-field small { color: var(--dowe-muted); font-size: 9px; font-weight: 700; }
.request-field input, .request-field select, .request-section > input, .request-section > textarea, .ws-controls textarea { width: 100%; min-width: 0; padding: 9px 10px; border: 1px solid var(--dowe-line); border-radius: 7px; background: var(--dowe-surface); color: var(--dowe-background-text); font: inherit; font-size: 12px; }
.request-field input:focus, .request-field select:focus, .request-section > input:focus, .request-section > textarea:focus, .ws-controls textarea:focus { outline: 3px solid var(--dowe-soft-primary); outline-offset: 1px; border-color: var(--dowe-primary); }
.endpoint-actions { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }
.execute-button { padding: 9px 13px; border: 1px solid var(--dowe-secondary); border-radius: 8px; background: var(--dowe-secondary); color: var(--dowe-secondary-text); cursor: pointer; font-weight: 850; }
.execute-button:hover { filter: brightness(.96); }
.execute-button:disabled { cursor: wait; opacity: .55; }
.response-panel { display: grid; gap: 8px; padding-top: 14px; border-top: 1px solid var(--dowe-line); }
.status-badge { padding: 4px 8px; border-radius: 999px; font-size: 10px; font-weight: 850; }
.status-ok { background: var(--dowe-soft-secondary); color: #245d33; }
.status-error { background: var(--dowe-soft-danger); color: #7b2c36; }
.response-meta { display: flex; flex-wrap: wrap; gap: 5px 12px; color: var(--dowe-muted); font-size: 10px; }
.response-meta b { color: var(--dowe-primary); }
.response-body { max-height: 300px; margin: 0; }
.ws-controls { display: grid; gap: 9px; margin-top: 16px; }
.ws-log-list { display: grid; gap: 7px; max-height: 420px; margin-top: 16px; overflow: auto; }
.ws-log { display: grid; grid-template-columns: 72px 62px minmax(0, 1fr); gap: 8px; align-items: start; padding: 8px 10px; border: 1px solid var(--dowe-line); border-radius: 8px; background: var(--dowe-background); font-size: 10px; }
.ws-log > span { color: var(--dowe-muted); }
.ws-log strong { color: var(--dowe-primary); text-transform: uppercase; }
.ws-log pre { margin: 0; overflow: auto; white-space: pre-wrap; word-break: break-word; }
.modal-open { overflow: hidden; }
.modal-backdrop { position: fixed; inset: 0; z-index: 100; display: grid; place-items: center; padding: 24px; background: rgba(23, 38, 58, .42); backdrop-filter: blur(3px); }
.modal-surface { width: min(760px, 100%); max-height: min(760px, calc(100vh - 48px)); overflow: auto; padding: 22px; border: 1px solid var(--dowe-line); border-radius: 16px; background: var(--dowe-background); box-shadow: 0 24px 70px rgba(23, 38, 58, .25); }
.modal-header { display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; padding-bottom: 16px; border-bottom: 1px solid var(--dowe-line); }
.modal-header h2 { margin: 5px 0 0; color: var(--dowe-background-title); font-size: 23px; line-height: 1.2; overflow-wrap: anywhere; }
.modal-subtitle { margin-top: 5px; color: var(--dowe-muted); font-size: 12px; }
.modal-header-actions { display: flex; align-items: center; gap: 8px; flex: 0 0 auto; }
.modal-close { display: grid; place-items: center; width: 34px; height: 34px; padding: 0; border: 1px solid var(--dowe-line); border-radius: 8px; background: var(--dowe-background); color: var(--dowe-primary); cursor: pointer; }
.modal-close svg { width: 16px; height: 16px; }
.modal-close:hover { border-color: var(--dowe-primary); background: var(--dowe-soft-primary); }
.modal-close:focus-visible { outline: 3px solid var(--dowe-soft-primary); outline-offset: 2px; }
.source-modal { width: min(820px, 100%); }
.source-modal .source { max-height: calc(100vh - 190px); margin-top: 16px; }
@media (max-width: 820px) { body { overflow: auto; } .side { position: relative; width: 100%; height: auto; min-height: auto; overflow: visible; padding: 20px 16px 16px; border-right: 0; border-bottom: 1px solid var(--dowe-line); } .side-footer { margin-top: 18px; padding-top: 16px; } .nav { grid-template-columns: repeat(3, 1fr); } .nav button { justify-content: center; padding-inline: 8px; } .nav-text { flex: 0 1 auto; } .main { width: 100%; height: auto; min-height: 0; margin-left: 0; overflow: visible; padding: 22px 16px; } .card, .card.wide { grid-column: 1 / -1; } .data-studio-toolbar { align-items: center; } .provider-tabs { grid-template-columns: repeat(2, minmax(150px, 1fr)); overflow-x: auto; } .data-controls { align-items: stretch; } .data-select { flex-basis: auto; } .data-layout, .endpoint-layout { grid-template-columns: 1fr; } .data-sidebar, .data-workspace { min-height: auto; } .data-table-wrap { max-height: none; } .endpoint-list { max-height: none; } }
"#;

