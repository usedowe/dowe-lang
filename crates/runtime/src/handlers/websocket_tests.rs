#[cfg(test)]
mod websocket_tests {
    use super::*;

    #[test]
    fn safe_client_payload_removes_provider_metadata() {
        assert_eq!(
            safe_client_payload("delta", &json!({ "model": "private/model", "choices": [] })),
            json!({})
        );
        assert_eq!(
            safe_client_payload(
                "error",
                &json!({ "model": "private/model", "error": { "code": "provider_error", "message": "Try again" } })
            ),
            json!({ "error": { "code": "provider_error", "message": "Try again" } })
        );
    }
}
