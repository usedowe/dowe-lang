pub async fn backend_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;

    if uri.path() == "/_dowe/dev/modules/manifest.json" {
        return generated_json_response(&project, "dev/modules/manifest.json");
    }

    if let Some(response) = dev_module_response(&project, uri.path()) {
        return response;
    }
    server_response(
        &project,
        &project.backend,
        &state.dev_origins,
        state.cache_mode,
        method,
        uri.path(),
        uri.query(),
        headers,
        body,
    )
    .await
}

pub async fn backend_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    uri: Uri,
    headers: HeaderMap,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(route) = project.backend.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let params = std::collections::HashMap::new();
    let body = Bytes::new();
    if let MiddlewareFlow::Respond(response) = execute_middlewares(
        &project,
        &project.root,
        &route.middlewares,
        &headers,
        &params,
        &body,
        uri.query(),
        state.cache_mode,
    )
    .await
    {
        return response;
    }
    websocket_response(upgrade, project.clone(), route.handlers, state.cache_mode)
}

pub async fn desktop_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let project = state.project.read().await;
    let Some(server) = &project.desktop_server else {
        return StatusCode::NOT_FOUND.into_response();
    };
    if !server.has_endpoint_path(uri.path())
        && !(method == Method::OPTIONS && is_preflight(&headers))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    server_response(
        &project,
        server,
        &state.dev_origins,
        state.cache_mode,
        method,
        uri.path(),
        uri.query(),
        headers,
        body,
    )
    .await
}

