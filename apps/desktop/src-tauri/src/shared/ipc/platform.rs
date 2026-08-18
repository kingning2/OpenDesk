//! ???? Tauri commands ? ??????????????????

use common::OpenDeskResult;
use platform::registry::PlatformRegistry;
use serde::Serialize;

/// ?????IPC ???? ? ???????????

#[derive(Debug, Clone, Serialize)]
pub struct PlatformDescriptorDto {
    pub kind: String,
    pub name: String,
    pub capabilities: Vec<String>,
}

/// ?????????????

#[tauri::command]
pub fn platform_descriptors() -> OpenDeskResult<Vec<PlatformDescriptorDto>> {
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
    Ok(descriptors)
}
