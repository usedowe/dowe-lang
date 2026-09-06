pub(crate) const IMMUTABLE_CACHE_CONTROL: &str = "public, max-age=31536000, immutable";

pub(crate) fn chunk_response(
    web: &WebOutput,
    path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Option<Response> {
    let prefix = "/chunks/";
    let chunk_path = path.strip_prefix(prefix)?;
    let relative = std::path::Path::new("web/chunks").join(chunk_path);
    if let Some(chunk) = web
        .runtime_chunks()
        .into_iter()
        .find(|chunk| chunk.relative_path == relative)
    {
        return Some(cacheable_text_response(
            chunk.content,
            "application/javascript; charset=utf-8",
            request_headers,
            cache_control,
        ));
    }
    if let Some(chunk) = web
        .translation_chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative)
    {
        return Some(cacheable_text_response(
            chunk.content.clone(),
            "application/javascript; charset=utf-8",
            request_headers,
            cache_control,
        ));
    }
    let chunk = web
        .chunks
        .iter()
        .find(|chunk| chunk.relative_path == relative || chunk.css_relative_path == relative)?;
    let content_type = if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else {
        "application/javascript; charset=utf-8"
    };
    let content = if path.ends_with(".css") {
        chunk.css_content.clone()
    } else {
        chunk.content.clone()
    };

    Some(cacheable_text_response(
        content,
        content_type,
        request_headers,
        cache_control,
    ))
}

pub(crate) fn cacheable_design_css_response(
    project: &CompiledProject,
    relative_path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    let paths = [
        project.root.join(".dowe").join(relative_path),
        project.root.join(".dowe/apps/desktop").join(relative_path),
    ];
    match paths.iter().find_map(|path| fs::read_to_string(path).ok()) {
        Some(css) => cacheable_text_response(
            css,
            "text/css; charset=utf-8",
            request_headers,
            cache_control,
        ),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn cacheable_dev_design_css_response(
    project: &CompiledProject,
    web: &WebOutput,
    relative_path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    let response = cacheable_design_css_response(
        project,
        relative_path,
        request_headers,
        cache_control,
    );
    if response.status() != StatusCode::NOT_FOUND {
        return response;
    }
    dev_web_artifact_response(project, web, relative_path, request_headers, cache_control)
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response())
}

pub(crate) fn dev_web_artifact_response(
    project: &CompiledProject,
    web: &WebOutput,
    relative_path: &str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Option<Response> {
    let relative_path = Path::new(relative_path);
    if relative_path.is_absolute()
        || relative_path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let artifact = dowe_compiler::web_artifacts_for_target(
        web,
        &project.font_config,
        &project.design_config,
        Path::new(""),
        "web",
    )
    .into_iter()
    .find(|artifact| artifact.relative_path == relative_path)?;
    let content_type = if relative_path.extension().and_then(|value| value.to_str()) == Some("css") {
        "text/css; charset=utf-8"
    } else if relative_path.extension().and_then(|value| value.to_str()) == Some("json") {
        "application/json; charset=utf-8"
    } else if relative_path.extension().and_then(|value| value.to_str()) == Some("js") {
        "application/javascript; charset=utf-8"
    } else {
        "text/html; charset=utf-8"
    };
    Some(cacheable_text_response(
        artifact.content,
        content_type,
        request_headers,
        cache_control,
    ))
}

pub(crate) fn design_css_chunk_relative_path(path: &str) -> Option<String> {
    let file_name = path.strip_prefix("/chunks/design/")?;
    if file_name.is_empty() || file_name.contains('/') || !file_name.ends_with(".css") {
        return None;
    }
    Some(format!("web/chunks/design/{file_name}"))
}

pub(crate) fn font_response(project: &CompiledProject, path: &str) -> Option<Response> {
    let font_path = path.strip_prefix("/fonts/")?;
    let relative = Path::new(font_path);
    if relative.is_absolute()
        || relative.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Some(StatusCode::NOT_FOUND.into_response());
    }

    let path = project.root.join(".dowe/fonts").join(relative);
    let Ok(content) = fs::read(path) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };

    Some(
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "font/ttf"),
                (CACHE_CONTROL, "public, max-age=31536000"),
            ],
            content,
        )
            .into_response(),
    )
}

