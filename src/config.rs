use base64::Engine;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub upload_dir: String,
    pub login_redirect_url: String,
    pub oauth_client_id: String,
    pub oauth_client_secret: String,
    pub oauth_redirect_url: String,
    pub cookie_secret: String,
    pub cookie_domain: Option<String>,
    pub cookie_secure: bool,
    pub cookie_samesite: SameSiteMode,
    pub cors_allow_origins: Option<String>,
    pub enable_offline_access: bool,
    pub openai_key: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub enum SameSiteMode {
    Lax,
    Strict,
    None,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            bind_addr: "0.0.0.0:8080".into(),
            database_url: "".into(),
            upload_dir: "uploads".into(),
            login_redirect_url: String::new(),
            oauth_client_id: String::new(),
            oauth_client_secret: String::new(),
            oauth_redirect_url: String::new(),
            cookie_secret: String::new(),
            cookie_domain: None,
            cookie_secure: false,
            cookie_samesite: SameSiteMode::Lax,
            cors_allow_origins: None,
            enable_offline_access: false,
            openai_key: String::new(),
        }
    }
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let mut c = Config::default();
        c.bind_addr = std::env::var("BIND_ADDR").unwrap_or(c.bind_addr);
        c.database_url = std::env::var("DATABASE_URL").unwrap_or(c.database_url);
        c.upload_dir = std::env::var("UPLOAD_DIR").unwrap_or(c.upload_dir);
        c.login_redirect_url = std::env::var("LOGIN_REDIRECT_URL").unwrap_or(c.login_redirect_url);
        c.oauth_client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or(c.oauth_client_id);
        c.oauth_client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or(c.oauth_client_secret);
        c.oauth_redirect_url = std::env::var("OAUTH_REDIRECT_URL").unwrap_or(c.oauth_redirect_url);
        c.cookie_secret = std::env::var("COOKIE_SECRET").unwrap_or(c.cookie_secret);
        c.cookie_domain = std::env::var("COOKIE_DOMAIN").ok();
        c.cookie_secure = std::env::var("COOKIE_SECURE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        c.cookie_samesite = match std::env::var("COOKIE_SAMESITE").unwrap_or("Lax".into()).as_str() {
            "Strict" => SameSiteMode::Strict,
            "None" => SameSiteMode::None,
            _ => SameSiteMode::Lax,
        };
        c.cors_allow_origins = std::env::var("CORS_ALLOW_ORIGINS").ok();
        c.enable_offline_access = std::env::var("ENABLE_OFFLINE_ACCESS")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        c.openai_key = std::env::var("OPENAI_API_KEY").unwrap_or(c.openai_key);
        if matches!(c.cookie_samesite, SameSiteMode::None) && !c.cookie_secure {
            panic!("COOKIE_SAMESITE=None requires COOKIE_SECURE=true");
        }
        let secret_bytes = match base64::engine::general_purpose::STANDARD.decode(&c.cookie_secret) {
            Ok(decoded) if !decoded.is_empty() => decoded,
            _ => c.cookie_secret.as_bytes().to_vec(),
        };
        if secret_bytes.len() < 64 {
            panic!("COOKIE_SECRET too weak (needs at least 64 bytes)");
        }
        c
    }
}
