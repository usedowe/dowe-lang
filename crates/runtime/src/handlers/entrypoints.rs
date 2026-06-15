pub async fn backend_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;
    server_response(
        &project,
        &project.backend,
        &state.dev_origins,
        method,
        uri.path(),
        headers,
        body,
    )
    .await
}

pub async fn backend_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(route) = project.backend.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    websocket_response(upgrade, project.clone(), route.handlers)
}

pub async fn desktop_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;
    if let Some(server) = &project.desktop_server
        && (server.has_endpoint_path(uri.path())
            || method == Method::OPTIONS && is_preflight(&headers))
    {
        return server_response(
            &project,
            server,
            &state.dev_origins,
            method,
            uri.path(),
            headers,
            body,
        )
        .await;
    }

    if method == Method::GET {
        desktop_static_response(&project, uri.path())
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn desktop_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(server) = &project.desktop_server else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(route) = server.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    websocket_response(upgrade, project.clone(), route.handlers)
}

pub(crate) async fn server_response(
    project: &CompiledProject,
    server: &ServerConfig,
    dev_origins: &[String],
    method: Method,
    path: &str,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if method == Method::OPTIONS && is_preflight(&headers) && server.cors.enabled {
        return cors_preflight_response(server, dev_origins, path, &headers);
    }

    let response = match HttpMethod::from_str(method.as_str()) {
        Ok(method) => match server.find_endpoint(&method, path) {
            Some(matched) => {
                let middleware_context =
                    match execute_middlewares(project, &matched.endpoint.middlewares, &headers) {
                        MiddlewareFlow::Continue(context) => context,
                        MiddlewareFlow::Respond(response) => {
                            return cors_actual_response(
                                &server.cors,
                                dev_origins,
                                &headers,
                                response,
                            );
                        }
                    };
                if !matches!(
                    &matched.endpoint.behavior,
                    EndpointBehavior::HttpProxy(_)
                        | EndpointBehavior::HttpActionJson(_)
                        | EndpointBehavior::AgentResponse(_)
                        | EndpointBehavior::StoreActionJson(_)
                        | EndpointBehavior::KvActionJson(_)
                ) {
                    execute_server_action_with_resolver(&matched.endpoint.action, |reference| {
                        resolve_request_reference(reference, &matched.params, &middleware_context)
                            .map(log_json_text)
                    });
                }

                match matched.endpoint.behavior {
                    EndpointBehavior::StaticText(text) => text_response(StatusCode::OK, text),
                    EndpointBehavior::TextTemplate(text) => text_response(
                        StatusCode::OK,
                        render_text_template(&text, &matched.params, &middleware_context),
                    ),
                    EndpointBehavior::UserGreeting => {
                        let id = matched.params.get("id").cloned().unwrap_or_default();
                        text_response(StatusCode::OK, format!("Hello User {id}!"))
                    }
                    EndpointBehavior::CreatePostJson => created_json_response(&body),
                    EndpointBehavior::HttpProxy(response) => {
                        execute_http_proxy(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::HttpActionJson(response) => {
                        execute_http_action_json(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::AgentResponse(response) => {
                        execute_agent_response(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::StoreInsertJson(insert) => {
                        match execute_store_insert(
                            project,
                            &insert.connection,
                            &insert.table,
                            &insert.value,
                        )
                        .await
                        {
                            Ok(value) => json_response(StatusCode::OK, value),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreQueryJson(query) => {
                        match execute_store_query(project, &query.connection, &query.sql).await {
                            Ok(value) => json_response(StatusCode::OK, value),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreTransactionJson(transaction) => {
                        match execute_store_transaction(&project.root, &transaction) {
                            Ok(value) => json_response(StatusCode::OK, value),
                            Err(error) => store_error_response(error),
                        }
                    }
                    EndpointBehavior::StoreActionJson(response) => {
                        execute_store_action_json(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                    EndpointBehavior::KvActionJson(response) => {
                        execute_kv_action_json(
                            project,
                            &project.root,
                            &matched.endpoint.action,
                            &response,
                            &matched.params,
                            &body,
                        )
                        .await
                    }
                }
            }
            None => {
                if server.has_endpoint_path(path) {
                    StatusCode::METHOD_NOT_ALLOWED.into_response()
                } else {
                    StatusCode::NOT_FOUND.into_response()
                }
            }
        },
        Err(_) => StatusCode::METHOD_NOT_ALLOWED.into_response(),
    };

    cors_actual_response(&server.cors, dev_origins, &headers, response)
}

pub async fn views_handler(State(state): State<DevRuntimeState>, uri: Uri) -> Response {
    if uri.path() == "/_dowe/dev/ws" {
        return StatusCode::BAD_REQUEST.into_response();
    }

    if uri.path() == "/_dowe/dev/client.js" {
        return dev_client_response();
    }

    let project = state.project.read().await;

    if uri.path() == "/design.css" {
        return design_css_response(&project, "web/design.css");
    }

    if uri.path() == "/router.js" {
        return javascript_response(project.web.router_js.clone());
    }

    if uri.path() == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }

    if let Some(response) = font_response(&project, uri.path()) {
        return response;
    }

    if let Some(response) = chunk_response(&project.web, uri.path()) {
        return response;
    }

    if let Some(page) = project
        .web
        .pages
        .iter()
        .find(|page| page.route_path == uri.path())
    {
        return render_page(page);
    }

    StatusCode::NOT_FOUND.into_response()
}

fn desktop_static_response(project: &CompiledProject, path: &str) -> Response {
    if path == "/_dowe/dev/client.js" {
        return dev_client_response();
    }
    if path == "/design.css" {
        return design_css_response(project, "apps/desktop/web/design.css");
    }
    if path == "/router.js" {
        return javascript_response(project.desktop_web.router_js.clone());
    }
    if path == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }
    if let Some(response) = font_response(project, path) {
        return response;
    }
    if let Some(response) = chunk_response(&project.desktop_web, path) {
        return response;
    }
    let Some(page) = project
        .desktop_web
        .pages
        .iter()
        .find(|page| page.route_path == path)
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let html_path = if page.route_path == "/" {
        project.root.join(".dowe/apps/desktop/web/index.html")
    } else {
        let file_name = page.route_path.trim_matches('/').replace('/', "-");
        project
            .root
            .join(".dowe/apps/desktop/web/pages")
            .join(format!("{file_name}.html"))
    };
    match fs::read_to_string(html_path) {
        Ok(html) => Html(inject_dev_client(&html)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
