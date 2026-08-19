//! dingda-server — Web 服务形态（验证 crates/app 不绑定 Tauri，可复用为 HTTP 服务）。
//!
//! 仅暴露只读健康检查与账号列表，证明业务库可被 Web 层直接复用；
//! 完整业务 API（订单/发货/评价等）后续按需补齐。

#[macro_use]
extern crate tracing;

mod state;

use axum::{extract::State, routing::get, Json, Router};
use state::AppState;

/// 健康检查响应。
#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
}

/// 账号列表响应（复用 crates/app 的账号领域模型）。
#[derive(serde::Serialize)]
struct AccountsResponse {
    total: usize,
    accounts: Vec<app::account::XianyuAccount>,
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "dingda-server",
    })
}

async fn list_accounts(State(state): State<AppState>) -> Json<AccountsResponse> {
    let service = app::account::AccountService::new(state.store());
    let accounts = service.list(1).unwrap_or_default();
    let total = accounts.len();
    Json(AccountsResponse { total, accounts })
}

fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/accounts", get(list_accounts))
        .with_state(state)
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().with_env_filter("info").init();

    let state = AppState::new();
    let app = router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8089").await?;
    info!("dingda-server listening on http://127.0.0.1:8089");
    axum::serve(listener, app).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn health_returns_ok() {
        let state = AppState::new();
        let app = router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{addr}/health"))
            .send()
            .await
            .expect("request");
        assert_eq!(response.status(), 200);
        server.abort();
    }

    #[tokio::test]
    async fn accounts_endpoint_works() {
        let state = AppState::new();
        let app = router(state);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let addr = listener.local_addr().expect("addr");
        let server = tokio::spawn(async move {
            axum::serve(listener, app).await.expect("serve");
        });

        let client = reqwest::Client::new();
        let response = client
            .get(format!("http://{addr}/api/accounts"))
            .send()
            .await
            .expect("request");
        assert_eq!(response.status(), 200);
        server.abort();
    }
}
