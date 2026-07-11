//! Sidecar process lifecycle — start / health / stop / restart.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use common::contracts::RuntimeEventSidecarRestarted;
use kernel::event::EventBus;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;
use tracing::{info, warn};

use super::client::SidecarClient;
use super::log_pipe;

pub const SIDECAR_RESTARTED_TOPIC: &str = "runtime.sidecar.restarted";

#[derive(Debug, thiserror::Error)]
pub enum SidecarLifecycleError {
    #[error("sidecar directory not found: {0}")]
    SidecarDirNotFound(String),
    #[error("failed to spawn sidecar: {0}")]
    SpawnFailed(String),
    #[error("sidecar startup timed out after {0:?}")]
    StartupTimeout(Duration),
    #[error("sidecar stop failed: {0}")]
    StopFailed(String),
}

#[derive(Debug, Clone)]
pub struct SidecarConfig {
    pub port: u16,
    pub sidecar_dir: PathBuf,
    pub use_uv: bool,
    pub python_executable: String,
    pub startup_timeout: Duration,
    pub max_restart_attempts: u32,
    /// Frozen sidecar executable (release / OPENDESK_SIDECAR_BIN). When set, takes precedence over dev spawn.
    pub bundled_executable: Option<PathBuf>,
}

impl SidecarConfig {
    pub fn from_env() -> Self {
        let port = std::env::var("OPENDESK_SIDECAR_PORT")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(8787);
        let max_restart_attempts = std::env::var("OPENDESK_SIDECAR_MAX_RESTARTS")
            .ok()
            .and_then(|value| value.parse().ok())
            .unwrap_or(5);

        Self {
            port,
            sidecar_dir: resolve_sidecar_dir(),
            use_uv: std::env::var("OPENDESK_USE_UV")
                .map(|value| value != "0")
                .unwrap_or(true),
            python_executable: std::env::var("OPENDESK_PYTHON")
                .unwrap_or_else(|_| "python".to_string()),
            startup_timeout: Duration::from_secs(15),
            max_restart_attempts,
            bundled_executable: resolve_bundled_executable(),
        }
    }

    pub fn with_bundled_executable(mut self, path: PathBuf) -> Self {
        self.bundled_executable = Some(path);
        self
    }
}

pub struct SidecarLifecycle {
    config: SidecarConfig,
    client: SidecarClient,
    child: Mutex<Option<Child>>,
    event_bus: Arc<dyn EventBus>,
    ever_started: Mutex<bool>,
    restart_attempts: Mutex<u32>,
}

impl SidecarLifecycle {
    pub fn new(config: SidecarConfig, event_bus: Arc<dyn EventBus>) -> Self {
        let client = SidecarClient::new(config.port);
        Self {
            config,
            client,
            child: Mutex::new(None),
            event_bus,
            ever_started: Mutex::new(false),
            restart_attempts: Mutex::new(0),
        }
    }

    pub fn client(&self) -> &SidecarClient {
        &self.client
    }

    pub async fn ensure_running(&self) -> Result<(), SidecarLifecycleError> {
        if self.child_exited().await {
            return self.restart_with_event("process exited").await;
        }
        if self.health_check().await? {
            return Ok(());
        }
        self.restart_with_event("health check failed").await
    }

    pub async fn health_check(&self) -> Result<bool, SidecarLifecycleError> {
        self.client
            .health_check()
            .await
            .map_err(|error| SidecarLifecycleError::SpawnFailed(error.to_string()))
    }

    pub async fn restart(&self) -> Result<(), SidecarLifecycleError> {
        self.stop().await?;
        self.start_internal(true).await
    }