pub(crate) fn project_asset_response(
    project: &CompiledProject,
    path: &str,
    cache_control: &'static str,
) -> Option<Response> {
    let (directory, relative) = if let Some(relative) = path.strip_prefix("/assets/") {
        ("assets", relative)
    } else if let Some(relative) = path.strip_prefix("/icons/") {
        ("icons", relative)
    } else {
        return None;
    };
    let Some(path) = safe_project_asset_path(&project.root, directory, relative) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };
    let Ok(content) = fs::read(&path) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };
    let content_type = asset_content_type(&path);
    Some(
        (
            StatusCode::OK,
            [(CONTENT_TYPE, content_type), (CACHE_CONTROL, cache_control)],
            content,
        )
            .into_response(),
    )
}

fn safe_project_asset_path(
    root: &Path,
    directory: &str,
    relative: &str,
) -> Option<std::path::PathBuf> {
    let relative = Path::new(relative);
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    let base = root.join(directory);
    if fs::symlink_metadata(&base).ok()?.file_type().is_symlink() {
        return None;
    }
    let mut current = base;
    for component in relative.components() {
        let Component::Normal(value) = component else {
            return None;
        };
        current.push(value);
        let metadata = fs::symlink_metadata(&current).ok()?;
        if metadata.file_type().is_symlink() {
            return None;
        }
    }
    current.is_file().then_some(current)
}

