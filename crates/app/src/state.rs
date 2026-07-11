use adapter::agent_sidecar::RuntimeAgentSidecar;
use kernel::event::InMemoryEventBus;
use runtime::sidecar::lifecycle::SidecarLifecycle;
use std::sync::Arc;

pub struct AppState {
    pub lifecycle: Arc<SidecarLifecycle>,
    pub gateway: Arc<RuntimeAgentSidecar>,
    #[allow(dead_code)]
    pub event_bus: Arc<InMemoryEventBus>,
}
