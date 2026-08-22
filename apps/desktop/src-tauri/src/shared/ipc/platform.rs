//! ???? Tauri commands ? ??????????????????

use common::DingDaResult;
use platform::protocol::registry::PlatformRegistry;
use serde::Serialize;

use crate::shared::ipc::IpcResponse;

/// ?????IPC ???? ? ???????????

#[derive(Debug, Clone, Serialize)]
pub struct PlatformDescriptorDto {
    pub kind: String,
    pub name: String,
    pub capabilities: Vec<String>,
}

/// ?????????????

#[tauri::command]
pub fn platform_descriptors() -> DingDaResult<IpcResponse<Vec<PlatformDescriptorDto>>> {
    let registry = PlatformRegistry::new();
    let descriptors = registry
        .descriptors()
        .into_iter()
        .map(|descriptor| PlatformDescriptorDto {
            kind: descriptor.kind,
            name: descriptor.name,
            capabilities: descriptor.capabilities.as_strings(),
        })
        .collect();
    Ok(IpcResponse::ok(descriptors))
}