fn asset_content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|value| value.to_str()) {
        Some("png") => "image/png",
        Some("ico") => "image/x-icon",
        Some("svg") => "image/svg+xml",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("avif") => "image/avif",
        Some("json") => "application/json",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn render_page(page: &ViewPage) -> Response {
    with_cache_control(
        Html(inject_dev_client(&page.html_document)).into_response(),
        "no-store",
    )
}

pub(crate) fn inspector_selection_response(project: &CompiledProject, body: &Bytes) -> Response {
    if body.len() > 64 * 1024 {
        return StatusCode::PAYLOAD_TOO_LARGE.into_response();
    }
    let Ok(value) = serde_json::from_slice::<Value>(body) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(node) = value.get("node").and_then(Value::as_object) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    let Some(path) = node.get("path").and_then(Value::as_str) else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    if !safe_inspector_source_path(path) {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let mut selected_node = Map::new();
    for key in ["id", "kind", "path", "startLine", "endLine"] {
        if let Some(value) = node.get(key)
            && matches!(value, Value::String(_) | Value::Number(_))
        {
            selected_node.insert(key.to_string(), value.clone());
        }
    }
    if let Some(usages) = node.get("usages").and_then(Value::as_array) {
        let usages = usages
            .iter()
            .filter_map(|usage| {
                let usage = usage.as_object()?;
                let path = usage.get("path").and_then(Value::as_str)?;
                if !safe_inspector_source_path(path) {
                    return None;
                }
                let line = usage.get("line").and_then(Value::as_u64)?;
                let column = usage.get("column").and_then(Value::as_u64)?;
                Some(json!({"path": path, "line": line, "column": column}))
            })
            .collect::<Vec<_>>();
        selected_node.insert("usages".to_string(), Value::Array(usages));
    }
    let selection = json!({"node": selected_node});
    let selection_root = project.root.join(".dowe/dev");
    if let Err(error) = fs::create_dir_all(&selection_root) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("inspector selection directory failed: {error}"),
        )
            .into_response();
    }
    let content = match serde_json::to_vec(&selection) {
        Ok(content) => content,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let staged = selection_root.join(".inspector-selection.json.tmp");
    let target = selection_root.join("inspector-selection.json");
    if fs::write(&staged, content).is_err() || fs::rename(&staged, &target).is_err() {
        let _ = fs::remove_file(&staged);
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    json_response_text(r#"{"ok":true}"#.to_string())
}

fn safe_inspector_source_path(path: &str) -> bool {
    let path = Path::new(path);
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn dev_client_response(inspector_enabled: bool, server_inspector_url: Option<&str>) -> Response {
    javascript_response(dev_client_script(inspector_enabled, server_inspector_url))
}

pub(crate) fn studio_host_client_response() -> Response {
    let mut script = dev_client_script(false, None);
    script.push('\n');
    script.push_str(include_str!("../studio_inspector_client.js"));
    javascript_response(script)
}

pub(crate) fn studio_preview_client_response() -> Response {
    javascript_response(studio_preview_client_script())
}

fn studio_preview_client_script() -> String {
    let mut script = dev_client_script(false, None);
    script.push_str(
        r##"
const DOWE_STUDIO_PREVIEW=true;
let doweStudioPreviewOrigin=null;
let doweStudioPreviewNonce=null;
let doweStudioPreviewManifest=null;
let doweStudioPreviewHover="";
let doweStudioPreviewActiveNode=null;
let doweStudioPreviewManifestAttempts=0;
let doweStudioPreviewOverlay=null;
let doweStudioPreviewTooltip=null;
function postDoweStudioPreview(type,payload){if(window.parent===window||!doweStudioPreviewNonce)return;const message={channel:"dowe-studio",version:1,nonce:doweStudioPreviewNonce,type,payload};try{window.parent.postMessage(message,doweStudioPreviewOrigin||"*");}catch(error){}}
function studioPreviewNode(target){return target instanceof Element?target.closest("[data-dowe-node]"):null;}
function studioPreviewMetadata(node){if(!node)return null;return doweStudioPreviewManifest?.nodes?.find(item=>item.id===node.dataset.doweNode)||{id:node.dataset.doweNode,kind:"Component",path:"",startLine:0,endLine:0};}
function studioPreviewStyle(element,values){for(const [name,value] of Object.entries(values))element.style[name]=value;}
function studioPreviewEnsureUi(){if(doweStudioPreviewOverlay)return;doweStudioPreviewOverlay=document.createElement("div");studioPreviewStyle(doweStudioPreviewOverlay,{position:"fixed",zIndex:"2147483646",pointerEvents:"none",display:"none",boxSizing:"border-box",border:"2px solid #6bc670",background:"#6bc67022"});document.body.append(doweStudioPreviewOverlay);doweStudioPreviewTooltip=document.createElement("div");studioPreviewStyle(doweStudioPreviewTooltip,{position:"fixed",zIndex:"2147483647",pointerEvents:"none",display:"none",maxWidth:"420px",padding:"6px 8px",borderRadius:"6px",background:"#1f3a5f",color:"#f8fbff",font:"12px/1.35 system-ui",boxShadow:"0 6px 20px #0005"});document.body.append(doweStudioPreviewTooltip);}
function studioPreviewClearHover(){doweStudioPreviewHover="";doweStudioPreviewActiveNode=null;if(doweStudioPreviewOverlay)doweStudioPreviewOverlay.style.display="none";if(doweStudioPreviewTooltip)doweStudioPreviewTooltip.style.display="none";}
function studioPreviewShowHover(node,item,x,y){doweStudioPreviewActiveNode=node;studioPreviewEnsureUi();const box=node.getBoundingClientRect();studioPreviewStyle(doweStudioPreviewOverlay,{display:"block",left:String(box.left)+"px",top:String(box.top)+"px",width:String(box.width)+"px",height:String(box.height)+"px"});doweStudioPreviewTooltip.textContent=String(item.kind||"Component")+" · "+String(item.path||"")+":"+String(item.startLine||"");const left=Math.min(Math.max(8,x+12),Math.max(8,window.innerWidth-430));const top=Math.min(Math.max(8,y+12),Math.max(8,window.innerHeight-48));studioPreviewStyle(doweStudioPreviewTooltip,{display:"block",left:String(left)+"px",top:String(top)+"px"});if(item.id!==doweStudioPreviewHover){doweStudioPreviewHover=item.id;postDoweStudioPreview("studio:view:hover",{node:item});}}
function studioPreviewBuilderPayload(event){const value=event.dataTransfer?.getData("application/x-dowe-builder")||event.dataTransfer?.getData("text/plain")||"";if(!value)return null;try{const payload=JSON.parse(value);if(!payload||typeof payload.component!=="string"||!/^[A-Z][A-Za-z0-9]*$/.test(payload.component))return null;return {component:payload.component,props:payload.props&&typeof payload.props==="object"?payload.props:{}};}catch(error){return /^[A-Z][A-Za-z0-9]*$/.test(value)?{component:value,props:{}}:null;}}
function studioPreviewBuilderTarget(target){const node=studioPreviewNode(target);return node?studioPreviewMetadata(node):null;}
window.addEventListener("message",event=>{const message=event.data;if(!message||message.channel!=="dowe-studio"||event.source!==window.parent)return;if(message.type==="studio:hello"&&typeof message.nonce==="string"){doweStudioPreviewNonce=message.nonce;doweStudioPreviewOrigin=event.origin&&event.origin!=="null"?event.origin:null;studioPreviewEnsureUi();postDoweStudioPreview("studio:ready",{nonce:doweStudioPreviewNonce});}});
function studioPreviewLoadManifest(){fetch("/_dowe/dev/inspector.json",{cache:"no-store"}).then(response=>response.ok?response.json():null).then(manifest=>{if(manifest?.nodes){doweStudioPreviewManifest=manifest;if(doweStudioPreviewActiveNode){const node=doweStudioPreviewActiveNode;const item=studioPreviewMetadata(node);if(item){doweStudioPreviewHover="";const box=node.getBoundingClientRect();studioPreviewShowHover(node,item,box.left,box.top);}}}else if(doweStudioPreviewManifestAttempts++<20){setTimeout(studioPreviewLoadManifest,500);}}).catch(()=>{if(doweStudioPreviewManifestAttempts++<20)setTimeout(studioPreviewLoadManifest,500);});}studioPreviewLoadManifest();
document.addEventListener("mousemove",event=>{const node=studioPreviewNode(event.target);const item=studioPreviewMetadata(node);if(!item){studioPreviewClearHover();return;}studioPreviewShowHover(node,item,event.clientX,event.clientY);},true);
document.addEventListener("dragover",event=>{if(!doweStudioPreviewNonce)return;const payload=studioPreviewBuilderPayload(event);const node=studioPreviewNode(event.target);const item=studioPreviewBuilderTarget(event.target);if(!payload||!node||!item)return;event.preventDefault();event.dataTransfer.dropEffect="copy";studioPreviewShowHover(node,item,event.clientX,event.clientY);postDoweStudioPreview("studio:builder:dragover",{node:item,payload});},true);
document.addEventListener("drop",event=>{if(!doweStudioPreviewNonce)return;const payload=studioPreviewBuilderPayload(event);const item=studioPreviewBuilderTarget(event.target);if(!payload||!item)return;event.preventDefault();event.stopPropagation();postDoweStudioPreview("studio:builder:drop",{node:item,payload,relation:"child"});},true);
window.addEventListener("blur",studioPreviewClearHover);
document.addEventListener("click",event=>{if(!doweStudioPreviewNonce)return;const node=studioPreviewNode(event.target);const item=studioPreviewMetadata(node);if(!item)return;studioPreviewShowHover(node,item,event.clientX,event.clientY);event.preventDefault();event.stopPropagation();postDoweStudioPreview("studio:view:selected",{node:item});},true);
"##,
    );
    script
}

pub(crate) fn javascript_response(content: String) -> Response {
    web_text_response(
        content,
        "application/javascript; charset=utf-8",
        Some("no-store"),
    )
}

pub(crate) fn cacheable_javascript_response(
    content: String,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    cacheable_text_response(
        content,
        "application/javascript; charset=utf-8",
        request_headers,
        cache_control,
    )
}

pub(crate) fn json_response_text(content: String) -> Response {
    web_text_response(content, "application/json; charset=utf-8", Some("no-store"))
}

fn cacheable_text_response(
    content: String,
    content_type: &'static str,
    request_headers: &HeaderMap,
    cache_control: &'static str,
) -> Response {
    let etag = content_etag(content.as_bytes());
    if request_headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|values| values.split(',').any(|value| value.trim() == etag))
    {
        let mut response = StatusCode::NOT_MODIFIED.into_response();
        response.headers_mut().insert(
            ETAG,
            HeaderValue::from_str(&etag).expect("valid generated etag"),
        );
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
        return response;
    }
    let mut response = web_text_response(content, content_type, Some(cache_control));
    response.headers_mut().insert(
        ETAG,
        HeaderValue::from_str(&etag).expect("valid generated etag"),
    );
    response
}

fn web_text_response(
    content: String,
    content_type: &'static str,
    cache_control: Option<&'static str>,
) -> Response {
    let mut response = (StatusCode::OK, content).into_response();
    response
        .headers_mut()
        .insert(CONTENT_TYPE, HeaderValue::from_static(content_type));
    if let Some(cache_control) = cache_control {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    }
    response
}

fn content_etag(content: &[u8]) -> String {
    let digest = Sha256::digest(content)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    format!(r#"W/"{digest}""#)
}

pub(crate) fn generated_web_manifest_response(
    project: &CompiledProject,
    web: &WebOutput,
) -> Response {
    let path = project.root.join(".dowe/web/manifest.json");
    match fs::read_to_string(path) {
        Ok(content) => (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "application/json; charset=utf-8"),
                (CACHE_CONTROL, "no-store"),
            ],
            content,
        )
            .into_response(),
        Err(_) => dev_web_artifact_response(
            project,
            web,
            "web/manifest.json",
            &HeaderMap::new(),
            "no-store",
        )
        .unwrap_or_else(|| StatusCode::NOT_FOUND.into_response()),
    }
}

pub(crate) fn generated_json_response(project: &CompiledProject, relative_path: &str) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(content) => (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "application/json; charset=utf-8"),
                (CACHE_CONTROL, "no-store"),
            ],
            content,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn production_json_response(project: &CompiledProject, relative_path: &str) -> Response {
    let path = project.root.join(".dowe").join(relative_path);
    match fs::read_to_string(path) {
        Ok(content) => {
            web_text_response(content, "application/json; charset=utf-8", Some("no-cache"))
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub(crate) fn with_cache_control(mut response: Response, cache_control: &'static str) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    response
}

pub(crate) fn dev_module_response(project: &CompiledProject, path: &str) -> Option<Response> {
    let relative = path.strip_prefix("/_dowe/dev/modules/")?;
    let relative = Path::new(relative);
    let components = relative.components().collect::<Vec<_>>();
    if components.len() != 2
        || components
            .iter()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Some(StatusCode::NOT_FOUND.into_response());
    }
    let file = project.root.join(".dowe/dev/modules").join(relative);
    let Ok(content) = fs::read(file) else {
        return Some(StatusCode::NOT_FOUND.into_response());
    };

    Some(
        (
            StatusCode::OK,
            [
                (CONTENT_TYPE, "application/octet-stream"),
                (CACHE_CONTROL, "no-store"),
            ],
            content,
        )
            .into_response(),
    )
}

fn inject_dev_client(html: &str) -> String {
    let script = r#"<script type="module" src="/_dowe/dev/client.js"></script>"#;
    if html.contains(script) {
        return html.to_string();
    }

    if let Some(index) = html.rfind("</body>") {
        let mut output = String::with_capacity(html.len() + script.len());
        output.push_str(&html[..index]);
        output.push_str(script);
        output.push_str(&html[index..]);
        output
    } else {
        format!("{html}{script}")
    }
}

fn dev_client_script(inspector_enabled: bool, server_inspector_url: Option<&str>) -> String {
    let refresh = if inspector_enabled {
        "window.__doweInspectorRefresh?.();"
    } else {
        ""
    };
    let hmr = format!(
        r#"const DOWE_STUDIO_CHANNEL="dowe-studio";let doweStudioOrigin=null;let doweStudioNonce=null;function postDoweStudio(type,payload){{if(window.parent===window)return;const message={{channel:DOWE_STUDIO_CHANNEL,version:1,type,payload}};try{{window.parent.postMessage(message,doweStudioOrigin||"*");}}catch(error){{}}}}window.addEventListener("message",event=>{{const message=event.data;if(!message||message.channel!==DOWE_STUDIO_CHANNEL||event.source!==window.parent)return;if(message.type==="studio:hello"&&typeof message.nonce==="string"){{doweStudioNonce=message.nonce;doweStudioOrigin=event.origin&&event.origin!=="null"?event.origin:null;postDoweStudio("studio:ready",{{nonce:doweStudioNonce}});}}}});const protocol=location.protocol==="https:"?"wss":"ws";let active=true;let hmrQueue=Promise.resolve();function queueHotUpdate(version){{hmrQueue=hmrQueue.then(async()=>{{if(typeof window.__doweHotUpdate==="function"){{try{{await window.__doweHotUpdate(version||"");{refresh}return;}}catch(error){{}}}}location.reload();}}).catch(()=>{{}});}}function connect(){{if(!active)return;const socket=new WebSocket(`${{protocol}}://${{location.host}}/_dowe/dev/ws`);socket.onmessage=async(event)=>{{try{{const message=JSON.parse(event.data);postDoweStudio("studio:dev:event",message);if(message.type==="module_update"&&message.target==="web"){{queueHotUpdate(message.version||"");return;}}if(message.type==="reload"&&(message.target==="web"||message.target==="desktop")){{queueHotUpdate(message.version||"");return;}}if(message.type==="shutdown")active=false;}}catch(error){{}}}};socket.onclose=()=>{{if(active)setTimeout(connect,250);}};}}connect();"#
    );
    if inspector_enabled {
        let icon = serde_json::to_string(include_str!("../dowe_inspector_icon.svg"))
            .expect("Dowe inspector icon must be JSON encodable");
        let server_inspector_url = server_inspector_url
            .map(|url| {
                serde_json::to_string(url).expect("Server inspector URL must be JSON encodable")
            })
            .unwrap_or_else(|| "null".to_string());
        let client = include_str!("../dev_inspector_client.js")
            .replace("\"__DOWE_INSPECTOR_ICON_SVG__\"", &icon)
            .replace("\"__DOWE_SERVER_INSPECTOR_URL__\"", &server_inspector_url);
        format!("{hmr}\n{client}")
    } else {
        hmr.to_string()
    }
}

#[cfg(test)]
mod project_asset_tests {
    use super::{
        asset_content_type, dev_client_script, safe_inspector_source_path, safe_project_asset_path,
        studio_preview_client_script,
    };
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    #[test]
    fn resolves_regular_project_assets_and_rejects_traversal() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons/web")).expect("icons");
        fs::write(temp.path().join("icons/web/favicon-32x32.png"), "png").expect("asset");

        assert!(safe_project_asset_path(temp.path(), "icons", "web/favicon-32x32.png").is_some());
        assert!(safe_project_asset_path(temp.path(), "icons", "../main.dowe").is_none());
        assert!(safe_project_asset_path(temp.path(), "icons", "/etc/passwd").is_none());
        assert_eq!(asset_content_type(Path::new("favicon.png")), "image/png");
    }

    #[test]
    fn inspector_client_is_only_included_for_dev_web_output() {
        let inspector = dev_client_script(true, Some("http://127.0.0.1:8081/_dowe/dev/server/"));
        assert!(inspector.contains("Dowe inspect"));
        assert!(inspector.contains("setPointerCapture"));
        assert!(inspector.contains("dowe-inspector-position"));
        assert!(inspector.contains("left:\"16px\""));
        assert!(inspector.contains("right:\"auto\""));
        assert!(inspector.contains("Number.isFinite(top)"));
        assert!(inspector.contains("dowe-inspector-enabled"));
        assert!(inspector.contains("dowe-inspector-hidden"));
        assert!(inspector.contains("dowe-inspector-panel-open"));
        assert!(inspector.contains("KeyD"));
        assert!(inspector.contains("KeyR"));
        assert!(inspector.contains("#1f3a5f"));
        assert!(inspector.contains("#6bc670"));
        assert!(inspector.contains("rgb(31,58,95)"));
        assert!(!inspector.contains("__DOWE_INSPECTOR_ICON_SVG__"));
        assert!(inspector.contains("function solarIcon"));
        assert!(inspector.contains("Open Dowe Server Inspector"));
        assert!(inspector.contains("http://127.0.0.1:8081/_dowe/dev/server/"));
        assert!(inspector.contains("aria-label"));
        assert!(inspector.contains("Routes"));
        assert!(inspector.contains("Show details"));
        assert!(inspector.contains("loadManifest();"));
        assert!(inspector.contains("DOWE_STUDIO_CHANNEL"));
        assert!(inspector.contains("studio:dev:event"));
        assert!(!inspector.contains("inspectorPreview"));
        let studio_preview = studio_preview_client_script();
        assert!(studio_preview.contains("DOWE_STUDIO_PREVIEW"));
        assert!(studio_preview.contains("studio:view:hover"));
        assert!(studio_preview.contains("studio:view:selected"));
        assert!(studio_preview.contains("studio:builder:dragover"));
        assert!(studio_preview.contains("studio:builder:drop"));
        assert!(studio_preview.contains("application/x-dowe-builder"));
        assert!(studio_preview.contains("studioPreviewShowHover"));
        assert!(studio_preview.contains("if(!node)return null"));
        assert!(studio_preview.contains("studioPreviewLoadManifest"));
        assert!(studio_preview.contains("document.addEventListener(\"mousemove\",event=>{const node"));
        assert!(studio_preview.contains("nonce:doweStudioPreviewNonce"));
        assert!(!studio_preview.contains("Dowe Devtools"));
        let studio_host = include_str!("../studio_inspector_client.js");
        assert!(studio_host.contains("Dowe Studio Inspector"));
        assert!(studio_host.contains("Pasa el cursor"));
        assert!(studio_host.contains("studioInspectorSendToChat"));
        assert!(studio_host.contains("Send to chat"));
        assert!(studio_host.contains("sendButton.onclick"));
        assert!(!studio_host.contains("studioInspectorSendToChat(message.payload.node)"));
        assert!(studio_host.contains("data-dowe-studio-builder-item"));
        assert!(studio_host.contains("application/x-dowe-builder"));
        assert!(studio_host.contains("studio:builder:drop"));
        assert!(studio_host.contains("studioInspectorStageBuilderDrop"));
        assert!(studio_host.contains("studioInspector.selected"));
        assert!(studio_host.contains("functionName,args"));
        assert!(studio_host.contains("stageStudioChanges"));
        assert!(studio_host.contains("changePlan"));
        assert!(studio_host.contains("Builder change review"));
        assert!(studio_host.contains("#studio-chat"));
        assert!(studio_host.contains("applyStudioChanges"));
        assert!(studio_host.contains("rejectStudioChanges"));
        assert!(studio_host.contains("studio:view:hover"));
        assert!(!studio_host.contains("Dowe Devtools"));
        assert!(!inspector.contains("<iframe"));
        assert!(dev_client_script(true, None).contains("const SERVER_INSPECTOR_URL=null;"));
        assert!(!dev_client_script(false, None).contains("Dowe inspect"));
        assert!(!dev_client_script(false, None).contains("Inspector"));
        assert!(safe_inspector_source_path("views/pages/home.dowe"));
        assert!(!safe_inspector_source_path(""));
        assert!(!safe_inspector_source_path("../main.dowe"));
        assert!(!safe_inspector_source_path("/tmp/main.dowe"));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinked_project_assets() {
        let temp = TempDir::new().expect("tempdir");
        fs::create_dir_all(temp.path().join("icons")).expect("icons");
        fs::write(temp.path().join("outside.png"), "outside").expect("outside");
        std::os::unix::fs::symlink(
            temp.path().join("outside.png"),
            temp.path().join("icons/favicon.png"),
        )
        .expect("symlink");

        assert!(safe_project_asset_path(temp.path(), "icons", "favicon.png").is_none());
    }
}
