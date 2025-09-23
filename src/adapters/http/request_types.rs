use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct AuthRequest {
    pub(crate) code: String,
    pub(crate) state: String,
}
