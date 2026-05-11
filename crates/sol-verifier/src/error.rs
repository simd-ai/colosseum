use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum VerifierError {
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for VerifierError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            VerifierError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            VerifierError::Internal(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.clone()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}
