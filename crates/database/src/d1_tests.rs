#[cfg(test)]
mod tests {
    use super::{D1Client, D1TableSchema, d1_error};
    use crate::{DatabaseTransactionInsert, StoreError};
    use axum::Router;
    use axum::extract::State;
    use axum::http::HeaderMap;
    use axum::routing::post;
    use serde_json::{Value, json};
    use std::sync::{Arc, Mutex};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn sends_prepared_d1_queries_with_compound_filters() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let state = requests.clone();
        let router = Router::new()
            .route(
                "/query",
                post(
                    |State(requests): State<Arc<Mutex<Vec<Value>>>>,
                     headers: HeaderMap,
                     axum::Json(body): axum::Json<Value>| async move {
                        assert_eq!(
                            headers
                                .get("authorization")
                                .and_then(|value| value.to_str().ok()),
                            Some("Bearer secret")
                        );
                        requests.lock().expect("requests").push(body);
                        axum::Json(json!({
                            "success": true,
                            "result": [{
                                "success": true,
                                "results": [],
                                "meta": { "changes": 1 }
                            }],
                            "errors": []
                        }))
                    },
                ),
            )
            .with_state(state);
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let client = D1Client::for_endpoint(format!("http://{addr}/query"), "secret".to_string());

        let updated = client
            .update(
                "blogs",
                &[
                    ("id".to_string(), json!("blog-1")),
                    ("ownerId".to_string(), json!("user-1")),
                ],
                json!({ "title": "Updated" }),
                true,
            )
            .await
            .expect("update");

        assert_eq!(updated, json!({ "changed": 1 }));
        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 1);
        assert_eq!(
            requests[0]["sql"],
            "UPDATE \"blogs\" SET \"title\" = ?1 WHERE \"id\" = ?2 AND \"ownerId\" = ?3"
        );
        assert_eq!(
            requests[0]["params"],
            json!(["Updated", "blog-1", "user-1"])
        );
        drop(requests);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[tokio::test]
    async fn sends_bound_parameters_for_d1_query() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let state = requests.clone();
        let router = Router::new()
            .route(
                "/query",
                post(
                    |State(requests): State<Arc<Mutex<Vec<Value>>>>,
                     axum::Json(body): axum::Json<Value>| async move {
                        requests.lock().expect("requests").push(body);
                        axum::Json(json!({
                            "success": true,
                            "result": [{
                                "success": true,
                                "results": [{ "name": "alt-arrow-down" }],
                                "meta": { "changes": 0 }
                            }],
                            "errors": []
                        }))
                    },
                ),
            )
            .with_state(state);
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let client = D1Client::for_endpoint(format!("http://{addr}/query"), "secret".to_string());

        let rows = client
            .query_with_params(
                "SELECT name FROM icons WHERE category = ?1 AND style = ?2",
                &[json!("arrows"), json!("linear")],
            )
            .await
            .expect("rows");

        assert_eq!(rows[0]["name"], "alt-arrow-down");
        let requests = requests.lock().expect("requests");
        assert_eq!(requests[0]["params"], json!(["arrows", "linear"]));
        drop(requests);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[tokio::test]
    async fn sends_atomic_batches_and_decodes_entity_values() {
        let requests = Arc::new(Mutex::new(Vec::<Value>::new()));
        let state = requests.clone();
        let router = Router::new()
            .route(
                "/query",
                post(
                    |State(requests): State<Arc<Mutex<Vec<Value>>>>,
                     axum::Json(body): axum::Json<Value>| async move {
                        requests.lock().expect("requests").push(body);
                        axum::Json(json!({
                            "success": true,
                            "result": [
                                {
                                    "success": true,
                                    "results": [{
                                        "id": "blog-1",
                                        "published": 1,
                                        "metadata": "{\"tags\":[\"dowe\"]}"
                                    }],
                                    "meta": { "changes": 1 }
                                },
                                {
                                    "success": true,
                                    "results": [{ "fingerprint": "seed-1" }],
                                    "meta": { "changes": 1 }
                                }
                            ],
                            "errors": []
                        }))
                    },
                ),
            )
            .with_state(state);
        let listener = TcpListener::bind(("127.0.0.1", 0)).await.expect("bind");
        let addr = listener.local_addr().expect("address");
        let server = tokio::spawn(async move { axum::serve(listener, router).await });
        let mut client =
            D1Client::for_endpoint(format!("http://{addr}/query"), "secret".to_string());
        client.config.schema.push(D1TableSchema {
            table: "blogs".to_string(),
            bool_fields: vec!["published".to_string()],
            json_fields: vec!["metadata".to_string()],
        });
        let result = client
            .transaction(&[
                DatabaseTransactionInsert {
                    table: "blogs".to_string(),
                    value: json!({
                        "id": "blog-1",
                        "published": true,
                        "metadata": { "tags": ["dowe"] }
                    }),
                },
                DatabaseTransactionInsert {
                    table: "_dowe_seeders".to_string(),
                    value: json!({ "fingerprint": "seed-1" }),
                },
            ])
            .await
            .expect("transaction");

        assert_eq!(result[0]["published"], json!(true));
        assert_eq!(result[0]["metadata"], json!({ "tags": ["dowe"] }));
        let requests = requests.lock().expect("requests");
        assert_eq!(requests.len(), 1);
        assert_eq!(requests[0]["batch"].as_array().map(Vec::len), Some(2));
        let first_params = requests[0]["batch"][0]["params"]
            .as_array()
            .expect("params");
        assert!(first_params.contains(&json!(1)));
        assert!(first_params.contains(&json!("{\"tags\":[\"dowe\"]}")));
        drop(requests);
        drop(client);
        server.abort();
        let _ = server.await;
    }

    #[test]
    fn classifies_d1_constraint_and_query_errors() {
        assert!(matches!(
            d1_error("UNIQUE constraint failed: users.email".to_string()),
            StoreError::AlreadyExists(_)
        ));
        assert!(matches!(
            d1_error("no such table: blogs".to_string()),
            StoreError::InvalidQuery(_)
        ));
    }
}
