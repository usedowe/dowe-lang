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

