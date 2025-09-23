mod errors;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::Request;
use axum::response::Html;
use axum::routing::get_service;
use axum::{
    extract::{Multipart, Query, State}, http::{HeaderMap, StatusCode}, response::{IntoResponse, Redirect},
    routing::{get, post},
    Extension,
    Json,
    Router,
};
use axum_extra::extract::PrivateCookieJar;
use axum_prometheus::PrometheusMetricLayer;
use cookie::time::Duration as TimeDuration;
use cookie::{Cookie, Key};
use oauth2::basic::BasicClient;
use oauth2::basic::*;
use oauth2::{
    AuthUrl, AuthorizationCode, Client, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, RedirectUrl, RevocationUrl, Scope, StandardRevocableToken, TokenResponse,
    TokenUrl,
};
use serde::Deserialize;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use std::{env, net::SocketAddr};
use tokio::signal;
use tracing::{error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
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
use crate::errors::ApiError;
use reqwest::Client as ReqwestClient;
use sqlx::types::chrono::Local;
use tower_http::services::ServeFile;

#[derive(Clone)]
struct AppState {
    db: PgPool,
    ctx: ReqwestClient,
    key: Key,
}
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() {
    init_tracing();

    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    sqlx::migrate!()
        .run(&db)
        .await
        .expect("Failed migrations :(");

    let client_id = env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set");
    let client_secret = env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set");
    let oauth_client = build_google_oauth_client(client_id.clone(), client_secret);
    let ctx = ReqwestClient::new();
    let state = AppState {
        db,
        ctx,
        key: Key::generate(),
    };

    // Metrics layer and handle
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let login_router = Router::new().route("/login", get(google_login));
    let auth_router = Router::new().route("/auth/google/callback", get(google_callback));
    let protected_router = Router::new().route("/protected", get(protected));
    let homepage_router = Router::new()
        .route("/", get(homepage))
        .layer(Extension(client_id));
    let favicon = Router::new().route_service(
        "/favicon.ico",
        get_service(ServeFile::new("assets/favicon.ico")),
    );

    let app = Router::new()
        .merge(favicon)
        .merge(login_router)
        .merge(auth_router)
        .merge(protected_router)
        .merge(homepage_router)
        .route("/healthz", get(health_handler))
        .route("/upload", post(upload_handler))
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(Extension(oauth_client))
        .with_state(state)
        .layer(prometheus_layer);

    let addr: SocketAddr = env::var("BIND_ADDR")
        .expect("BIND_ADDR must be set")
        .parse()
        .expect("Invalid BIND_ADDR");

    info!(%addr, "starting server");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind failed");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(false))
        .init();
}

#[axum::debug_handler]
async fn homepage(Extension(oauth_id): Extension<String>) -> Html<String> {
    Html(format!("<p>Welcome!</p>

    <a href=\"https://accounts.google.com/o/oauth2/v2/auth?scope=openid%20profile%20email&client_id={oauth_id}&response_type=code&redirect_uri=http://localhost:8080/auth/google/callback\">
    Click here to sign into Google!
     </a>"))
}
fn build_google_oauth_client(client_id: String, client_secret: String) -> OauthClient {
    let redirect_url = env::var("OAUTH_REDIRECT_URL").expect("OAUTH_REDIRECT_URL must be set");

    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("invalid auth url");
    let token_url = TokenUrl::new("https://oauth2.googleapis.com/token".to_string())
        .expect("invalid token url");

    BasicClient::new(ClientId::new(client_id))
        .set_client_secret(ClientSecret::new(client_secret))
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(RedirectUrl::new(redirect_url).expect("invalid redirect url"))
        .set_revocation_url(
            RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
                .expect("Invalid revocation endpoint URL"),
        )
}

// #[axum::debug_handler]
async fn health_handler(State(state): State<AppState>, profile: UserProfile) -> impl IntoResponse {
    let db_ok = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await;

    let body = match db_ok {
        Ok(x) => {
            info!(?x, "db check succeed");
            serde_json::json!({
                "status": "ok",
                "user": profile.email,
            })
        }
        Err(e) => {
            error!(?e, "db check failed");
            serde_json::json!({
                       "status": "error",
                       "error": e.to_string(),
                       "user": profile.email,
            })
        }
    };

    (StatusCode::OK, Json(body))
}
async fn upload_handler(
    State(_state): State<AppState>,
    _profile: UserProfile,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> impl IntoResponse {
    // Check content type just for example
    if !headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .map(|ct| ct.to_lowercase().starts_with("multipart/"))
        .unwrap_or(false)
    {
        return (StatusCode::BAD_REQUEST, "Expected multipart/form-data").into_response();
    }

    let upload_dir = env::var("UPLOAD_DIR").unwrap_or_else(|_| "uploads".to_string());
    if let Err(e) = tokio::fs::create_dir_all(&upload_dir).await {
        error!(?e, "failed to create upload dir");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to prepare upload dir",
        )
            .into_response();
    }

    let mut saved_files = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().map(|s| s.to_string());
        let file_name = field
            .file_name()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}.bin", ulid::Ulid::new()));
        let content_type = field.content_type().map(|s| s.to_string());
        let data = match field.bytes().await {
            Ok(b) => b,
            Err(e) => {
                error!(?e, "failed to read multipart field");
                return (StatusCode::BAD_REQUEST, "Failed to read upload").into_response();
            }
        };

        // naive image-type filter (optional)
        if let Some(ct) = &content_type {
            if !ct.starts_with("image/") {
                return (
                    StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    "Only images are allowed",
                )
                    .into_response();
            }
        }

        let path = format!("{}/{}", upload_dir, sanitize_filename(&file_name));
        if let Err(e) = tokio::fs::write(&path, &data).await {
            error!(?e, path, "failed to save file");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file").into_response();
        }

        saved_files.push(serde_json::json!({
            "field": name,
            "file_name": file_name,
            "content_type": content_type,
            "size": data.len()
        }));
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({ "saved": saved_files })),
    )
        .into_response()
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[axum::debug_handler]
async fn google_login(
    State(_state): State<AppState>,
    Extension(oauth): Extension<OauthClient>,
) -> impl IntoResponse {
    let (auth_url, _csrf_token) = oauth
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    Redirect::temporary(auth_url.as_ref())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("failed to install signal handler");
        term.recv().await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[derive(Debug, Deserialize)]
struct AuthRequest {
    code: String,
}

#[derive(Deserialize, sqlx::FromRow, Clone)]
struct UserProfile {
    email: String,
}

async fn protected(profile: UserProfile) -> impl IntoResponse {
    (StatusCode::OK, profile.email)
}

async fn google_callback(
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    Query(query): Query<AuthRequest>,
    Extension(oauth_client): Extension<OauthClient>,
) -> Result<impl IntoResponse, ApiError> {
    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");

    let token = oauth_client
        .exchange_code(AuthorizationCode::new(query.code))
        .request_async(&http_client)
        .await?;

    let profile = state
        .ctx
        .get("https://openidconnect.googleapis.com/v1/userinfo")
        .bearer_auth(token.access_token().secret().to_owned())
        .send()
        .await?;

    let profile = profile.json::<UserProfile>().await?;

    let Some(secs) = token.expires_in() else {
        return Err(ApiError::OptionError);
    };

    let secs: i64 = secs.as_secs().try_into()?;

    let max_age = Local::now().naive_local() + Duration::try_from_secs_f64(secs as f64).unwrap();

    let cookie = Cookie::build(("sid", token.access_token().secret().to_owned()))
        .domain("localhost")
        .path("/")
        .secure(true)
        .http_only(true)
        .max_age(TimeDuration::seconds(secs));

    sqlx::query("INSERT INTO users (email) VALUES ($1) ON CONFLICT (email) DO NOTHING")
        .bind(profile.email.clone())
        .execute(&state.db)
        .await?;

    sqlx::query(
        "INSERT INTO sessions (user_id, session_id, expires_at) VALUES (
        (SELECT ID FROM USERS WHERE email = $1 LIMIT 1), $2, $3)
        ON CONFLICT (user_id) DO UPDATE SET
        session_id = excluded.session_id,
        expires_at = excluded.expires_at",
    )
    .bind(profile.email)
    .bind(token.access_token().secret().to_owned())
    .bind(max_age)
    .execute(&state.db)
    .await?;

    Ok((jar.add(cookie), Redirect::to("/protected")))
}

impl FromRequestParts<AppState> for UserProfile {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let cookiejar: PrivateCookieJar =
            PrivateCookieJar::from_request_parts(parts, &state).await?;

        let Some(cookie) = cookiejar.get("sid").map(|cookie| cookie.value().to_owned()) else {
            return Err(ApiError::Unauthorized);
        };

        let res = sqlx::query_as::<_, UserProfile>(
            "SELECT users.email
            FROM sessions
            LEFT JOIN USERS ON sessions.user_id = users.id
            WHERE sessions.session_id = $1
            LIMIT 1",
        )
        .bind(cookie)
        .fetch_one(&state.db)
        .await?;

        Ok(Self { email: res.email })
    }
}

// struct AuthMiddleware;
//
// impl<S> tower::Service<Request<S>> for AuthMiddleware {
//     type Response = Request<S>;
//     type Error = axum::http::Error;
//     type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;
//
//     fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//         Poll::Ready(Ok(()))
//     }
//
//     fn call(&mut self, req: Request<S>) -> Self::Future {
//         let future = async {
//             // Implement authentication logic here
//             Ok(req)
//         };
//
//         Box::pin(future)
//     }
// }
