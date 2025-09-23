use crate::application::errors::AppError;
use crate::application::types::{AccessToken, AuthCode, PKCEVerifier};
use async_trait::async_trait;
use oauth2::url::Url;
use oauth2::{CsrfToken, PkceCodeChallenge};

pub struct ExternalProfile {
    pub sub: String,
    pub email: String,
}

#[async_trait]
pub trait AuthProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn exchange_code(&self, code: AuthCode, pkce_verifier: PKCEVerifier) -> Result<AccessToken, AppError>;

    async fn fetch_profile(&self, access_token: &AccessToken) -> Result<ExternalProfile, AppError>;

    async fn login_url(&self, pkce_challenge: PkceCodeChallenge) -> (Url, CsrfToken);
}
