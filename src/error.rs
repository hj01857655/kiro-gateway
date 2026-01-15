use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
#[allow(dead_code)]
pub enum AppError {
    #[error("未设置 access_token")]
    NoToken,

    #[error("Token 已过期")]
    TokenExpired,

    #[error("Token 刷新失败: {0}")]
    TokenRefreshFailed(String),

    #[error("请求参数错误: {0}")]
    BadRequest(String),

    #[error("Kiro API 错误: {0}")]
    KiroApiError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("解析错误: {0}")]
    ParseError(String),

    #[error("限流，请稍后重试")]
    RateLimited,

    #[error("配额已用尽")]
    QuotaExceeded,

    #[error("账号已被封禁: {0}")]
    AccountBanned(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message) = match &self {
            AppError::NoToken | AppError::TokenExpired => {
                (StatusCode::UNAUTHORIZED, "authentication_error", self.to_string())
            }
            AppError::TokenRefreshFailed(_) => {
                (StatusCode::UNAUTHORIZED, "authentication_error", self.to_string())
            }
            AppError::BadRequest(_) => {
                (StatusCode::BAD_REQUEST, "invalid_request_error", self.to_string())
            }
            AppError::RateLimited => {
                (StatusCode::TOO_MANY_REQUESTS, "rate_limit_error", self.to_string())
            }
            AppError::QuotaExceeded => {
                (StatusCode::TOO_MANY_REQUESTS, "rate_limit_error", self.to_string())
            }
            AppError::AccountBanned(_) => {
                (StatusCode::FORBIDDEN, "account_banned", self.to_string())
            }
            AppError::KiroApiError(_) | AppError::NetworkError(_) => {
                (StatusCode::BAD_GATEWAY, "api_error", self.to_string())
            }
            AppError::ParseError(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "server_error", self.to_string())
            }
        };

        let body = json!({
            "error": {
                "message": message,
                "type": error_type,
                "code": Option::<String>::None
            }
        });

        (status, Json(body)).into_response()
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::NetworkError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ParseError(err.to_string())
    }
}


