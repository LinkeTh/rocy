pub mod adapters;
pub mod application;
mod config;
pub mod domain;
use application::types::{SessionId, SessionUser};
use axum::extract::{DefaultBodyLimit, FromRef, FromRequestParts};
use axum::{
    routing::{get, post},
    Router,
};
use axum_prometheus::PrometheusMetricLayer;
use cookie::Key;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use adapters::db::pg_image_repository::PgImageRepository;
use adapters::db::pg_session_repository::PgSessionRepository;
use adapters::db::pg_user_repository::PgUserRepository;
use adapters::http::handler;
use adapters::oauth::google_oauth::GoogleOAuthProvider;
use application::errors::AppError;
use application::services::image_service::ImageService;
use application::services::session_service::SessionService;
use application::services::user_service::UserService;
use application::use_cases::authentication::Authentication;
use axum::http::request::Parts;
use axum::http::{HeaderName, HeaderValue};
use axum_extra::extract::PrivateCookieJar;
use config::Config;
use reqwest::Method;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
struct AppState {
    key: Key,
    config: Config,
    session_service: Arc<SessionService<PgSessionRepository>>,
    user_service: Arc<UserService<PgUserRepository>>,
    image_service: Arc<ImageService<PgImageRepository>>,
    authentication: Arc<Authentication<PgSessionRepository, PgUserRepository, GoogleOAuthProvider>>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.key.clone()
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_env();
    init_tracing();
    let db = init_pg_pool(&config).await;
    run_sqlx_migration(&db).await;
    let state = init_app_state(&config, db.clone());
    let app = build_routes(state);
    let addr: SocketAddr = config.bind_addr.parse().expect("Invalid BIND_ADDR");

    info!(%addr, "starting server");
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind failed");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("server error");
}

fn init_app_state(config: &Config, db: Pool<Postgres>) -> AppState {
    let provider = Arc::new(GoogleOAuthProvider::new(config.clone()));
    let session_repo = PgSessionRepository { pool: db.clone() };
    let user_repo = PgUserRepository { pool: db.clone() };
    let image_repo = PgImageRepository { pool: db.clone() };
    let session_service = Arc::new(SessionService::new(session_repo));
    let user_service = Arc::new(UserService::new(user_repo));
    let image_service = Arc::new(ImageService::new(image_repo, config.clone()));
    let auth_service = Arc::new(Authentication::new(session_service.clone(), user_service.clone(), provider));
    let key = config.cookie_secret.clone();

    AppState {
        key: Key::from(key.as_bytes()),
        config: config.clone(),
        session_service,
        user_service,
        image_service,
        authentication: auth_service,
    }
}

async fn run_sqlx_migration(db: &Pool<Postgres>) {
    sqlx::migrate!().run(db).await.expect("Failed migrations :(");
}

async fn init_pg_pool(config: &Config) -> Pool<Postgres> {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres")
}

fn build_routes(state: AppState) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let login_router = Router::new().route("/api/login", get(handler::login_handler));
    let auth_router = Router::new().route("/api/auth/google/callback", get(handler::google_callback));
    let protected_router = Router::new()
        .route("/api/protected", get(handler::protected))
        .route("/api/me", get(handler::me))
        .route("/api/logout", get(handler::logout_handler));
    let serve_dir = ServeDir::new("web/dist/rocy-app/browser/").not_found_service(ServeFile::new("web/dist/rocy-app/browser/index.html"));

    let cors = if let Some(origins) = state.config.cors_allow_origins.as_ref() {
        if origins.as_str() == "*" {
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::PUT, Method::DELETE, Method::OPTIONS, Method::POST])
                .allow_headers(Any)
        } else {
            let origins = origins.split(',').filter_map(|s| s.trim().parse().ok()).collect::<Vec<_>>();
            CorsLayer::new()
                .allow_origin(origins)
                .allow_methods([Method::GET, Method::PUT, Method::DELETE, Method::OPTIONS, Method::POST])
                .allow_headers(Any)
        }
    } else {
        CorsLayer::permissive()
    };

    Router::new()
        .merge(login_router)
        .merge(auth_router)
        .merge(protected_router)
        .route("/api/healthz", get(handler::health_handler))
        .route("/api/upload", post(handler::upload_handler))
        .route("/api/images", get(handler::images_handler))
        .route("/api/metrics", get(|| async move { metric_handle.render() }))
        .fallback_service(serve_dir.clone())
        .with_state(state)
        .layer(prometheus_layer)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        // Security headers (set only if not already present)
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(DefaultBodyLimit::max(1024 * 1024 * 50))
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().with_target(false))
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("failed to install Ctrl+C handler");
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

    info!("shutting down");
}

impl FromRequestParts<AppState> for SessionUser {
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let state = state.to_owned();
        let cookie_jar: PrivateCookieJar = PrivateCookieJar::from_request_parts(parts, &state).await?;

        let Some(cookie_session_id) = cookie_jar.get("sid").map(|sid| sid.value().to_owned()) else {
            return Err(AppError::Unauthorized);
        };

        let res = state.session_service.find_user_by_session(&SessionId(cookie_session_id)).await?;

        let Some(user) = res else {
            return Err(AppError::Unauthorized);
        };
        Ok(user)
    }
}
