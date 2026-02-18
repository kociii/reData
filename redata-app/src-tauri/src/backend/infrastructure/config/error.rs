// 错误处理模块

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    // 数据库错误
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("SQL error: {0}")]
    Sql(#[from] sqlx::Error),

    // 业务逻辑错误
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    // AI 相关错误
    #[error("AI service error: {0}")]
    AiService(String),

    // 文件处理错误
    #[error("File error: {0}")]
    FileError(String),

    #[error("Excel parsing error: {0}")]
    ExcelParsing(String),

    // 通用错误
    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    // 外部库错误
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// 错误响应结构
#[derive(serde::Serialize)]
struct ErrorResponse {
    error: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type, message, details) = match self {
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg,
                None,
            ),
            AppError::AlreadyExists(msg) => (
                StatusCode::CONFLICT,
                "ALREADY_EXISTS",
                msg,
                None,
            ),
            AppError::InvalidInput(msg) | AppError::ValidationFailed(msg) => (
                StatusCode::BAD_REQUEST,
                "INVALID_INPUT",
                msg,
                None,
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "UNAUTHORIZED",
                "Unauthorized access".to_string(),
                None,
            ),
            AppError::Forbidden => (
                StatusCode::FORBIDDEN,
                "FORBIDDEN",
                "Forbidden".to_string(),
                None,
            ),
            AppError::Database(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database operation failed".to_string(),
                Some(e.to_string()),
            ),
            AppError::Sql(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SQL_ERROR",
                "SQL operation failed".to_string(),
                Some(e.to_string()),
            ),
            AppError::AiService(msg) => (
                StatusCode::BAD_GATEWAY,
                "AI_SERVICE_ERROR",
                msg,
                None,
            ),
            AppError::FileError(msg) | AppError::ExcelParsing(msg) => (
                StatusCode::BAD_REQUEST,
                "FILE_ERROR",
                msg,
                None,
            ),
            AppError::Io(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
                "IO operation failed".to_string(),
                Some(e.to_string()),
            ),
            AppError::Serialization(ref e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERIALIZATION_ERROR",
                "Serialization failed".to_string(),
                Some(e.to_string()),
            ),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                msg,
                None,
            ),
            AppError::Other(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                e.to_string(),
                None,
            ),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message,
            details,
        });

        (status, body).into_response()
    }
}

/// Result 类型别名
pub type Result<T> = std::result::Result<T, AppError>;
