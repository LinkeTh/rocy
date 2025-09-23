use oauth2::{AuthorizationCode, PkceCodeVerifier};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Copy, Clone, sqlx::Type, PartialEq, Hash, Eq)]
#[sqlx(transparent)]
pub struct ImageId(pub i32);

#[derive(Deserialize, Debug, Serialize, sqlx::FromRow, Copy, Clone, sqlx::Type, PartialEq, Hash, Eq)]
#[sqlx(transparent)]
pub struct UserId(pub i32);

#[derive(Deserialize, Debug, Serialize, Eq, sqlx::FromRow, Clone, sqlx::Type, PartialEq, Hash)]
#[sqlx(transparent)]
pub struct SessionId(pub String);

#[derive(Deserialize, Serialize, Debug, Eq, sqlx::FromRow, Clone, sqlx::Type, PartialEq, Hash)]
#[sqlx(transparent)]
pub struct Email(pub String);

#[derive(Deserialize, Serialize, Debug, Eq, sqlx::FromRow, Clone, sqlx::Type, PartialEq, Hash)]
#[sqlx(transparent)]
pub struct Provider(pub String);

#[derive(Deserialize, Serialize, Debug, Eq, sqlx::FromRow, Clone, sqlx::Type, PartialEq, Hash)]
#[sqlx(transparent)]
pub struct Sub(pub String);

pub struct PKCEChallenge(pub String);

pub struct PKCEVerifier(pub String);

pub struct CSRFToken(pub String);

pub struct AuthCode(pub String);

pub struct AccessToken(pub String);

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for CSRFToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl AsRef<str> for PKCEVerifier {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}
#[derive(Deserialize, Serialize, Debug, sqlx::FromRow, Clone)]
pub struct SessionUser {
    pub user_id: UserId,
    pub email: Email,
}

impl Into<AuthorizationCode> for AuthCode {
    fn into(self) -> AuthorizationCode {
        AuthorizationCode::new(self.0)
    }
}

impl Into<PkceCodeVerifier> for PKCEVerifier {
    fn into(self) -> PkceCodeVerifier {
        PkceCodeVerifier::new(self.0)
    }
}

impl AsRef<i32> for UserId {
    fn as_ref(&self) -> &i32 {
        &self.0
    }
}

impl AsRef<str> for SessionId {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<SessionId> for String {
    fn from(value: SessionId) -> Self {
        value.0
    }
}
