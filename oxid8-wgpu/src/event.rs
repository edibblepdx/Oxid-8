use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use crate::app::WgpuContext;

pub enum UserEvent {
    #[cfg(target_arch = "wasm32")]
    ContextCreated(WgpuContext),
    Resumed(PathBuf),
    //Shader update
}