    async fn restart_with_event(&self, reason: &str) -> Result<(), SidecarLifecycleError> {
        let should_publish = *self.ever_started.lock().await;
        let attempt = {
            let mut attempts = self.restart_attempts.lock().await;
            *attempts += 1;
            *attempts
        };

        if attempt > self.config.max_restart_attempts {
            return Err(SidecarLifecycleError::SpawnFailed(format!(
                "max restart attempts ({}) exceeded",
                self.config.max_restart_attempts
            )));
        }

        self.stop().await?;
        self.start_internal(true).await?;

        if should_publish {
            self.publish_restarted(attempt, reason);
        }
        Ok(())
    }

    pub async fn start(&self) -> Result<(), SidecarLifecycleError> {
        if self.health_check().await? {
            info!(port = self.config.port, "sidecar already healthy");
            *self.ever_started.lock().await = true;
            return Ok(());
        }
        self.start_internal(true).await
    }

    async fn start_internal(&self, track_child: bool) -> Result<(), SidecarLifecycleError> {
        {
            let guard = self.child.lock().await;
            if guard.is_some() {
                return self.wait_until_healthy().await;
            }
        }

        if self.config.bundled_executable.is_none() && !self.config.sidecar_dir.exists() {
            return Err(SidecarLifecycleError::SidecarDirNotFound(
                self.config.sidecar_dir.display().to_string(),
            ));
        }

        let mut command = build_spawn_command(&self.config)?;
        let mut child = command
            .spawn()
            .map_err(|error| SidecarLifecycleError::SpawnFailed(error.to_string()))?;

        if let Some(stdout) = child.stdout.take() {
            tokio::spawn(pipe_logs(stdout, "stdout"));
        }
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(pipe_logs(stderr, "stderr"));
        }

        info!(port = self.config.port, "sidecar process spawned");
        if track_child {
            *self.child.lock().await = Some(child);
        }
        self.wait_until_healthy().await?;
        *self.ever_started.lock().await = true;
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), SidecarLifecycleError> {
        let mut guard = self.child.lock().await;
        if let Some(mut child) = guard.take() {
            child
                .kill()
                .await
                .map_err(|error| SidecarLifecycleError::StopFailed(error.to_string()))?;
            if let Err(error) = child.wait().await {
                warn!(%error, "sidecar wait after kill failed");
            }
            info!("sidecar process stopped");
        }
        Ok(())
    }

    async fn child_exited(&self) -> bool {
        let mut guard = self.child.lock().await;
        let Some(child) = guard.as_mut() else {
            return false;
        };

        match child.try_wait() {
            Ok(Some(_status)) => {
                guard.take();
                true
            }
            Ok(None) => false,
            Err(error) => {
                warn!(%error, "sidecar try_wait failed");
                false
            }
        }
    }

    fn publish_restarted(&self, attempt: u32, reason: &str) {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        let payload = RuntimeEventSidecarRestarted {
            event_id: format!("evt-{millis}"),
            occurred_at: millis.to_string(),
            port: self.config.port as i64,
            attempt: attempt as i64,
            reason: Some(reason.to_string()),
        };

        let Ok(bytes) = serde_json::to_vec(&payload) else {
            warn!("failed to serialize sidecar restarted event");
            return;
        };

        if let Err(error) = self.event_bus.publish(SIDECAR_RESTARTED_TOPIC, &bytes) {
            warn!(%error, "failed to publish sidecar restarted event");
        }
    }

    async fn wait_until_healthy(&self) -> Result<(), SidecarLifecycleError> {
        let deadline = Instant::now() + self.config.startup_timeout;
        while Instant::now() < deadline {
            if self.health_check().await? {
                info!(port = self.config.port, "sidecar is healthy");
                return Ok(());
            }
            tokio::time::sleep(Duration::from_millis(200)).await;
        }

        let _ = self.stop().await;
        Err(SidecarLifecycleError::StartupTimeout(
            self.config.startup_timeout,
        ))
    }
}

