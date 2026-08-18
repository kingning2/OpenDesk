//! Example: Rust sidecar HTTP client (Rust → Python).
//!
//! Real code lives in `crates/runtime/src/sidecar/`.
//! This file documents the pattern only.

/*
use runtime::sidecar::client::SidecarClient;
use runtime::sidecar::routes::agent_ping::{agent_ping, AgentPingRequest};

async fn example(runtime_port: u16, trace_id: String) -> Result<(), Box<dyn std::error::Error>> {
    let client = SidecarClient::new(runtime_port);
    let response = agent_ping(
        &client,
        AgentPingRequest { trace_id },
    )
    .await?;
    assert!(response.ok);
    Ok(())
}
*/

fn main() {
    // documentation-only example; see crates/runtime/src/sidecar/
}