pub async fn desktop_declared_websocket_handler(
    state: DevRuntimeState,
    upgrade: WebSocketUpgrade,
    uri: Uri,
    headers: HeaderMap,
    path: String,
) -> Response {
    let project = state.project.read().await;
    let Some(server) = &project.desktop_server else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(route) = server.find_websocket(&path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let params = std::collections::HashMap::new();
    let body = Bytes::new();
    if let MiddlewareFlow::Respond(response) = execute_middlewares(
        &project,
        &project.root,
        &route.middlewares,
        &headers,
        &params,
        &body,
        uri.query(),
        state.cache_mode,
    )
    .await
    {
        return response;
    }
    websocket_response(upgrade, project.clone(), route.handlers, state.cache_mode)
}

pub(crate) async fn server_response(
    project: &CompiledProject,
    server: &ServerConfig,
    dev_origins: &[String],
    cache_mode: CacheRuntimeMode,
    method: Method,
    path: &str,
    raw_query: Option<&str>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if method == Method::OPTIONS && is_preflight(&headers) && server.cors.enabled {
        return cors_preflight_response(server, dev_origins, path, &headers);
    }

    let response = match HttpMethod::from_str(method.as_str()) {
        Ok(method) => match server.find_endpoint(&method, path) {
            Some(matched) => {
                let middleware_context = match execute_middlewares(
                    project,
                    &project.root,
                    &matched.endpoint.middlewares,
                    &headers,
                    &matched.params,
                    &body,
                    raw_query,
                    cache_mode,
                )
                .await
                {
                    MiddlewareFlow::Continue(context) => context,
                    MiddlewareFlow::Respond(response) => {
                        return cors_actual_response(&server.cors, dev_origins, &headers, response);
                    }
                };
                let uses_simplified_action = matches!(
                    &matched.endpoint.behavior,
                    EndpointBehavior::StaticText(_)
                        | EndpointBehavior::TextTemplate(_)
                        | EndpointBehavior::UserGreeting
                        | EndpointBehavior::CreatePostJson
                );
                let action_result = if uses_simplified_action {
                    execute_simplified_http_action(
                        project,
                        &project.root,
                        &matched.endpoint.action,
                        &matched.params,
                        &body,
                        raw_query,
                        &headers,
                        &middleware_context,
                        cache_mode,
                    )
                    .await
                } else {
                    Ok(())
                };
                if !uses_simplified_action
                    && !matches!(
                        &matched.endpoint.behavior,
                        EndpointBehavior::HttpProxy(_)
                            | EndpointBehavior::HttpReverseProxy(_)
                            | EndpointBehavior::HttpBytes(_)
                            | EndpointBehavior::HttpActionJson(_)
                            | EndpointBehavior::AgentResponse(_)
                            | EndpointBehavior::StoreActionJson(_)
                            | EndpointBehavior::KvActionJson(_)
                            | EndpointBehavior::VectorActionJson(_)
                            | EndpointBehavior::QueueActionJson(_)
                    )
                {
                    crate::background_jobs::launch_task_statements(
                        &project.root,
                        &matched.endpoint.action,
                        cache_mode,
                    );
                    execute_server_action_with_resolver(&matched.endpoint.action, |reference| {
                        resolve_request_reference(reference, &matched.params, &middleware_context)
                            .map(log_json_text)
                    });
                }

                match action_result {
                    Err(error) => json_error(error.status, error.code, error.message),
                    Ok(()) => match matched.endpoint.behavior {
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
                                raw_query,
                                &headers,
                                cache_mode,
                            )
                            .await
                        }
                        EndpointBehavior::HttpReverseProxy(response) => {
                            execute_http_reverse_proxy(
                                project,
                                &project.root,
                                &matched.endpoint.action,
                                &response,
                                &matched.params,
                                &body,
                                raw_query,
                                &headers,
                                &method,
                                path,
                                cache_mode,
                            )
                            .await
                        }
                        EndpointBehavior::HttpBytes(response) => {
                            execute_http_bytes(
                                project,
                                &project.root,
                                &matched.endpoint.action,
                                &response,
                                &matched.params,
                                &body,
                                raw_query,
                                &headers,
                                cache_mode,
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
                                raw_query,
                                &headers,
                                &middleware_context,
                                cache_mode,
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
                                raw_query,
                                &headers,
                                cache_mode,
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
                            match execute_store_query(project, &query.connection, &query.query)
                                .await
                            {
                                Ok(value) => json_response(StatusCode::OK, value),
                                Err(error) => {
                                    log_error(format!(
                                        "Database query failed for `{}`: {error}",
                                        query.connection.database
                                    ));
                                    store_error_response(error)
                                }
                            }
                        }
                        EndpointBehavior::StoreTransactionJson(transaction) => {
                            match execute_store_transaction(project, &transaction).await {
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
                                raw_query,
                                &headers,
                                &middleware_context,
                                cache_mode,
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
                                raw_query,
                                &headers,
                                cache_mode,
                            )
                            .await
                        }
                        EndpointBehavior::VectorActionJson(response) => {
                            execute_vector_action_json(
                                project,
                                &project.root,
                                &matched.endpoint.action,
                                &response,
                                &matched.params,
                                &body,
                                raw_query,
                                &headers,
                                cache_mode,
                            )
                            .await
                        }
                        EndpointBehavior::QueueActionJson(response) => {
                            execute_queue_action_json(
                                project,
                                &project.root,
                                &matched.endpoint.action,
                                &response,
                                &matched.params,
                                &body,
                                raw_query,
                                &headers,
                                cache_mode,
                            )
                            .await
                        }
                    },
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

pub async fn views_handler(
    State(state): State<DevRuntimeState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    if uri.path() == "/_dowe/dev/ws" {
        return StatusCode::BAD_REQUEST.into_response();
    }

    let project = state.project.read().await;
    let inspector_enabled = project
        .web
        .chunks
        .iter()
        .any(|chunk| chunk.inspector.is_some());

    if uri.path() == "/_dowe/dev/client.js" {
        let server_inspector_url = project.server_inspector.as_ref().and_then(|_| {
            state
                .dev_origins
                .last()
                .map(|origin| format!("{origin}/_dowe/dev/server/"))
        });
        return dev_client_response(inspector_enabled, server_inspector_url.as_deref());
    }

    if uri.path() == "/_dowe/dev/inspector-selection" {
        if !inspector_enabled {
            return StatusCode::NOT_FOUND.into_response();
        }
        if method != Method::POST {
            return StatusCode::METHOD_NOT_ALLOWED.into_response();
        }
        return inspector_selection_response(&project, &body);
    }

    if uri.path() == "/_dowe/dev/modules/manifest.json" {
        return generated_json_response(&project, "dev/modules/manifest.json");
    }

    if uri.path() == "/_dowe/dev/inspector.json" {
        if !inspector_enabled {
            return StatusCode::NOT_FOUND.into_response();
        }
        return generated_json_response(&project, "web/inspector.json");
    }

    if let Some(response) = dev_module_response(&project, uri.path()) {
        return response;
    }

    let design_file_name = uri.path().strip_prefix('/').filter(|file_name| {
        project.web.has_design_file_name(file_name)
    });
    if design_file_name.is_some() {
        return cacheable_design_css_response(
            &project,
            &format!("web/{}", project.web.design_file_name()),
            &headers,
            "no-store",
        );
    }

    if let Some(relative_path) = design_css_chunk_relative_path(uri.path()) {
        return cacheable_design_css_response(&project, &relative_path, &headers, "no-store");
    }

    if uri.path() == "/router.js" || uri.path() == format!("/{}", project.web.router_file_name()) {
        return javascript_response(project.web.router_js.clone());
    }

    if uri.path() == "/env.json" {
        return json_response_text(project.environment_config.client_json());
    }

    if uri.path() == "/manifest.json" {
        return generated_json_response(&project, "web/manifest.json");
    }

    if let Some(response) = font_response(&project, uri.path()) {
        return response;
    }

    if let Some(response) = project_asset_response(&project, uri.path(), "no-store") {
        return response;
    }

    if let Some(response) = chunk_response(&project.web, uri.path(), &headers, "no-store") {
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
