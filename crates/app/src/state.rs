//! 应用运行时状态组装。
//!
//! 作者：Xiaoman
//! 创建时间：2026-07-16

use adapter::agent_sidecar::RuntimeAgentSidecar;
#[cfg(not(feature = "license-lock"))]
use adapter::license::UnlockedLicenseGate;
#[cfg(feature = "license-lock")]
use adapter::license::{FailClosedLicenseGate, VerifierProcessLicense};
use kernel::event::InMemoryEventBus;
use ports::license::LicenseGate;
use runtime::sidecar::lifecycle::SidecarLifecycle;
use std::sync::Arc;

/// 桌面应用共享状态。
///
/// 功能：
///
/// - 持有 sidecar 生命周期与 Agent 网关
/// - 持有 License 闸门实现
/// - 持有进程内事件总线
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
pub struct AppState {
    /// Sidecar 生命周期控制器。
    pub lifecycle: Arc<SidecarLifecycle>,
    /// Agent sidecar 网关适配器。
    pub gateway: Arc<RuntimeAgentSidecar>,
    /// License 闸门（无锁 stub 或 verifier / fail-closed）。
    pub license: Arc<dyn LicenseGate>,
    /// 进程内事件总线（当前主要用于 sidecar 生命周期）。
    #[allow(dead_code)]
    pub event_bus: Arc<InMemoryEventBus>,
}

/// 按 Cargo feature 构造 License 闸门。
///
/// 功能：
///
/// - 无 `license-lock`：返回 [`UnlockedLicenseGate`]
/// - 有 `license-lock`：优先 [`VerifierProcessLicense`]，失败则 fail-closed
///
/// 作者：Xiaoman
/// 创建时间：2026-07-16
///
/// # 返回值
///
/// 返回可共享的 [`LicenseGate`] trait 对象。
pub fn build_license_gate() -> Arc<dyn LicenseGate> {
    #[cfg(feature = "license-lock")]
    {
        match VerifierProcessLicense::from_env() {
            Ok(gate) => Arc::new(gate),
            Err(error) => {
                tracing::error!(%error, "license-lock enabled but verifier unavailable");
                Arc::new(FailClosedLicenseGate::new(error.to_string()))
            }
        }
    }
    #[cfg(not(feature = "license-lock"))]
    {
        Arc::new(UnlockedLicenseGate::new())
    }
}
