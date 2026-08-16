const state={enabled:false,selected:null,manifest:null,overlay:null,tooltip:null,panel:null,button:null,drag:{active:false,moved:false,suppressClick:false,offsetX:0,offsetY:0,startX:0,startY:0}};
function nodeAt(target){return target instanceof Element?target.closest("[data-dowe-node]"):null;}
function metadata(node){return state.manifest?.nodes?.find(item=>item.id===node?.dataset.doweNode)||null;}
function reference(item){return item?`${item.path}:${item.startLine}-${item.endLine}`:"";}
function style(element,values){for(const [name,value] of Object.entries(values))element.style[name]=value;}
function clamp(value,min,max){return Math.min(Math.max(value,min),max);}
function positionButton(left,top){
  if(!state.button)return;
  const rect=state.button.getBoundingClientRect();
  const margin=8;
  const maxLeft=Math.max(margin,window.innerWidth-rect.width-margin);
  const maxTop=Math.max(margin,window.innerHeight-rect.height-margin);
  style(state.button,{left:`${clamp(left,margin,maxLeft)}px`,top:`${clamp(top,margin,maxTop)}px`,right:"auto",bottom:"auto"});
}
function restoreButtonPosition(){
  try{
    const value=JSON.parse(localStorage.getItem("dowe-inspector-position")||"null");
    if(Number.isFinite(value?.left)&&Number.isFinite(value?.top))positionButton(value.left,value.top);
  }catch(error){}
}
function saveButtonPosition(){
  if(!state.button||!state.button.style.left)return;
  try{localStorage.setItem("dowe-inspector-position",JSON.stringify({left:parseFloat(state.button.style.left),top:parseFloat(state.button.style.top)}));}catch(error){}
}
function beginButtonDrag(event){
  if(event.button!==undefined&&event.button!==0)return;
  const rect=state.button.getBoundingClientRect();
  state.drag={active:true,moved:false,suppressClick:false,offsetX:event.clientX-rect.left,offsetY:event.clientY-rect.top,startX:event.clientX,startY:event.clientY};
  try{state.button.setPointerCapture?.(event.pointerId);}catch(error){}
  state.button.style.cursor="grabbing";
  event.preventDefault();
}
function moveButton(event){
  if(!state.drag.active)return;
  const moved=Math.hypot(event.clientX-state.drag.startX,event.clientY-state.drag.startY)>4;
  state.drag.moved=state.drag.moved||moved;
  positionButton(event.clientX-state.drag.offsetX,event.clientY-state.drag.offsetY);
  event.preventDefault();
}
function endButtonDrag(event){
  if(!state.drag.active)return;
  try{state.button.releasePointerCapture?.(event.pointerId);}catch(error){}
  state.drag.active=false;
  state.drag.suppressClick=state.drag.moved;
  state.button.style.cursor="grab";
  if(state.drag.moved)saveButtonPosition();
}
function keepButtonInViewport(){
  if(!state.button?.style.left)return;
  positionButton(parseFloat(state.button.style.left),parseFloat(state.button.style.top));
}
function ensureUi(){
  if(state.button)return;
  state.button=document.createElement("button");
  state.button.id="dowe-inspector-ui";
  state.button.type="button";
  state.button.textContent="Dowe inspect";
  state.button.setAttribute("aria-pressed","false");
  state.button.setAttribute("aria-label","Toggle Dowe view inspector");
  state.button.title="Drag to reposition; click to toggle inspector";
  style(state.button,{position:"fixed",left:"16px",right:"auto",bottom:"16px",zIndex:"2147483646",border:"1px solid #475569",borderRadius:"999px",padding:"8px 12px",background:"#0f172a",color:"#fff",font:"600 12px system-ui",cursor:"grab",touchAction:"none",userSelect:"none",boxShadow:"0 8px 30px #0005"});
  state.button.addEventListener("click",()=>{if(state.drag.suppressClick){state.drag.suppressClick=false;return;}toggle();});
  state.button.addEventListener("pointerdown",beginButtonDrag);
  state.button.addEventListener("pointermove",moveButton);
  state.button.addEventListener("pointerup",endButtonDrag);
  state.button.addEventListener("pointercancel",endButtonDrag);
  document.body.append(state.button);
  restoreButtonPosition();
  state.overlay=document.createElement("div");
  style(state.overlay,{position:"fixed",zIndex:"2147483644",pointerEvents:"none",border:"2px solid #38bdf8",background:"#38bdf822",display:"none",boxSizing:"border-box"});
  document.body.append(state.overlay);
  state.tooltip=document.createElement("div");
  style(state.tooltip,{position:"fixed",zIndex:"2147483645",pointerEvents:"none",display:"none",maxWidth:"420px",padding:"6px 8px",borderRadius:"6px",background:"#0f172a",color:"#fff",font:"12px/1.35 system-ui",boxShadow:"0 6px 20px #0005"});
  document.body.append(state.tooltip);
  state.panel=document.createElement("div");
  state.panel.id="dowe-inspector-panel";
  style(state.panel,{position:"fixed",right:"16px",bottom:"58px",zIndex:"2147483645",display:"none",width:"min(420px,calc(100vw - 32px))",maxHeight:"50vh",overflow:"auto",padding:"14px",border:"1px solid #334155",borderRadius:"12px",background:"#020617",color:"#e2e8f0",font:"13px/1.45 system-ui",boxShadow:"0 12px 40px #0008"});
  document.body.append(state.panel);
  document.addEventListener("mousemove",event=>{if(!state.enabled)return;const node=nodeAt(event.target);if(!node){clearHover();return;}showHover(node,event.clientX,event.clientY);});
  document.addEventListener("click",event=>{if(!state.enabled)return;const node=nodeAt(event.target);if(!node||event.target.closest("#dowe-inspector-ui,#dowe-inspector-panel"))return;event.preventDefault();event.stopPropagation();select(node);},true);
  document.addEventListener("keydown",event=>{if(event.altKey&&event.shiftKey&&event.code==="KeyD"){event.preventDefault();toggle();}if(event.key==="Escape"&&state.enabled)toggle(false);});
  window.addEventListener("resize",keepButtonInViewport);
}
function clearHover(){if(state.overlay)state.overlay.style.display="none";if(state.tooltip)state.tooltip.style.display="none";}
function showHover(node,x,y){const box=node.getBoundingClientRect();style(state.overlay,{display:"block",left:`${box.left}px`,top:`${box.top}px`,width:`${box.width}px`,height:`${box.height}px`});const item=metadata(node);if(!item)return;state.tooltip.textContent=`${item.kind} · ${reference(item)}`;style(state.tooltip,{display:"block",left:`${Math.min(x+12,window.innerWidth-430)}px`,top:`${Math.min(y+12,window.innerHeight-48)}px`});}
function renderPanel(item){
  if(!item){state.panel.style.display="none";return;}
  state.panel.replaceChildren();
  const title=document.createElement("div");title.textContent=`${item.kind} · ${reference(item)}`;title.style.fontWeight="700";state.panel.append(title);
  const path=document.createElement("div");path.textContent=`Source: ${item.path}`;path.style.marginTop="6px";state.panel.append(path);
  if(item.usages?.length){const usage=document.createElement("div");usage.textContent=`Uso: ${item.usages.map(entry=>`${entry.path}:${entry.line}`).join(" → ")}`;usage.style.marginTop="4px";state.panel.append(usage);}
  const copy=document.createElement("button");copy.type="button";copy.textContent="Copy agent context";style(copy,{marginTop:"10px",border:"0",borderRadius:"6px",padding:"7px 9px",background:"#38bdf8",color:"#082f49",font:"700 12px system-ui",cursor:"pointer"});copy.addEventListener("click",()=>{const usage=item.usages?.length?` Usage: ${item.usages.map(entry=>`${entry.path}:${entry.line}`).join(" → ")}.`:"";const value=`Dowe source selected: ${item.kind} at ${reference(item)}.${usage} Modify this selected view node.`;navigator.clipboard?.writeText(value);copy.textContent="Copied";setTimeout(()=>copy.textContent="Copy agent context",1200);});state.panel.append(copy);
  state.panel.style.display="block";
}
function showSelected(node){const item=metadata(node);if(!item)return;state.selected=item;renderPanel(item);fetch("/_dowe/dev/inspector-selection",{method:"POST",headers:{"content-type":"application/json"},body:JSON.stringify({node:item})}).catch(()=>{});}
function select(node){showSelected(node);const box=node.getBoundingClientRect();style(state.overlay,{display:"block",left:`${box.left}px`,top:`${box.top}px`,width:`${box.width}px`,height:`${box.height}px`,borderColor:"#fbbf24",background:"#fbbf2422"});}
function toggle(value){ensureUi();state.enabled=value===undefined?!state.enabled:value;state.button.setAttribute("aria-pressed",String(state.enabled));state.button.textContent=state.enabled?"Dowe inspect on":"Dowe inspect";if(!state.enabled){clearHover();state.panel.style.display="none";} }
async function loadManifest(){try{const response=await fetch("/_dowe/dev/inspector.json",{cache:"no-store"});if(!response.ok)return;state.manifest=await response.json();ensureUi();}catch(error){}}
window.__doweInspectorRefresh=loadManifest;
loadManifest();
