//! User events send via the winit event_loop proxy.

#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use crate::wgpu_context::WgpuContext;

/// How to rom source is stored.
/// An alternative to this would be to only use #[cfg].
pub enum RomSource {
    /// As a file for native use
    #[cfg(not(target_arch = "wasm32"))]
    Path(PathBuf),
    /// As bytes for web use
    #[cfg(target_arch = "wasm32")]
    Bytes(Vec<u8>),
}

/// User events sent by the winit event_loop proxy
/// This is not actually needed in native targets,
/// since I can load the rom directly, and even
/// forgo a proxy all together on native. I like
/// the organization this provides.
pub enum UserEvent {
    /// Creating a Wgpu Context is async in the web
    #[cfg(target_arch = "wasm32")]
    ContextCreated(WgpuContext),
    /// User uploaded rom
    RomSelected(RomSource),
    // TODO: Shader swap event
}
