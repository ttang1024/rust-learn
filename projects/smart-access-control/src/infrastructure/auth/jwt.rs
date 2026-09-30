//! HS256 JWT access tokens and opaque refresh tokens.

use chrono::Duration;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::secrets::{random_secret, sha256_hex};

use crate::{
    application::{AccessClaims, BoxError, IssuedAccessToken, TokenIssuer},
    config::JwtSecret,
    domain::{Administrator, AdministratorId, Timestamp},
};

const ISSUER: &str = "smart-access-control";
const AUDIENCE: &str = "smart-access-control-admin-api";

/// The JWT payload. Standard claim names, so any JWT tool can inspect it.
#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    role: String,
    iat: i64,
    exp: i64,
    iss: String,
    aud: String,
}

pub struct JwtTokenIssuer {
    encoding: EncodingKey,
    decoding: DecodingKey,
    validation: Validation,
    access_ttl: Duration,
}

impl JwtTokenIssuer {
    pub fn new(secret: &JwtSecret, access_ttl: Duration) -> Self {
        // Accept exactly one algorithm. Letting the token header choose is
        // the classic JWT flaw (`alg: none`, or RSA/HMAC key confusion).
        let mut validation = Validation::new(Algorithm::HS256);
        validation.set_issuer(&[ISSUER]);
        validation.set_audience(&[AUDIENCE]);
        validation.required_spec_claims =
            ["exp", "iat", "sub", "iss", "aud"].map(String::from).into();
        // Expiry is checked below against the injected clock, not the
        // library's system clock, so tests are deterministic.
        validation.validate_exp = false;

        Self {
            encoding: EncodingKey::from_secret(secret.expose()),
            decoding: DecodingKey::from_secret(secret.expose()),
            validation,
            access_ttl,
        }
    }
}

