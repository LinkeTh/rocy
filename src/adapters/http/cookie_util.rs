use crate::config;
use crate::config::Config;
use cookie::time::Duration as CookieDuration;
use cookie::{Cookie, SameSite};

pub fn make_cookie(config: Config, name: &str, value: &str, max_age_secs: CookieDuration, http_only: bool) -> Cookie<'static> {
    let mut builder = Cookie::build((name.to_string(), value.to_string()))
        .path("/")
        .http_only(http_only)
        .max_age(max_age_secs);
    if let Some(domain) = &config.cookie_domain {
        builder = builder.domain(domain.clone());
    }
    if config.cookie_secure {
        builder = builder.secure(true);
    }
    builder = match config.cookie_samesite {
        config::SameSiteMode::Lax => builder.same_site(SameSite::Lax),
        config::SameSiteMode::Strict => builder.same_site(SameSite::Strict),
        config::SameSiteMode::None => builder.same_site(SameSite::None),
    };
    builder.build()
}
