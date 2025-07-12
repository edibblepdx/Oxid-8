use cfg_if::cfg_if;
use winit::event_loop::EventLoop;

use crate::app::{App, State};

mod app;
mod geometry;

pub fn run() -> anyhow::Result<()> {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            console_log::init_with_level(log::Level::Info).unwrap_throw();
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::<State>::with_user_event().build()?;
    let mut app = App::new(
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );

    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::EventLoopExtWebSys;
            event_loop.spawn_app(&mut app)?;
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
