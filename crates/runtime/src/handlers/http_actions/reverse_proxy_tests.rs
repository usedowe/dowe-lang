#[cfg(test)]
mod reverse_proxy_tests {
    use super::*;
    use axum::Router;
    use axum::body::to_bytes;

    async fn echo(method: Method, uri: Uri, headers: HeaderMap, body: Bytes) -> Response {
        let mut response = json_response(
            StatusCode::CREATED,
            json!({
                "method": method.as_str(),
                "path": uri.path(),
                "query": uri.query(),
                "body": String::from_utf8_lossy(&body),
                "forwardedHost": headers
                    .get("x-forwarded-host")
                    .and_then(|value| value.to_str().ok()),
                "requestHeader": headers
                    .get("x-request-id")
                    .and_then(|value| value.to_str().ok()),
                "connectionForwarded": headers.contains_key("connection"),
            }),
        );
        response
            .headers_mut()
            .insert("x-upstream", HeaderValue::from_static("ready"));
        response
            .headers_mut()
            .insert("connection", HeaderValue::from_static("close"));
        response
    }

    async fn reverse_proxy_outcome(
        status: StatusCode,
        content_length: Option<&'static str>,
    ) -> ReverseProxyRequestOutcome {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        let server = tokio::spawn(async move {
            let upstream = Router::new().fallback(move || async move {
                let body = match content_length {
                    Some(_) => Body::from("upstream"),
                    None => Body::from_stream(futures_util::stream::once(async {
                        Ok::<_, std::io::Error>(Bytes::from_static(b"upstream"))
                    })),
                };
                let mut response = Response::new(body);
                *response.status_mut() = status;
                if let Some(content_length) = content_length {
                    response
                        .headers_mut()
                        .insert("content-length", HeaderValue::from_static(content_length));
                }
                response
            });
            axum::serve(listener, upstream).await.expect("upstream");
        });
        let outcome = reverse_proxy_request(
            &format!("http://{address}"),
            &HttpMethod::Get,
            "/status",
            None,
            &HeaderMap::new(),
            &Bytes::new(),
        )
        .await;
        server.abort();
        outcome
    }

