use axum::response::Response;
use axum::{Json, http::StatusCode, response::IntoResponse};
use oauth2::HttpClientError;
use reqwest::Error;
use serde::{Serialize, Serializer};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("SQL error: {0}")]
    SQL(#[from] sqlx::Error),
    #[error("HTTP request error: {0}")]
    Request(#[from] reqwest::Error),
    #[error("OAuth token error: {0}")]
    TokenError(#[from] oauth2::RequestTokenError<HttpClientError<Error>, oauth2::StandardErrorResponse<oauth2::basic::BasicErrorResponseType>>),
    #[error("Payload too large")]
    PayloadTooLarge,
    #[error("Invalid content type")]
    InvalidContentType,
    #[error("Media type not supported")]
    UnsupportedMediaType,
    #[error("You're not authorized!")]
    Unauthorized,
    #[error("Attempted to get a non-none value but found none")]
    ParseIntError(#[from] std::num::TryFromIntError),
    #[error("Encountered an error trying to convert an infallible value: {0}")]
    FromRequestPartsError(#[from] std::convert::Infallible),
}

struct AppStatusCode(StatusCode);

impl Serialize for AppStatusCode {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(u16::from(self.0))
    }
}

#[derive(Serialize)]
struct JsonErrorResponse {
    code: &'static str,
    message: &'static str,
    status: AppStatusCode,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let res = match &self {
            Self::SQL(_) => JsonErrorResponse {
                code: "sql_error",
                message: "Internal server error",
                status: AppStatusCode(StatusCode::INTERNAL_SERVER_ERROR),
            },
            Self::UnsupportedMediaType => JsonErrorResponse {
                code: "unsupported_media_type",
                message: "Only valid image types are allowed (jpeg, png, gif, webp)",
                status: AppStatusCode(StatusCode::UNSUPPORTED_MEDIA_TYPE),
            },
            Self::PayloadTooLarge => JsonErrorResponse {
                code: "payload_too_large",
                message: "Upload too large",
                status: AppStatusCode(StatusCode::PAYLOAD_TOO_LARGE),
            },
            Self::InvalidContentType => JsonErrorResponse {
                code: "invalid_content",
                message: "Expected multipart/form-data",
                status: AppStatusCode(StatusCode::BAD_REQUEST),
            },
            Self::Request(_) => JsonErrorResponse {
                code: "http_error",
                message: "Upstream request failed",
                status: AppStatusCode(StatusCode::INTERNAL_SERVER_ERROR),
            },
            Self::TokenError(_) => JsonErrorResponse {
                code: "oauth_error",
                message: "OAuth token exchange failed",
                status: AppStatusCode(StatusCode::BAD_GATEWAY),
            },
            Self::Unauthorized => JsonErrorResponse {
                code: "unauthorized",
                message: "Unauthorized",
                status: AppStatusCode(StatusCode::UNAUTHORIZED),
            },
            Self::ParseIntError(_) => JsonErrorResponse {
                code: "parse_int_error",
                message: "Parsing error",
                status: AppStatusCode(StatusCode::INTERNAL_SERVER_ERROR),
            },
            Self::FromRequestPartsError(_) => JsonErrorResponse {
                code: "extract_error",
                message: "Request extraction error",
                status: AppStatusCode(StatusCode::INTERNAL_SERVER_ERROR),
            },
        };

        error!(error = ?self, code = res.code, message = res.message, status = %res.status.0);

        (res.status.0, Json(res)).into_response()
    }
}
