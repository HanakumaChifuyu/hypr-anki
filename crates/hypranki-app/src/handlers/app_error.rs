use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use tracing::error;

use self::AppError::*;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, msg) = match self {
            NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            InternalError(msg) => {
                error!("{:#?}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "内部服务错误".to_string(),
                )
            }
        };
        let body = Json(json! (
            { "success":false,
             "message" : msg

            }
        ));
        (status, body).into_response()
    }
}