    #[tokio::test]
    async fn reverse_proxy_preserves_request_and_filters_hop_headers() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move {
            axum::serve(listener, Router::new().fallback(echo))
                .await
                .expect("upstream");
        });
        let mut headers = HeaderMap::new();
        headers.insert("host", HeaderValue::from_static("app.dowe.cloud"));
        headers.insert("x-request-id", HeaderValue::from_static("req_1"));
        headers.insert("connection", HeaderValue::from_static("keep-alive"));

        let outcome = reverse_proxy_request(
            &format!("http://{addr}"),
            &HttpMethod::Post,
            "/api/items",
            Some("page=2"),
            &headers,
            &Bytes::from_static(b"payload"),
        )
        .await;
        let ReverseProxyRequestOutcome::Upstream {
            response,
            status,
            bytes_out,
        } = outcome
        else {
            panic!("expected upstream headers");
        };

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(status, 201);
        assert!(bytes_out > 0);
        assert_eq!(
            response.headers().get("x-upstream"),
            Some(&HeaderValue::from_static("ready"))
        );
        assert!(!response.headers().contains_key("connection"));
        let body = to_bytes(response.into_body(), 4096).await.expect("body");
        let body: Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(body["method"], "POST");
        assert_eq!(body["path"], "/api/items");
        assert_eq!(body["query"], "page=2");
        assert_eq!(body["body"], "payload");
        assert_eq!(body["forwardedHost"], "app.dowe.cloud");
        assert_eq!(body["requestHeader"], "req_1");
        assert_eq!(body["connectionForwarded"], false);
        server.abort();
    }

    #[tokio::test]
    async fn reverse_proxy_treats_upstream_error_headers_as_real_upstream_responses() {
        for status in [StatusCode::NOT_FOUND, StatusCode::INTERNAL_SERVER_ERROR] {
            let outcome = reverse_proxy_outcome(status, Some("8")).await;
            let ReverseProxyRequestOutcome::Upstream {
                response,
                status: observed_status,
                bytes_out,
            } = outcome
            else {
                panic!("expected upstream headers");
            };

            assert_eq!(response.status(), status);
            assert_eq!(observed_status, u64::from(status.as_u16()));
            assert_eq!(bytes_out, 8);
        }
    }

    #[tokio::test]
    async fn reverse_proxy_uses_zero_bytes_out_without_an_upstream_content_length() {
        let outcome = reverse_proxy_outcome(StatusCode::OK, None).await;
        let ReverseProxyRequestOutcome::Upstream { bytes_out, .. } = outcome else {
            panic!("expected upstream headers");
        };

        assert_eq!(bytes_out, 0);
    }

    #[tokio::test]
    async fn reverse_proxy_treats_invalid_and_unreachable_upstreams_as_local_failures() {
        let invalid = reverse_proxy_request(
            "file:///tmp/proxy",
            &HttpMethod::Get,
            "/status",
            None,
            &HeaderMap::new(),
            &Bytes::new(),
        )
        .await;
        assert!(matches!(
            invalid,
            ReverseProxyRequestOutcome::Local(response) if response.status() == StatusCode::BAD_GATEWAY
        ));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("listener");
        let address = listener.local_addr().expect("address");
        drop(listener);
        let unreachable = reverse_proxy_request(
            &format!("http://{address}"),
            &HttpMethod::Get,
            "/status",
            None,
            &HeaderMap::new(),
            &Bytes::new(),
        )
        .await;
        assert!(matches!(
            unreachable,
            ReverseProxyRequestOutcome::Local(response) if response.status() == StatusCode::BAD_GATEWAY
        ));
    }

    #[test]
    fn reverse_proxy_round_robin_filters_unavailable_runtimes() {
        let pool = json!([
            { "upstreamUrl": "http://runtime-a:8080", "status": "ready" },
            { "upstreamUrl": "http://runtime-b:8080", "status": "loading" },
            { "url": "http://runtime-c:8080", "status": "ready", "enabled": true },
            { "url": "http://runtime-d:8080", "status": "ready", "enabled": false }
        ]);
        let key = "round-robin-test.dowe.cloud";

        assert_eq!(
            select_reverse_proxy_upstream(&pool, ReverseProxyStrategy::RoundRobin, key).as_deref(),
            Some("http://runtime-a:8080")
        );
        assert_eq!(
            select_reverse_proxy_upstream(&pool, ReverseProxyStrategy::RoundRobin, key).as_deref(),
            Some("http://runtime-c:8080")
        );
        assert_eq!(
            select_reverse_proxy_upstream(&pool, ReverseProxyStrategy::RoundRobin, key).as_deref(),
            Some("http://runtime-a:8080")
        );
    }

    #[test]
    fn reverse_proxy_uses_temporary_redirect_for_state_fallbacks() {
        let response = reverse_proxy_redirect("https://cloud.dowe.dev/loading");

        assert_eq!(response.status(), StatusCode::TEMPORARY_REDIRECT);
        assert_eq!(
            response.headers().get(LOCATION),
            Some(&HeaderValue::from_static("https://cloud.dowe.dev/loading"))
        );
        assert_eq!(
            reverse_proxy_redirect("file:///tmp/loading").status(),
            StatusCode::BAD_GATEWAY
        );
    }

    #[test]
    fn reverse_proxy_enriches_dowe_task_telemetry() {
        let mut args = json!({ "event": { "projectId": "project_1" } });
        enrich_reverse_proxy_telemetry(&mut args, 201, "POST", "/api/items", 4.5, 7, 19);

        assert_eq!(args["event"]["projectId"], "project_1");
        assert_eq!(args["event"]["status"], 201);
        assert_eq!(args["event"]["method"], "POST");
        assert_eq!(args["event"]["path"], "/api/items");
        assert_eq!(args["event"]["latencyMs"], 4.5);
        assert_eq!(args["event"]["bytesIn"], 7);
        assert_eq!(args["event"]["bytesOut"], 19);
    }
}