impl TokenIssuer for JwtTokenIssuer {
    fn issue_access_token(
        &self,
        admin: &Administrator,
        now: Timestamp,
    ) -> Result<IssuedAccessToken, BoxError> {
        let expires_at = now + self.access_ttl;
        let claims = Claims {
            sub: admin.id().as_uuid(),
            role: admin.role().as_str().to_owned(),
            iat: now.timestamp(),
            exp: expires_at.timestamp(),
            iss: ISSUER.to_owned(),
            aud: AUDIENCE.to_owned(),
        };
        let token = jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &self.encoding)?;
        Ok(IssuedAccessToken { token, expires_at })
    }

    fn verify_access_token(&self, token: &str, now: Timestamp) -> Option<AccessClaims> {
        let claims = jsonwebtoken::decode::<Claims>(token, &self.decoding, &self.validation)
            .ok()?
            .claims;
        let expires_at = Timestamp::from_timestamp(claims.exp, 0)?;
        if now >= expires_at {
            return None;
        }
        Some(AccessClaims {
            admin_id: AdministratorId::from_uuid(claims.sub),
            role: claims.role.parse().ok()?,
            expires_at,
        })
    }

    /// 32 random bytes from the OS, hex-encoded (64 characters).
    fn generate_refresh_token(&self) -> Result<String, BoxError> {
        random_secret()
    }

    /// SHA-256: see `secrets` for why a fast hash is right here.
    fn hash_refresh_token(&self, token: &str) -> String {
        sha256_hex(token)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::*;
    use crate::domain::{PasswordHash, Role, Username};

    fn now() -> Timestamp {
        Utc.with_ymd_and_hms(2026, 9, 30, 9, 0, 0).unwrap()
    }

    fn issuer(secret: &str) -> JwtTokenIssuer {
        JwtTokenIssuer::new(&JwtSecret::new(secret).unwrap(), Duration::minutes(15))
    }

    const SECRET: &str = "test-secret-test-secret-test-secret!";

    fn admin() -> Administrator {
        Administrator::create(
            Username::parse("ops").unwrap(),
            PasswordHash::new("x".into()),
            Role::Viewer,
            now(),
        )
    }

    #[test]
    fn issued_token_verifies_until_expiry() {
        let jwt = issuer(SECRET);
        let admin = admin();
        let issued = jwt.issue_access_token(&admin, now()).unwrap();

        let claims = jwt.verify_access_token(&issued.token, now()).unwrap();
        assert_eq!(claims.admin_id, admin.id());
        assert_eq!(claims.role, Role::Viewer);
        assert_eq!(claims.expires_at, now() + Duration::minutes(15));

        let almost = issued.expires_at - Duration::seconds(1);
        assert!(jwt.verify_access_token(&issued.token, almost).is_some());
        assert!(
            jwt.verify_access_token(&issued.token, issued.expires_at)
                .is_none()
        );
    }

    #[test]
    fn token_signed_with_another_secret_is_rejected() {
        let token = issuer("another-secret-another-secret-12345")
            .issue_access_token(&admin(), now())
            .unwrap()
            .token;
        assert!(issuer(SECRET).verify_access_token(&token, now()).is_none());
    }

    #[test]
    fn tampered_payload_is_rejected() {
        let jwt = issuer(SECRET);
        let token = jwt.issue_access_token(&admin(), now()).unwrap().token;
        // Swap the payload for one claiming the admin role, keep the signature.
        let parts: Vec<&str> = token.split('.').collect();
        let forged_payload = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &Claims {
                sub: admin().id().as_uuid(),
                role: "admin".into(),
                iat: now().timestamp(),
                exp: (now() + Duration::days(365)).timestamp(),
                iss: ISSUER.into(),
                aud: AUDIENCE.into(),
            },
            &EncodingKey::from_secret(b"attacker-does-not-know-the-secret"),
        )
        .unwrap();
        let forged = format!(
            "{}.{}.{}",
            parts[0],
            forged_payload.split('.').nth(1).unwrap(),
            parts[2]
        );
        assert!(jwt.verify_access_token(&forged, now()).is_none());
    }

    #[test]
    fn unsigned_alg_none_token_is_rejected() {
        // {"alg":"none","typ":"JWT"} with valid-looking claims and no signature.
        let header = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
        let jwt = issuer(SECRET);
        let real = jwt.issue_access_token(&admin(), now()).unwrap().token;
        let payload = real.split('.').nth(1).unwrap();
        assert!(
            jwt.verify_access_token(&format!("{header}.{payload}."), now())
                .is_none()
        );
    }

    #[test]
    fn wrong_audience_is_rejected() {
        let claims = Claims {
            sub: admin().id().as_uuid(),
            role: "viewer".into(),
            iat: now().timestamp(),
            exp: (now() + Duration::minutes(5)).timestamp(),
            iss: ISSUER.into(),
            aud: "some-other-service".into(),
        };
        let token = jsonwebtoken::encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(SECRET.as_bytes()),
        )
        .unwrap();
        assert!(issuer(SECRET).verify_access_token(&token, now()).is_none());
    }

    #[test]
    fn garbage_is_rejected() {
        let jwt = issuer(SECRET);
        for token in ["", "abc", "a.b.c", "Bearer x"] {
            assert!(jwt.verify_access_token(token, now()).is_none(), "{token:?}");
        }
    }

    #[test]
    fn refresh_tokens_are_random_and_hashed() {
        let jwt = issuer(SECRET);
        let a = jwt.generate_refresh_token().unwrap();
        let b = jwt.generate_refresh_token().unwrap();
        assert_eq!(a.len(), 64);
        assert_ne!(a, b);

        let hash = jwt.hash_refresh_token(&a);
        assert_eq!(hash.len(), 64);
        assert_ne!(hash, a);
        assert_eq!(hash, jwt.hash_refresh_token(&a), "hashing is deterministic");
        // Known SHA-256 test vector.
        assert_eq!(
            jwt.hash_refresh_token("abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
