#[cfg(test)]
mod tests {
    use super::{
        CryptoError, JwtValidationOptions, aes_128_ctr, cenc_aes_128_ctr, decrypt_jwe_dir_a256gcm,
        encrypt_jwe_dir_a256gcm, sign_jws_hs256, verify_jws_hs256,
    };
    use serde_json::json;

    const SECRET: &str = "01234567890123456789012345678901";

    #[test]
    fn applies_aes_128_ctr_with_hex_key_and_iv() {
        let encrypted = aes_128_ctr(
            b"hello segment",
            "00000000000000000000000000000000",
            "00000000000000000000000000000000",
        )
        .expect("encrypt");
        let decrypted = aes_128_ctr(
            &encrypted,
            "00000000000000000000000000000000",
            "00000000000000000000000000000000",
        )
        .expect("decrypt");

        assert_eq!(decrypted, b"hello segment");
    }

    #[test]
    fn applies_cenc_aes_ctr_with_clear_and_encrypted_subsamples() {
        let input = b"clearencryptedtail".to_vec();
        let encrypted = cenc_aes_128_ctr(
            &input,
            "00000000000000000000000000000000",
            "0000000000000000",
            &[(5, 9), (0, 4)],
        )
        .expect("encrypt");

        assert_eq!(&encrypted[..5], b"clear");
        assert_ne!(&encrypted[5..14], b"encrypted");

        let decrypted = cenc_aes_128_ctr(
            &encrypted,
            "00000000000000000000000000000000",
            "0000000000000000",
            &[(5, 9), (0, 4)],
        )
        .expect("decrypt");

        assert_eq!(decrypted, input);
    }

    #[test]
    fn signs_and_verifies_hs256() {
        let token = sign_jws_hs256(&json!({"sub":"user-1","exp":200}), SECRET).expect("sign");
        let claims = verify_jws_hs256(
            &token,
            SECRET,
            &JwtValidationOptions {
                now: 100,
                ..JwtValidationOptions::default()
            },
        )
        .expect("verify");

        assert_eq!(claims["sub"], "user-1");
    }

    #[test]
    fn rejects_invalid_signature_and_none_algorithm() {
        let token = sign_jws_hs256(&json!({"sub":"user-1"}), SECRET).expect("sign");
        assert_eq!(
            verify_jws_hs256(
                &format!("{token}x"),
                SECRET,
                &JwtValidationOptions::default()
            )
            .expect_err("signature"),
            CryptoError::InvalidSignature
        );

        let none = "eyJhbGciOiJub25lIn0.eyJzdWIiOiJ1c2VyIn0.";
        assert_eq!(
            verify_jws_hs256(none, SECRET, &JwtValidationOptions::default()).expect_err("none"),
            CryptoError::UnsafeAlgorithm
        );
    }

    #[test]
    fn validates_temporal_and_registered_claims() {
        let expired = sign_jws_hs256(&json!({"exp":100}), SECRET).expect("sign");
        assert_eq!(
            verify_jws_hs256(
                &expired,
                SECRET,
                &JwtValidationOptions {
                    now: 101,
                    ..JwtValidationOptions::default()
                }
            )
            .expect_err("expired"),
            CryptoError::Expired
        );

        let future = sign_jws_hs256(&json!({"nbf":200}), SECRET).expect("sign");
        assert_eq!(
            verify_jws_hs256(
                &future,
                SECRET,
                &JwtValidationOptions {
                    now: 100,
                    ..JwtValidationOptions::default()
                }
            )
            .expect_err("nbf"),
            CryptoError::NotYetValid
        );

        let claims = sign_jws_hs256(
            &json!({"iss":"issuer","aud":["app"],"sub":"user-1"}),
            SECRET,
        )
        .expect("sign");
        verify_jws_hs256(
            &claims,
            SECRET,
            &JwtValidationOptions {
                issuer: Some("issuer".to_string()),
                audience: vec!["app".to_string()],
                required_claims: vec!["sub".to_string()],
                ..JwtValidationOptions::default()
            },
        )
        .expect("registered");
    }

    #[test]
    fn encrypts_and_decrypts_jwe() {
        let token =
            encrypt_jwe_dir_a256gcm(&json!({"sub":"user-1","exp":200}), SECRET).expect("encrypt");
        let claims = decrypt_jwe_dir_a256gcm(
            &token,
            SECRET,
            &JwtValidationOptions {
                now: 100,
                ..JwtValidationOptions::default()
            },
        )
        .expect("decrypt");

        assert_eq!(claims["sub"], "user-1");
        assert!(decrypt_jwe_dir_a256gcm(&token, "bad", &JwtValidationOptions::default()).is_err());
    }

    #[test]
    fn rejects_short_keys() {
        assert_eq!(
            sign_jws_hs256(&json!({"sub":"user"}), "short").expect_err("short"),
            CryptoError::InvalidKey
        );
        assert_eq!(
            encrypt_jwe_dir_a256gcm(&json!({"sub":"user"}), "short").expect_err("short"),
            CryptoError::InvalidKey
        );
    }
}

