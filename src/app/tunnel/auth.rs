use std::fmt;

use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use super::TunnelSessionId;

const SIGNING_SECRET_BYTES: usize = 32;
const TOKEN_TTL_SECONDS: i64 = 12 * 60 * 60;
const TUNNEL_AUTH_SUBJECT: &str = "tunnel-user";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActiveTunnel {
    host: String,
    session_id: TunnelSessionId,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TunnelSigningSecret([u8; SIGNING_SECRET_BYTES]);

#[derive(Clone)]
pub struct TunnelAuth {
    secret: TunnelSigningSecret,
}

#[derive(Clone, PartialEq, Eq)]
pub struct TunnelAuthToken(String);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TunnelAuthPurpose {
    WebApi,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TunnelAuthClaims {
    #[serde(rename = "sub")]
    pub subject: String,
    pub purpose: TunnelAuthPurpose,
    #[serde(rename = "iat")]
    pub issued_at: i64,
    #[serde(rename = "exp")]
    pub expires_at: i64,
    pub active_tunnel_host: String,
    pub active_tunnel_session_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TunnelAuthIssue<'a> {
    pub config_password: &'a str,
    pub password_attempt: &'a str,
    pub active_tunnel: &'a ActiveTunnel,
    pub now: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TunnelAuthValidation<'a> {
    pub config_password: &'a str,
    pub authorization_header: Option<&'a str>,
    pub active_tunnel: &'a ActiveTunnel,
    pub now: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TunnelAuthValidationResult {
    Bypassed,
    Authorized(TunnelAuthClaims),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TunnelAuthError {
    SecretGenerationFailed,
    TokenSignFailed,
    TokenRejected,
    TokenExpired,
    AuthorizationRejected,
    StaleHost,
    StaleSession,
    ClockOutOfRange,
}

impl ActiveTunnel {
    pub fn new(host: impl Into<String>, session_id: TunnelSessionId) -> Self {
        Self {
            host: host.into(),
            session_id,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn session_id(&self) -> &TunnelSessionId {
        &self.session_id
    }
}

impl TunnelSigningSecret {
    pub const fn from_bytes(bytes: [u8; SIGNING_SECRET_BYTES]) -> Self {
        Self(bytes)
    }

    pub fn generate() -> Result<Self, TunnelAuthError> {
        let mut bytes = [0_u8; SIGNING_SECRET_BYTES];
        getrandom::fill(&mut bytes).map_err(|_error| TunnelAuthError::SecretGenerationFailed)?;
        Ok(Self(bytes))
    }

    const fn as_bytes(&self) -> &[u8; SIGNING_SECRET_BYTES] {
        &self.0
    }
}

impl fmt::Debug for TunnelSigningSecret {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TunnelSigningSecret(<redacted>)")
    }
}

impl TunnelAuth {
    pub fn new_per_run() -> Result<Self, TunnelAuthError> {
        Ok(Self {
            secret: TunnelSigningSecret::generate()?,
        })
    }

    pub const fn from_secret(secret: TunnelSigningSecret) -> Self {
        Self { secret }
    }

    pub fn issue_token(
        &self,
        request: TunnelAuthIssue<'_>,
    ) -> Result<Option<TunnelAuthToken>, TunnelAuthError> {
        if request.config_password.is_empty() || request.password_attempt != request.config_password
        {
            return Ok(None);
        }
        let issued_at = request.now.timestamp();
        let expires_at = issued_at
            .checked_add(TOKEN_TTL_SECONDS)
            .ok_or(TunnelAuthError::ClockOutOfRange)?;
        let claims = TunnelAuthClaims {
            subject: TUNNEL_AUTH_SUBJECT.to_string(),
            purpose: TunnelAuthPurpose::WebApi,
            issued_at,
            expires_at,
            active_tunnel_host: request.active_tunnel.host.clone(),
            active_tunnel_session_id: request.active_tunnel.session_id.as_str().to_string(),
        };
        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map(Some)
        .map(|token| token.map(TunnelAuthToken))
        .map_err(|_error| TunnelAuthError::TokenSignFailed)
    }

    pub fn validate_authorization(
        &self,
        request: TunnelAuthValidation<'_>,
    ) -> Result<TunnelAuthValidationResult, TunnelAuthError> {
        if request.config_password.is_empty() {
            return Ok(TunnelAuthValidationResult::Bypassed);
        }
        let token = bearer_token(request.authorization_header)?;
        let claims = self.decode_claims(token)?;
        validate_claims(&claims, request.active_tunnel, request.now)?;
        Ok(TunnelAuthValidationResult::Authorized(claims))
    }

    fn decode_claims(&self, token: &str) -> Result<TunnelAuthClaims, TunnelAuthError> {
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = false;
        validation.set_required_spec_claims(&["exp", "iat", "sub"]);
        decode::<TunnelAuthClaims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &validation,
        )
        .map(|data| data.claims)
        .map_err(|_error| TunnelAuthError::TokenRejected)
    }
}

impl TunnelAuthToken {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for TunnelAuthToken {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("TunnelAuthToken(<redacted>)")
    }
}

impl fmt::Display for TunnelAuthError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SecretGenerationFailed => {
                formatter.write_str("tunnel auth secret generation failed")
            }
            Self::TokenSignFailed => formatter.write_str("tunnel auth token signing failed"),
            Self::TokenRejected => formatter.write_str("tunnel auth token rejected"),
            Self::TokenExpired => formatter.write_str("tunnel auth token expired"),
            Self::AuthorizationRejected => formatter.write_str("authorization header rejected"),
            Self::StaleHost => formatter.write_str("tunnel auth token host is stale"),
            Self::StaleSession => formatter.write_str("tunnel auth token session is stale"),
            Self::ClockOutOfRange => formatter.write_str("tunnel auth timestamp is out of range"),
        }
    }
}

impl std::error::Error for TunnelAuthError {}

fn bearer_token(authorization_header: Option<&str>) -> Result<&str, TunnelAuthError> {
    let Some(value) = authorization_header else {
        return Err(TunnelAuthError::AuthorizationRejected);
    };
    let Some(token) = value.strip_prefix("Bearer ") else {
        return Err(TunnelAuthError::AuthorizationRejected);
    };
    if token.is_empty() {
        return Err(TunnelAuthError::AuthorizationRejected);
    }
    Ok(token)
}

fn validate_claims(
    claims: &TunnelAuthClaims,
    active_tunnel: &ActiveTunnel,
    now: DateTime<Utc>,
) -> Result<(), TunnelAuthError> {
    let now_timestamp = now.timestamp();
    if claims.expires_at <= now_timestamp {
        return Err(TunnelAuthError::TokenExpired);
    }
    if claims.issued_at > now_timestamp || claims.subject != TUNNEL_AUTH_SUBJECT {
        return Err(TunnelAuthError::TokenRejected);
    }
    match claims.purpose {
        TunnelAuthPurpose::WebApi => {}
    }
    if claims.active_tunnel_host != active_tunnel.host {
        return Err(TunnelAuthError::StaleHost);
    }
    if claims.active_tunnel_session_id != active_tunnel.session_id.as_str() {
        return Err(TunnelAuthError::StaleSession);
    }
    Ok(())
}
