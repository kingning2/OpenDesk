//! 知识库文件上传：把 web 前端上传的字节写入 server 临时目录。

use axum::extract::{DefaultBodyLimit, Multipart};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use std::path::PathBuf;
use uuid::Uuid;

/// 上传根目录：`OPENDESK_UPLOAD_DIR` 覆盖，否则取系统临时目录下 `opendesk-uploads`。
fn upload_root() -> PathBuf {
    std::env::var("OPENDESK_UPLOAD_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("opendesk-uploads"))
}

/// 处理 `multipart/form-data` 上传（字段名 `file`），返回落盘绝对路径。
pub async fn upload_file(mut multipart: Multipart) -> impl IntoResponse {
    let field = match multipart.next_field().await {
        Ok(Some(field)) => field,
        Ok(None) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": "no file field" })),
            )
                .into_response();
        }
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": error.to_string() })),
            )
                .into_response();
        }
    };

    let file_name = field.file_name().unwrap_or("upload").to_string();
    let bytes = match field.bytes().await {
        Ok(bytes) => bytes,
        Err(error) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": error.to_string() })),
            )
                .into_response();
        }
    };

    let dir = upload_root();
    if let Err(error) = std::fs::create_dir_all(&dir) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": error.to_string() })),
        )
            .into_response();
    }
    let safe_name = sanitize(&file_name);
    let path = dir.join(format!("{}-{safe_name}", Uuid::new_v4()));
    if let Err(error) = std::fs::write(&path, &bytes) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "ok": false, "error": error.to_string() })),
        )
            .into_response();
    }

    tracing::info!(path = %path.display(), bytes = bytes.len(), "uploaded file");

    (
        StatusCode::OK,
        Json(json!({ "ok": true, "file_path": path.to_string_lossy() })),
    )
        .into_response()
}

/// 去掉文件名中危险字符。
fn sanitize(name: &str) -> String {
    let name = name.trim();
    if name.is_empty() {
        return "upload".to_string();
    }
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect()
}

/// 配置上传体上限（默认 200MB）。
pub const UPLOAD_LIMIT: usize = 200 * 1024 * 1024;

/// 供路由使用。
pub fn default_body_limit() -> DefaultBodyLimit {
    DefaultBodyLimit::max(UPLOAD_LIMIT)
}
