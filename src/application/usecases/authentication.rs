use crate::application::errors::AppError;
use crate::application::ports::auth_provider::AuthProvider;
use crate::application::ports::session_repository::SessionRepository;
use crate::application::ports::user_repository::UserRepository;
use crate::application::service::session_service::{generate_session_id, SessionService};
use crate::application::service::user_service::UserService;
use crate::application::types::{AuthCode, CSRFToken, Email, PKCEVerifier, Provider, SessionId, Sub};
use oauth2::url::Url;
use oauth2::PkceCodeChallenge;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::error;

type OauthLoginResponse = (PKCEVerifier, CSRFToken, Url);

#[derive(Clone)]
pub struct Authentication<S: SessionRepository, D: UserRepository, R: AuthProvider> {
    session: Arc<SessionService<S>>,
    user: Arc<UserService<D>>,
    auth_provider: Arc<R>,
}
impl<S: SessionRepository, D: UserRepository, R: AuthProvider> Authentication<S, D, R> {
    pub fn new(session_repo: Arc<SessionService<S>>, user_repo: Arc<UserService<D>>, auth_provider: Arc<R>) -> Self {
        Self {
            session: session_repo,
            user: user_repo,
            auth_provider,
        }
    }

    pub async fn login(&self) -> OauthLoginResponse {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (auth_url, csrf_token) = self.auth_provider.login_url(pkce_challenge).await;

        (
            PKCEVerifier(pkce_verifier.secret().clone()),
            CSRFToken(csrf_token.secret().clone()),
            auth_url,
        )
    }

    pub async fn logout(&self, session_id: &SessionId) {
        if let Err(e) = self.session.delete_session(session_id).await {
            error!(?e, "failed to delete session");
        }
    }

    pub async fn update_session(&self, pkce_verifier: PKCEVerifier, code: AuthCode, ttl: Duration) -> Result<SessionId, AppError> {
        let token = self.auth_provider.exchange_code(code, pkce_verifier).await?;
        let oauth_user = self.auth_provider.fetch_profile(&token).await?;

        let user_id = self
            .user
            .create_or_update_user(
                &Provider(self.auth_provider.name().to_string()),
                &Sub(oauth_user.sub),
                &Email(oauth_user.email),
            )
            .await?;

        let session_id = generate_session_id();
        let expires_at = SystemTime::now() + ttl;
        self.session.create_session(&user_id, &session_id, expires_at).await?;

        Ok(session_id)
    }
}
