#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_pkce_authorization_parameters() {
        let challenge = create_pkce_challenge();
        let url = authorization_url(&challenge).expect("URL");
        let query = url
            .query_pairs()
            .map(|(key, value)| (key.into_owned(), value.into_owned()))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(query.get("client_id").map(String::as_str), Some(CLIENT_ID));
        assert_eq!(
            query.get("redirect_uri").map(String::as_str),
            Some(REDIRECT_URI)
        );
        assert_eq!(
            query.get("code_challenge_method").map(String::as_str),
            Some("S256")
        );
        assert_eq!(query.get("state").map(String::len), Some(22));
        assert_ne!(query.get("code_challenge"), Some(&challenge.verifier));
    }

    #[test]
    fn creates_openrouter_pkce_authorization_parameters() {
            let challenge = create_pkce_challenge();
            let callback = "http://localhost:43123/auth/callback";
            let url = openrouter_authorization_url(&challenge, callback).expect("URL");
            let query = url.query_pairs().map(|(key, value)| (key.into_owned(), value.into_owned())).collect::<std::collections::BTreeMap<_, _>>();
            assert_eq!(url.origin().ascii_serialization(), "https://openrouter.ai");
            assert_eq!(query.get("callback_url").map(String::as_str), Some(callback));
            assert_eq!(query.get("code_challenge_method").map(String::as_str), Some("S256"));
            assert_eq!(query.get("code_challenge").map(String::as_str), Some(challenge.challenge.as_str()));
        }

        #[test]
        fn parses_openrouter_api_key_as_api_key_credential() {
            let credential = parse_openrouter_key(br#"{"key":"sk-or-v1-test"}"#).expect("key");
            assert_eq!(credential, AgentCredential::api_key("sk-or-v1-test"));
        }

        #[test]
        fn rejects_openrouter_response_without_key() {
            let error = parse_openrouter_key(br#"{"key":""}"#).expect_err("missing key");
            assert!(error.to_string().contains("did not include an API key"));
        }

        #[test]
        fn only_expiring_oauth_credentials_need_refresh() {
        let credential = AgentCredential::OAuth {
            access: "access".to_string(),
            refresh: Some("refresh".to_string()),
            expires: Some(now_millis().saturating_add(1_000)),
            env: Default::default(),
        };
        assert!(token_needs_refresh(&credential));
        assert!(!token_needs_refresh(&AgentCredential::api_key("key")));
    }
}
