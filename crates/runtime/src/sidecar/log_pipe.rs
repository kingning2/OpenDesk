//! Parse structured Python sidecar log lines and emit tracing events.

use serde::Deserialize;
use tracing::{debug, error, info, warn};

#[derive(Debug, Deserialize)]
struct PythonLogLine {
    level: String,
    message: String,
    #[serde(default)]
    logger: Option<String>,
    #[serde(default)]
    trace_id: Option<String>,
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    feature: Option<String>,
    #[serde(default)]
    tenant_id: Option<String>,
}

pub fn emit_line(stream: &str, line: &str) {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return;
    }

    if let Ok(parsed) = serde_json::from_str::<PythonLogLine>(trimmed) {
        emit_structured(stream, &parsed);
        return;
    }

    info!(
        target: "opendesk.sidecar",
        stream,
        message = trimmed,
        "sidecar log (unstructured)"
    );
}

fn emit_structured(stream: &str, parsed: &PythonLogLine) {
    let level = parsed.level.to_ascii_uppercase();
    let logger = parsed.logger.as_deref().unwrap_or("unknown");
    let trace_id = parsed.trace_id.as_deref().unwrap_or("-");
    let task_id = parsed.task_id.as_deref().unwrap_or("-");
    let feature = parsed.feature.as_deref().unwrap_or("-");
    let tenant_id = parsed.tenant_id.as_deref().unwrap_or("-");

    match level.as_str() {
        "ERROR" | "CRITICAL" => error!(
            target: "opendesk.sidecar",
            stream,
            logger,
            trace_id,
            task_id,
            feature,
            tenant_id,
            message = %parsed.message,
        ),
        "WARNING" | "WARN" => warn!(
            target: "opendesk.sidecar",
            stream,
            logger,
            trace_id,
            task_id,
            feature,
            tenant_id,
            message = %parsed.message,
        ),
        "DEBUG" => debug!(
            target: "opendesk.sidecar",
            stream,
            logger,
            trace_id,
            task_id,
            feature,
            tenant_id,
            message = %parsed.message,
        ),
        _ => info!(
            target: "opendesk.sidecar",
            stream,
            logger,
            trace_id,
            task_id,
            feature,
            tenant_id,
            message = %parsed.message,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_json_log_line_fields() {
        let line = r#"{"level":"INFO","message":"handle_agent_ping","logger":"opendesk.sidecar.agent","trace_id":"t-1","feature":"agent"}"#;
        let parsed = serde_json::from_str::<PythonLogLine>(line);
        assert!(parsed.is_ok());
        let parsed = parsed.ok();
        assert_eq!(
            parsed.as_ref().and_then(|p| p.trace_id.as_deref()),
            Some("t-1")
        );
        assert_eq!(
            parsed.as_ref().and_then(|p| p.feature.as_deref()),
            Some("agent")
        );
    }
}
