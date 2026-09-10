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

