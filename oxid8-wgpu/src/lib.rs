use cfg_if::cfg_if;
use winit::event_loop::EventLoop;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{app::App, event::UserEvent};

mod app;
mod event;
mod geometry;
mod texture;

#[cfg(not(target_arch = "wasm32"))]
pub struct Config {
    pub rom_path: String,
}

pub fn run(#[cfg(not(target_arch = "wasm32"))] config: Config) -> anyhow::Result<()> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_log::init_with_level(log::Level::Info).unwrap_throw();
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::<UserEvent>::with_user_event().build()?;
    let mut app = App::new(
        &event_loop,
        #[cfg(not(target_arch = "wasm32"))]
        config,
    );

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(app);
        } else {
            event_loop.run_app(&mut app)?;
        }
    }

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    console_error_panic_hook::set_once();
    run().unwrap_throw();

    Ok(())
}
