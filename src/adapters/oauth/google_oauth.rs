use crate::application::errors::AppError;
use crate::application::ports::auth_provider::{AuthProvider, ExternalProfile};
use crate::application::types::{AccessToken, AuthCode, PKCEVerifier};
use crate::config::Config;
use async_trait::async_trait;
use oauth2::basic::*;
use oauth2::url::Url;
use oauth2::{
    AuthUrl, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet, PkceCodeChallenge, RedirectUrl, RevocationUrl, Scope,
    StandardRevocableToken, TokenResponse, TokenUrl,
};

type OauthClient = Client<
    BasicErrorResponse,
    BasicTokenResponse,
    BasicTokenIntrospectionResponse,
    StandardRevocableToken,
    BasicRevocationErrorResponse,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointSet,
>;

#[derive(Clone)]
pub struct GoogleOAuthProvider {
    oauth: OauthClient,
    http: reqwest::Client,
}

impl GoogleOAuthProvider {
    pub fn new(config: Config) -> Self {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");
        let client_id = ClientId::new(config.oauth_client_id);
        let client_secret = ClientSecret::new(config.oauth_client_secret);
        let redirect_url = RedirectUrl::new(config.oauth_redirect_url).expect("invalid redirect url");
        let oauth = Self::build_google_oauth_client(client_id, client_secret, redirect_url);
        Self { oauth, http: http_client }
    }

    fn build_google_oauth_client(client_id: ClientId, client_secret: ClientSecret, redirect_url: RedirectUrl) -> OauthClient {
        let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string()).expect("invalid auth url");
        let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string()).expect("invalid token url");

        BasicClient::new(client_id)
            .set_client_secret(client_secret)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(redirect_url)
            .set_revocation_url(RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string()).expect("Invalid revocation endpoint URL"))
    }
}

#[async_trait]
impl AuthProvider for GoogleOAuthProvider {
    fn name(&self) -> &'static str {
        "google"
    }

    async fn exchange_code(&self, code: AuthCode, pkce_verifier: PKCEVerifier) -> Result<AccessToken, AppError> {
        let token = self
            .oauth
            .exchange_code(code.into())
            .set_pkce_verifier(pkce_verifier.into())
            .request_async(&self.http)
            .await?;
        Ok(AccessToken(token.access_token().secret().to_string()))
    }

    async fn fetch_profile(&self, access_token: &AccessToken) -> Result<ExternalProfile, AppError> {
        let resp = self
            .http
            .get("https://openidconnect.googleapis.com/v1/userinfo")
            .bearer_auth(access_token.as_ref())
            .send()
            .await?;

        #[derive(serde::Deserialize)]
        struct GUser {
            sub: String,
            email: String,
        }
        let g: GUser = resp.json().await?;

        Ok(ExternalProfile { sub: g.sub, email: g.email })
    }

    async fn login_url(&self, pkce_challenge: PkceCodeChallenge) -> (Url, CsrfToken) {
        self.oauth
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".to_string()))
            .add_scope(Scope::new("email".to_string()))
            .add_scope(Scope::new("profile".to_string()))
            .set_pkce_challenge(pkce_challenge)
            .url()
    }
}
