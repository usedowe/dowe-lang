const DOWE_ICON_SVG="__DOWE_INSPECTOR_ICON_SVG__";
const SERVER_INSPECTOR_URL="__DOWE_SERVER_INSPECTOR_URL__";
const BRAND={navy:"#1f3a5f",green:"#6bc670",ink:"#102a15",panel:"#10243b",panelSoft:"#173452",line:"#33506f",muted:"#b8c7d8",white:"#f8fbff"};
const state={enabled:false,hidden:false,panelOpen:false,detailsOpen:false,selected:null,selectedElement:null,manifest:null,overlay:null,tooltip:null,panel:null,button:null,activeTab:"inspect",runtimeRegistered:false,runtimeView:null,routeQuery:"",focusRoutes:false,viewportBadge:null,drag:{active:false,moved:false,suppressClick:false,offsetX:0,offsetY:0,startX:0,startY:0}};
function nodeAt(target){return target instanceof Element?target.closest("[data-dowe-node]"):null;}
function metadata(node){return state.manifest?.nodes?.find(item=>item.id===node?.dataset.doweNode)||null;}
function reference(item){return item?item.path+":"+item.startLine+"-"+item.endLine:"";}
function style(element,values){for(const [name,value] of Object.entries(values))element.style[name]=value;}
function clamp(value,min,max){return Math.min(Math.max(value,min),max);}
function doweIcon(size=22){
  const height=Math.round(size*145/137);
  return DOWE_ICON_SVG.replace(/^<\?xml[^>]*>\s*/i,"").replace(/^<!DOCTYPE[^>]*>\s*/i,"").replace("<svg ",'<svg viewBox="0 0 137 145" ').replace('width="137px"','width="'+size+'px"').replace('height="145px"','height="'+height+'px"');
}
function positionButton(left,top){
  if(!state.button)return;
  const rect=state.button.getBoundingClientRect();
  const margin=12;
  const maxLeft=Math.max(margin,window.innerWidth-rect.width-margin);
  const maxTop=Math.max(margin,window.innerHeight-rect.height-margin);
  style(state.button,{left:String(clamp(left,margin,maxLeft))+"px",top:String(clamp(top,margin,maxTop))+"px",right:"auto",bottom:"auto"});
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
function sessionValue(key){try{return sessionStorage.getItem(key);}catch(error){return null;}}
function setSessionValue(key,value){try{sessionStorage.setItem(key,value);}catch(error){}}
function inspectorWasEnabled(){return sessionValue("dowe-inspector-enabled")==="true";}
function inspectorWasHidden(){return sessionValue("dowe-inspector-hidden")==="true";}
function inspectorPanelWasOpen(){return sessionValue("dowe-inspector-panel-open")==="true";}
function persistInspectorEnabled(){setSessionValue("dowe-inspector-enabled",String(state.enabled));}
function persistInspectorHidden(){setSessionValue("dowe-inspector-hidden",String(state.hidden));}
function persistPanelOpen(){setSessionValue("dowe-inspector-panel-open",String(state.panelOpen));}
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
  positionPanel();
  event.preventDefault();
}
function endButtonDrag(event){
  if(!state.drag.active)return;
  try{state.button.releasePointerCapture?.(event.pointerId);}catch(error){}
  state.drag.active=false;
  state.drag.suppressClick=state.drag.moved;
  state.button.style.cursor="grab";
  if(state.drag.moved)saveButtonPosition();
  positionPanel();
}
function keepButtonInViewport(){
  if(!state.button)return;
  const rect=state.button.getBoundingClientRect();
  const margin=12;
  const left=Number.isFinite(parseFloat(state.button.style.left))?parseFloat(state.button.style.left):margin;
  const top=parseFloat(state.button.style.top);
  const maxLeft=Math.max(margin,window.innerWidth-rect.width-margin);
  const maxTop=Math.max(margin,window.innerHeight-rect.height-margin);
  const values={left:String(clamp(left,margin,maxLeft))+"px",right:"auto"};
  if(Number.isFinite(top)){
    values.top=String(clamp(top,margin,maxTop))+"px";
    values.bottom="auto";
  }else{
    const bottom=parseFloat(state.button.style.bottom);
    values.top="auto";
    values.bottom=String(clamp(Number.isFinite(bottom)?bottom:margin,margin,Math.max(margin,window.innerHeight-rect.height-margin)))+"px";
  }
  style(state.button,values);
  positionPanel();
}
function positionPanel(){
  if(!state.panel||!state.button||state.hidden||!state.panelOpen)return;
  state.panel.style.display="block";
  const orb=state.button.getBoundingClientRect();
  const panel=state.panel.getBoundingClientRect();
  const margin=12;
  let left=orb.left;
  let top=orb.top-panel.height-10;
  if(top<margin)top=orb.bottom+10;
  if(top+panel.height>window.innerHeight-margin)top=Math.max(margin,window.innerHeight-panel.height-margin);
  if(left+panel.width>window.innerWidth-margin)left=orb.right-panel.width;
  if(left<margin)left=margin;
  style(state.panel,{left:String(Math.round(left))+"px",top:String(Math.round(top))+"px",right:"auto",bottom:"auto"});
}
function appendText(parent,value,styles={}){const element=document.createElement("div");element.textContent=value;style(element,styles);parent.append(element);return element;}
function appendButton(parent,label,handler,styles={}){const button=document.createElement("button");button.type="button";button.textContent=label;style(button,{border:"1px solid "+BRAND.line,borderRadius:"6px",padding:"6px 8px",background:BRAND.panelSoft,color:BRAND.white,font:"700 11px system-ui",cursor:"pointer",...styles});button.addEventListener("click",event=>{event.stopPropagation();handler(event,button);});parent.append(button);return button;}
function solarIcon(name){
  const paths={
    inspect:'<path d="M17 7.82959L18.6965 9.35641C20.239 10.7447 21.0103 11.4389 21.0103 12.3296C21.0103 13.2203 20.239 13.9145 18.6965 15.3028L17 16.8296M13.9868 5L12 12.4149L10.0132 19.8297M7.00005 7.82959L5.30358 9.35641C3.76102 10.7447 2.98975 11.4389 2.98975 12.3296C2.98975 13.2203 3.76102 13.9145 5.30358 15.3028L7.00005 16.8296" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>',
    routes:'<path d="M20 19H7.5C5.567 19 4 17.433 4 15.5C4 13.567 5.567 12 7.5 12H16.5C18.433 12 20 10.433 20 8.5C20 6.567 18.433 5 16.5 5H8M18 21L20 19L18 17M4 5a2 2 0 1 0 4 0a2 2 0 1 0 -4 0" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>'
  };
  paths.server='<rect x="3" y="4" width="18" height="6" rx="2" fill="none" stroke="currentColor" stroke-width="1.5"/><rect x="3" y="14" width="18" height="6" rx="2" fill="none" stroke="currentColor" stroke-width="1.5"/><path d="M7 7h.01M7 17h.01" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>';
  return '<svg viewBox="0 0 24 24" width="17" height="17" aria-hidden="true" focusable="false">'+paths[name]+'</svg>';
}
function appendIconButton(parent,name,label,handler,styles={}){
  const button=document.createElement("button");
  button.type="button";
  button.innerHTML=solarIcon(name);
  button.title=label;
  button.setAttribute("aria-label",label);
  style(button,{display:"inline-flex",alignItems:"center",justifyContent:"center",width:"29px",height:"29px",padding:0,border:"1px solid "+BRAND.line,borderRadius:"6px",background:BRAND.panelSoft,color:BRAND.white,cursor:"pointer",...styles});
  button.addEventListener("click",event=>{event.stopPropagation();handler(event,button);});
  parent.append(button);
  return button;
}
function openServerInspector(){
  if(!SERVER_INSPECTOR_URL)return;
  const popup=window.open(SERVER_INSPECTOR_URL,"_blank","noopener,noreferrer");
  if(popup)popup.opener=null;
}
function appendRows(parent,rows,empty){
  if(!rows?.length){appendText(parent,empty,{color:BRAND.muted});return;}
  for(const row of rows){
    const element=document.createElement("div");
    style(element,{display:"grid",gridTemplateColumns:"minmax(84px,1fr) minmax(0,2fr)",gap:"8px",padding:"5px 0",borderBottom:"1px solid "+BRAND.line});
    appendText(element,row.label,{color:BRAND.muted,overflowWrap:"anywhere"});
    appendText(element,row.value,{color:BRAND.white,overflowWrap:"anywhere",whiteSpace:"pre-wrap"});
    parent.append(element);
  }
}
function appendDisclosure(parent,label,count,open,render){
  const details=document.createElement("details");
  details.open=open;
  style(details,{borderTop:"1px solid "+BRAND.line,padding:"8px 0 0"});
  const summary=document.createElement("summary");
  summary.textContent=label+(count===undefined?"":"  ·  "+count);
  style(summary,{cursor:"pointer",fontWeight:"700",color:BRAND.green,outline:"none"});
  details.append(summary);
  const body=document.createElement("div");
  style(body,{paddingTop:"5px"});
  render(body);
  details.append(body);
  parent.append(details);
  return details;
}
function registerRuntimeBridge(){
  if(state.runtimeRegistered||typeof window.__doweRegisterRuntimeCapability!=="function")return;
  try{
    window.__doweRegisterRuntimeCapability("dowe-inspector",api=>{
      state.runtimeView=api.getActiveView?.()||null;
      return {setActiveView(view){state.runtimeView=view;if(state.enabled&&state.activeTab==="inspect")renderPanel(state.selected);}};
    });
    state.runtimeRegistered=true;
  }catch(error){}
}
function runtimeSignalValue(signal){
  const value=state.runtimeView?.state?.[signal.id];
  if(value===undefined)return "undefined";
  try{const serialized=JSON.stringify(value);return serialized.length<=160?serialized:serialized.slice(0,160)+"…";}catch(error){return String(value);}
}
function ancestorChain(node){
  const chain=[];
  let current=node;
  while(current instanceof Element){
    const item=metadata(current);
    if(item)chain.unshift({element:current,item});
    current=current.parentElement;
  }
  return chain;
}
function breakpointForWidth(width){
  const breakpoints=state.manifest?.breakpoints||[];
  return breakpoints.reduce((current,item)=>item.minWidth<=width?item:current,breakpoints[0]||{name:"xs",minWidth:0});
}
function refreshViewport(){
  const value=window.innerWidth+" × "+window.innerHeight+" · "+breakpointForWidth(window.innerWidth).name;
  if(state.viewportBadge)state.viewportBadge.textContent=value;
  positionPanel();
}
function agentContext(item){
  if(!item)return "";
  const hierarchy=ancestorChain(state.selectedElement).map(entry=>entry.item.kind+" at "+reference(entry.item)).join(" > ");
  const usage=item.usages?.length?" Usage: "+item.usages.map(entry=>entry.path+":"+entry.line).join(" → ")+".":"";
  return "Dowe source selected: "+item.kind+" at "+reference(item)+". Hierarchy: "+(hierarchy||"none")+"."+usage+" Modify this selected view node.";
}
function copyContext(feedback){
  const value=agentContext(state.selected);
  if(!value)return;
  const done=()=>{if(feedback){const old=feedback.textContent;feedback.textContent="Copied";setTimeout(()=>feedback.textContent=old,1200);}};
  if(navigator.clipboard?.writeText)navigator.clipboard.writeText(value).then(done).catch(()=>{});else done();
}
function renderInspect(parent,item){
  if(!item){
    appendText(parent,"Select a marked view element to inspect its source and runtime context.",{color:BRAND.muted});
    appendText(parent,"Alt/Option + Shift + I toggles inspection. Escape stops it.",{marginTop:"8px",color:BRAND.muted,fontSize:"12px"});
    return;
  }
  const selectedHeader=document.createElement("div");
  style(selectedHeader,{display:"flex",alignItems:"center",gap:"8px"});
  appendText(selectedHeader,item.kind,{fontWeight:"800",fontSize:"15px",color:BRAND.white,flex:"1",minWidth:0,overflow:"hidden",textOverflow:"ellipsis",whiteSpace:"nowrap"});
  const selectedActions=document.createElement("div");
  style(selectedActions,{display:"flex",alignItems:"center",gap:"6px",flexShrink:"0"});
  appendButton(selectedActions,"Copy",(event,button)=>copyContext(button),{padding:"5px 7px",background:BRAND.green,color:BRAND.ink,borderColor:BRAND.green});
  appendButton(selectedActions,state.detailsOpen?"Hide details":"Show details",()=>{state.detailsOpen=!state.detailsOpen;renderPanel(state.selected);},{padding:"5px 7px"});
  selectedHeader.append(selectedActions);
  parent.append(selectedHeader);
  appendText(parent,reference(item),{marginTop:"3px",color:BRAND.muted,fontSize:"12px",overflowWrap:"anywhere"});
  appendText(parent,"Source: "+item.path,{marginTop:"3px",color:BRAND.muted,fontSize:"12px",overflowWrap:"anywhere"});
  if(!state.detailsOpen)return;
  const hierarchy=ancestorChain(state.selectedElement);
  appendDisclosure(parent,"Hierarchy",hierarchy.length,true,body=>{
    if(!hierarchy.length){appendText(body,"No authored ancestors found.",{color:BRAND.muted});return;}
    for(const entry of hierarchy){
      const active=entry.item.id===item.id;
      appendButton(body,entry.item.kind+"  ·  "+reference(entry.item),()=>select(entry.element),{display:"block",width:"100%",textAlign:"left",border:0,borderLeft:"3px solid "+(active?BRAND.green:BRAND.line),borderRadius:0,padding:"5px 7px",background:active?"#25476a":"transparent",color:active?BRAND.white:BRAND.muted,font:"12px system-ui"});
    }
  });
  appendDisclosure(parent,"Props",item.props?.length||0,true,body=>{
    appendRows(body,(item.props||[]).map(prop=>({label:prop.name,value:prop.value})),"No authored props.");
    if(state.selectedElement?.className)appendRows(body,[{label:"web classes",value:String(state.selectedElement.className)}],"");
  });
  appendDisclosure(parent,"State and signals",item.signals?.length||0,true,body=>{
    appendRows(body,(item.signals||[]).map(signal=>({label:signal.name,value:signal.scope+" · "+signal.storage+" · initial "+signal.initial+" · current "+runtimeSignalValue(signal)})),"No signals declared in this view scope.");
    const bound=[...state.selectedElement?.querySelectorAll?.("[data-dowe-bind]")||[]].map(element=>element.dataset.doweBind).filter(Boolean);
    if(bound.length)appendRows(body,[{label:"bound in subtree",value:[...new Set(bound)].join(", ")}],"");
  });
  const actions=(item.actions||[]).filter(action=>!action.name.startsWith("$dowe:"));
  appendDisclosure(parent,"Actions",actions.length,true,body=>appendRows(body,actions.map(action=>({label:action.name,value:action.kind+(action.detail?" · "+action.detail:"")})),"No named actions declared in this view scope."));
}
function routeGroup(path){
  const segments=String(path||"/").split("/").filter(Boolean);
  return segments.length>1?"/"+segments.slice(0,2).join("/"):"/";
}
function renderRoutes(parent){
  appendText(parent,"Routes",{fontWeight:"800",fontSize:"15px",color:BRAND.white});
  const search=document.createElement("input");
  search.type="search";
  search.placeholder="Filter paths or source files";
  search.value=state.routeQuery;
  search.setAttribute("aria-label","Filter Dowe routes");
  style(search,{display:"block",boxSizing:"border-box",width:"100%",marginTop:"8px",padding:"8px 9px",border:"1px solid "+BRAND.line,borderRadius:"6px",background:"#0c1d30",color:BRAND.white,font:"13px system-ui",outline:"none"});
  search.addEventListener("input",()=>{state.routeQuery=search.value;renderPanel(state.selected);state.focusRoutes=true;});
  parent.append(search);
  const routes=state.manifest?.routes||[];
  const query=state.routeQuery.trim().toLowerCase();
  const filtered=routes.filter(route=>!query||[route.path,route.page?.source,...(route.layouts||[]).map(layout=>layout.source)].join(" ").toLowerCase().includes(query)).sort((a,b)=>{const current=location.pathname;return (a.path===current?-1:0)-(b.path===current?-1:0)||String(a.path).localeCompare(String(b.path));});
  appendText(parent,filtered.length+" of "+routes.length+" routes",{marginTop:"8px",color:BRAND.muted,fontSize:"12px"});
  if(!filtered.length){appendText(parent,"No matching routes.",{marginTop:"8px",color:BRAND.muted});return;}
  const groups=new Map();
  for(const route of filtered.slice(0,80)){const key=routeGroup(route.path);if(!groups.has(key))groups.set(key,[]);groups.get(key).push(route);}
  for(const [group,groupRoutes] of groups){
    appendText(parent,group,{marginTop:"10px",marginBottom:"3px",fontWeight:"700",color:BRAND.green,fontSize:"11px",textTransform:"uppercase",letterSpacing:".05em"});
    for(const route of groupRoutes){
      const current=route.path===location.pathname;
      const row=document.createElement("div");
      style(row,{display:"flex",alignItems:"center",gap:"8px",minHeight:"31px",padding:"4px 0",borderBottom:"1px solid "+BRAND.line});
      appendButton(row,route.path,()=>location.assign(route.path||"/"),{flex:"1",minWidth:0,textAlign:"left",border:0,padding:"3px 0",background:"transparent",color:current?BRAND.green:"#dbe9f7",font:"600 12px system-ui",overflow:"hidden",textOverflow:"ellipsis",whiteSpace:"nowrap"});
      appendText(row,route.page?.source?String(route.page.source).split("/").pop():"",{maxWidth:"45%",overflow:"hidden",textOverflow:"ellipsis",whiteSpace:"nowrap",color:BRAND.muted,fontSize:"11px"});
      if(current)appendText(row,"current",{color:BRAND.green,fontSize:"10px",fontWeight:"700"});
      parent.append(row);
    }
  }
  if(filtered.length>80)appendText(parent,"Showing the first 80 matches. Refine the search to see more.",{marginTop:"8px",color:BRAND.muted,fontSize:"11px"});
  if(state.focusRoutes){state.focusRoutes=false;setTimeout(()=>{search.focus();search.setSelectionRange(search.value.length,search.value.length);},0);}
}
function renderPanel(item){
  if(!state.panel)return;
  state.panel.replaceChildren();
  const header=document.createElement("div");
  style(header,{display:"flex",alignItems:"center",gap:"5px",minWidth:0});
  appendText(header,"Dowe Devtools",{fontWeight:"800",fontSize:"14px",color:BRAND.white,flex:"1",minWidth:0,overflow:"hidden",textOverflow:"ellipsis",whiteSpace:"nowrap"});
  const routeCount=state.manifest?.routes?.length||0;
  for(const [id,name,label] of [["inspect","inspect","Inspect"],["routes","routes","Routes"+(routeCount?" · "+routeCount:"")]]){
    const tab=appendIconButton(header,name,label,()=>{state.activeTab=id;if(id==="routes")state.focusRoutes=true;renderPanel(state.selected);},{background:state.activeTab===id?BRAND.green:BRAND.panelSoft,color:state.activeTab===id?BRAND.ink:BRAND.white,borderColor:state.activeTab===id?BRAND.green:BRAND.line,flexShrink:"0"});
    tab.setAttribute("role","tab");
    tab.setAttribute("aria-selected",String(state.activeTab===id));
  }
  if(SERVER_INSPECTOR_URL)appendIconButton(header,"server","Open Dowe Server Inspector",openServerInspector,{background:BRAND.panelSoft,color:BRAND.white,borderColor:BRAND.line,flexShrink:"0"});
  appendButton(header,state.enabled?"Inspect on":"Inspect off",()=>toggleInspect(),{padding:"5px 7px",fontSize:"10px",background:state.enabled?BRAND.green:BRAND.panelSoft,color:state.enabled?BRAND.ink:BRAND.white,borderColor:state.enabled?BRAND.green:BRAND.line});
  const hide=appendButton(header,"Hide",()=>hideDevtools(),{padding:"5px 7px",fontSize:"10px"});
  hide.setAttribute("aria-label","Hide Dowe Devtools for this session");
  const close=appendButton(header,"×",()=>togglePanel(false),{padding:"3px 7px",fontSize:"15px",lineHeight:"1"});
  close.setAttribute("aria-label","Close Dowe Devtools panel");
  state.panel.append(header);
  const viewport=document.createElement("div");
  style(viewport,{display:"flex",alignItems:"center",justifyContent:"space-between",gap:"8px",marginTop:"7px",fontSize:"11px",color:BRAND.muted});
  appendText(viewport,"Viewport",{fontWeight:"700",color:BRAND.green});
  state.viewportBadge=appendText(viewport,"",{whiteSpace:"nowrap"});
  state.panel.append(viewport);
  refreshViewport();
  const content=document.createElement("div");
  if(state.activeTab==="inspect")renderInspect(content,item);else renderRoutes(content);
  state.panel.append(content);
  appendText(state.panel,"Shortcuts  ⌥⇧D hide/show  ·  ⌥⇧I inspect  ·  ⌥⇧C copy  ·  ⌥⇧R routes  ·  Esc close",{marginTop:"12px",paddingTop:"8px",borderTop:"1px solid "+BRAND.line,color:BRAND.muted,fontSize:"10px",lineHeight:"1.4"});
  state.panel.style.display=state.panelOpen&&!state.hidden?"block":"none";
  positionPanel();
}
function ensureUi(){
  if(state.button)return;
  state.button=document.createElement("button");
  state.button.id="dowe-inspector-ui";
  state.button.type="button";
  state.button.innerHTML=doweIcon(24);
  state.button.setAttribute("aria-pressed","false");
  state.button.setAttribute("aria-label","Open Dowe Devtools");
  state.button.title="Dowe Devtools · drag to move";
  style(state.button,{position:"fixed",left:"16px",right:"auto",bottom:"16px",zIndex:"2147483646",display:state.hidden?"none":"flex",alignItems:"center",justifyContent:"center",width:"42px",height:"42px",padding:0,border:"2px solid "+BRAND.navy,borderRadius:"50%",background:"#fff",cursor:"grab",touchAction:"none",userSelect:"none",boxShadow:"0 7px 24px #102a1555"});
  state.button.addEventListener("click",()=>{if(state.drag.suppressClick){state.drag.suppressClick=false;return;}if(!state.enabled)toggleInspect(true);togglePanel();});
  state.button.addEventListener("pointerdown",beginButtonDrag);
  state.button.addEventListener("pointermove",moveButton);
  state.button.addEventListener("pointerup",endButtonDrag);
  state.button.addEventListener("pointercancel",endButtonDrag);
  document.body.append(state.button);
  restoreButtonPosition();
  state.overlay=document.createElement("div");
  style(state.overlay,{position:"fixed",zIndex:"2147483644",pointerEvents:"none",border:"2px solid "+BRAND.green,background:"#6bc67022",display:"none",boxSizing:"border-box"});
  document.body.append(state.overlay);
  state.tooltip=document.createElement("div");
  style(state.tooltip,{position:"fixed",zIndex:"2147483645",pointerEvents:"none",display:"none",maxWidth:"420px",padding:"6px 8px",borderRadius:"6px",background:BRAND.navy,color:BRAND.white,font:"12px/1.35 system-ui",boxShadow:"0 6px 20px #0005"});
  document.body.append(state.tooltip);
  state.panel=document.createElement("div");
  state.panel.id="dowe-inspector-panel";
  state.panel.tabIndex=-1;
  style(state.panel,{position:"fixed",zIndex:"2147483645",display:"none",boxSizing:"border-box",width:"min(470px,calc(100vw - 24px))",maxHeight:"min(78vh,680px)",overflow:"auto",padding:"13px",border:"1px solid "+BRAND.line,borderRadius:"12px",background:BRAND.panel,color:BRAND.white,font:"13px/1.45 system-ui",boxShadow:"0 14px 44px #102a1588"});
  document.body.append(state.panel);
  document.addEventListener("mousemove",event=>{if(!state.enabled||state.hidden)return;const node=nodeAt(event.target);if(!node){clearHover();return;}showHover(node,event.clientX,event.clientY);});
  document.addEventListener("click",event=>{if(!state.enabled||state.hidden)return;const node=nodeAt(event.target);if(!node||event.target.closest("#dowe-inspector-ui,#dowe-inspector-panel"))return;event.preventDefault();event.stopPropagation();select(node);},true);
  document.addEventListener("keydown",event=>{
    const editing=event.target instanceof HTMLInputElement||event.target instanceof HTMLTextAreaElement||event.target?.isContentEditable;
    if(event.altKey&&event.shiftKey&&event.code==="KeyD"){event.preventDefault();state.hidden?showDevtools():hideDevtools();return;}
    if(editing)return;
    if(event.altKey&&event.shiftKey&&event.code==="KeyI"){event.preventDefault();if(state.hidden)showDevtools();toggleInspect();}
    else if(event.altKey&&event.shiftKey&&event.code==="KeyC"){event.preventDefault();copyContext();}
    else if(event.altKey&&event.shiftKey&&event.code==="KeyR"){event.preventDefault();if(state.hidden)showDevtools();state.activeTab="routes";state.focusRoutes=true;togglePanel(true);renderPanel(state.selected);}
    else if(event.key==="Escape"){if(state.enabled)toggleInspect(false);else togglePanel(false);}
  });
  window.addEventListener("resize",()=>{keepButtonInViewport();refreshViewport();});
}
function clearHover(){if(state.overlay)state.overlay.style.display="none";if(state.tooltip)state.tooltip.style.display="none";}
function showHover(node,x,y){
  const box=node.getBoundingClientRect();
  style(state.overlay,{display:"block",left:String(box.left)+"px",top:String(box.top)+"px",width:String(box.width)+"px",height:String(box.height)+"px",borderColor:BRAND.green,background:"#6bc67022"});
  const item=metadata(node);
  if(!item)return;
  state.tooltip.textContent=item.kind+" · "+reference(item);
  style(state.tooltip,{display:"block",left:String(clamp(x+12,8,Math.max(8,window.innerWidth-430)))+"px",top:String(clamp(y+12,8,Math.max(8,window.innerHeight-48)))+"px"});
}
function showSelected(node){
  const item=metadata(node);
  if(!item)return;
  state.selected=item;
  state.selectedElement=node;
  state.detailsOpen=false;
  state.panelOpen=true;
  persistPanelOpen();
  renderPanel(item);
  fetch("/_dowe/dev/inspector-selection",{method:"POST",headers:{"content-type":"application/json"},body:JSON.stringify({node:item})}).catch(()=>{});
}
function select(node){
  showSelected(node);
  const box=node.getBoundingClientRect();
  style(state.overlay,{display:"block",left:String(box.left)+"px",top:String(box.top)+"px",width:String(box.width)+"px",height:String(box.height)+"px",borderColor:BRAND.green,background:"#6bc67033"});
}
function toggleInspect(value){
  ensureUi();
  state.enabled=value===undefined?!state.enabled:value;
  persistInspectorEnabled();
  state.button.setAttribute("aria-pressed",String(state.enabled));
  state.button.style.borderColor=state.enabled?BRAND.green:BRAND.navy;
  state.button.title=state.enabled?"Dowe inspection on · drag to move":"Dowe Devtools · drag to move";
  if(!state.enabled)clearHover();
  renderPanel(state.selected);
}
function toggle(value){toggleInspect(value);}
function togglePanel(value){
  ensureUi();
  state.panelOpen=value===undefined?!state.panelOpen:value;
  persistPanelOpen();
  renderPanel(state.selected);
  if(state.panelOpen)state.panel.focus?.();
}
function hideDevtools(){
  ensureUi();
  state.hidden=true;
  state.panelOpen=false;
  persistInspectorHidden();
  persistPanelOpen();
  clearHover();
  state.button.style.display="none";
  state.panel.style.display="none";
}
function showDevtools(){
  ensureUi();
  state.hidden=false;
  persistInspectorHidden();
  state.button.style.display="flex";
  renderPanel(state.selected);
}
async function loadManifest(){
  registerRuntimeBridge();
  try{
    const response=await fetch("/_dowe/dev/inspector.json",{cache:"no-store"});
    if(!response.ok)return;
    state.manifest=await response.json();
    ensureUi();
    state.hidden=inspectorWasHidden();
    state.panelOpen=inspectorPanelWasOpen()&&!state.hidden;
    if(inspectorWasEnabled())toggleInspect(true);else renderPanel(state.selected);
    if(state.hidden)hideDevtools();else if(state.panelOpen)renderPanel(state.selected);
  }catch(error){}
}
window.__doweInspectorRefresh=loadManifest;
loadManifest();