fn resolve_sidecar_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("OPENDESK_SIDECAR_DIR") {
        return PathBuf::from(dir);
    }

    if let Ok(mut dir) = std::env::current_dir() {
        for _ in 0..6 {
            let candidate = dir.join("python").join("sidecar");
            if candidate.exists() {
                return candidate;
            }
            if !dir.pop() {
                break;
            }
        }
    }

    PathBuf::from("python/sidecar")
}

fn build_spawn_command(config: &SidecarConfig) -> Result<Command, SidecarLifecycleError> {
    let port = config.port.to_string();

    if let Some(bundled) = config.bundled_executable.as_ref() {
        if bundled.is_file() {
            let mut cmd = Command::new(bundled);
            cmd.arg("--port").arg(&port);
            info!(executable = %bundled.display(), "starting bundled sidecar");
            return Ok(configure_stdio(cmd));
        }
        warn!(
            executable = %bundled.display(),
            "bundled sidecar executable missing; falling back to development launcher"
        );
    }

    let sidecar_dir = path_to_str(&config.sidecar_dir);

    if config.use_uv && command_available("uv") {
        let mut cmd = Command::new("uv");
        cmd.arg("run")
            .arg("--directory")
            .arg(&sidecar_dir)
            .arg("python")
            .arg("-m")
            .arg("sidecar.main")
            .arg("--port")
            .arg(&port);
        return Ok(configure_stdio(cmd));
    }

    if config.use_uv {
        warn!("uv not found in PATH, falling back to python executable");
    }

    for candidate in spawn_python_candidates(config) {
        if !command_available(&candidate) {
            continue;
        }

        let mut cmd = Command::new(&candidate);
        cmd.current_dir(&config.sidecar_dir)
            .arg("-m")
            .arg("sidecar.main")
            .arg("--port")
            .arg(&port);
        info!(executable = %candidate, "starting sidecar with python");
        return Ok(configure_stdio(cmd));
    }

    Err(SidecarLifecycleError::SpawnFailed(
        "no Python runtime found in PATH (install uv, or ensure python/py is available; set OPENDESK_PYTHON)".into(),
    ))
}

fn spawn_python_candidates(config: &SidecarConfig) -> Vec<String> {
    let mut candidates = vec![config.python_executable.clone()];
    for fallback in ["python", "python3", "py"] {
        if !candidates.iter().any(|value| value == fallback) {
            candidates.push(fallback.to_string());
        }
    }
    candidates
}

fn configure_stdio(mut command: Command) -> Command {
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    command
}

fn resolve_bundled_executable() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("OPENDESK_SIDECAR_BIN") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Some(path);
        }
    }

    if cfg!(debug_assertions) {
        return None;
    }

    let candidate = bundled_executable_candidate();
    if candidate.is_file() {
        Some(candidate)
    } else {
        None
    }
}

fn bundled_executable_candidate() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(Path::to_path_buf))
        .unwrap_or_default();
    exe_dir.join(bundled_sidecar_filename())
}

pub fn bundled_sidecar_filename() -> String {
    let base = format!("sidecar-{}", env!("OPENDESK_TARGET_TRIPLE"));
    if cfg!(target_os = "windows") {
        format!("{base}.exe")
    } else {
        base
    }
}

fn command_available(program: &str) -> bool {
    #[cfg(windows)]
    {
        std::process::Command::new("where")
            .arg(program)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    #[cfg(not(windows))]
    {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("command -v {program}"))
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }
}

fn path_to_str(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

async fn pipe_logs<R>(reader: R, stream: &'static str)
where
    R: tokio::io::AsyncRead + Unpin + Send + 'static,
{
    let mut lines = BufReader::new(reader).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log_pipe::emit_line(stream, &line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_sidecar_filename_matches_target_triple() {
        let filename = bundled_sidecar_filename();
        assert!(filename.starts_with("sidecar-"));
        if cfg!(target_os = "windows") {
            assert!(filename.ends_with(".exe"));
        } else {
            assert!(!filename.ends_with(".exe"));
        }
    }
}
