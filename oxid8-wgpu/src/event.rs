use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use crate::wgpu_context::WgpuContext;

pub enum UserEvent {
    #[cfg(target_arch = "wasm32")]
    ContextCreated(WgpuContext),
    Resumed(PathBuf),
    //Shader update
}
