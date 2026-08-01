//! 共享 tracing 初始化与本地文件日志。
//!
//! 作者：coisini
//! 创建时间：2026-08-01

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use chrono::{DateTime, FixedOffset, Utc};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::Value;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

static LOG_GUARD: OnceLock<tracing_appender::non_blocking::WorkerGuard> = OnceLock::new();
static FILE_MUTEX: Mutex<()> = Mutex::new(());

const MAX_FIELD: usize = 4_096;
const SENSITIVE_KEYS: [&str; 6] = [
    "api_key",
    "apikey",
    "password",
    "secret",
    "token",
    "authorization",
];

type BeijingTime = DateTime<FixedOffset>;

/// 北京时间（UTC+8）。
fn beijing_now() -> BeijingTime {
    Utc::now().with_timezone(&FixedOffset::east_opt(8 * 3600).expect("UTC+8"))
}

/// OpenDesk 数据目录。
pub fn data_dir() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(std::env::temp_dir);
    path.push("OpenDesk");
    path
}

/// 日志目录 `{data_dir}/logs`。
pub fn log_dir() -> PathBuf {
    data_dir().join("logs")
}

/// 初始化 tracing：压缩历史日志，文件记 `warn+` / `lifecycle` / `ipc`，开发环境额外输出控制台。
pub fn init_tracing(service_name: &str) {
    let dir = log_dir();
    let _ = std::fs::create_dir_all(&dir);
    compress_old_logs();

    let (writer, guard) = tracing_appender::non_blocking(tracing_appender::rolling::daily(
        &dir,
        format!("{service_name}.log"),
    ));
    let _ = LOG_GUARD.set(guard);

    let file = fmt::layer()
        .with_writer(writer)
        .with_ansi(false)
        .with_target(false)
        .with_filter(EnvFilter::new("warn,[lifecycle]=info,[ipc]=info"));
    let registry = tracing_subscriber::registry().with(file);

    if cfg!(debug_assertions) {
        let console_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            EnvFilter::new(
                "info,crawler=info,opendesk=debug,runtime=debug,app=debug,mail=info,mail_net=info",
            )
        });
        let console = fmt::layer().with_target(true).with_filter(console_filter);
        let _ = registry.with(console).try_init();
    } else {
        let _ = registry.try_init();
    }
}

/// 启动时把今天之前的滚动日志 gzip 压缩（如 `opendesk.log.2026-07-31` → `.gz`）。
fn compress_old_logs() {
    let today = beijing_now().format("%Y-%m-%d").to_string();
    let entries = match fs::read_dir(log_dir()) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if !path.is_file() || name.ends_with(".gz") {
            continue;
        }
        let Some(date) = log_file_date(name) else {
            continue;
        };
        if date == today {
            continue;
        }
        if let Err(error) = gzip_file(&path) {
            tracing::warn!(
                target: "lifecycle",
                %error,
                file = %path.display(),
                "failed to compress old log file"
            );
        }
    }
}

/// 从滚动日志文件名解析日期（`opendesk.log.2026-07-31`）。
fn log_file_date(name: &str) -> Option<&str> {
    if name.ends_with(".gz") {
        return None;
    }
    let (_, date) = name.split_once(".log.")?;
    Some(date).filter(|value| !value.is_empty())
}

/// 压缩单个日志文件并删除原文件。
fn gzip_file(path: &Path) -> std::io::Result<()> {
    let mut input = File::open(path)?;
    let gz_path = format!("{}.gz", path.display());
    let output = File::create(gz_path)?;
    let mut encoder = GzEncoder::new(output, Compression::default());
    std::io::copy(&mut input, &mut encoder)?;
    encoder.finish()?;
    fs::remove_file(path)
}

/// 写入一行：`北京时间【LEVEL】【event】【input】【output】`
pub fn write_log(level: &str, event: &str, input: &str, output: &str) {
    let now = beijing_now();
    let line = format!(
        "{}【{level}】【{event}】【入参：{}】【出参：{}】\n",
        now.format("%Y-%m-%d %H:%M:%S"),
        sanitize(input),
        sanitize(output),
    );

    if let Ok(_guard) = FILE_MUTEX.lock() {
        let path = log_dir().join(format!("opendesk.log.{}", now.format("%Y-%m-%d")));
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(line.as_bytes()));
    }
}

/// 脱敏 + 截断日志字段。
fn sanitize(raw: &str) -> String {
    let text = match serde_json::from_str::<Value>(raw) {
        Ok(mut value) => {
            redact(&mut value);
            serde_json::to_string(&value).unwrap_or_else(|_| raw.to_string())
        }
        Err(_) => raw.to_string(),
    };
    if text.len() <= MAX_FIELD {
        text
    } else {
        format!("{}…", &text[..MAX_FIELD])
    }
}

/// 递归脱敏敏感字段。
fn redact(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                let normalized = key.to_ascii_lowercase();
                if SENSITIVE_KEYS
                    .iter()
                    .any(|candidate| normalized.contains(candidate))
                {
                    *child = Value::String("***".into());
                } else {
                    redact(child);
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(redact),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::log_file_date;

    #[test]
    fn parses_log_file_date() {
        assert_eq!(log_file_date("opendesk.log.2026-07-31"), Some("2026-07-31"));
        assert_eq!(
            log_file_date("opendesk-worker.log.2026-07-31"),
            Some("2026-07-31")
        );
        assert_eq!(log_file_date("opendesk.log.2026-07-31.gz"), None);
        assert_eq!(log_file_date("readme.txt"), None);
    }
}
