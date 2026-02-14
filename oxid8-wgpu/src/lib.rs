//! Chip-8 Interpreter windowed natively and on the web.

mod app;
mod event;
mod geometry;
mod texture;
mod wgpu_context;

use cfg_if::cfg_if;
use winit::event_loop::{EventLoop, EventLoopProxy};

use std::cell::OnceCell;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

use crate::{app::App, event::UserEvent};

thread_local! {
    static EVENT_LOOP_PROXY: OnceCell<EventLoopProxy<UserEvent>> = OnceCell::new();
}

#[cfg(not(target_arch = "wasm32"))]
pub struct Config {
    pub rom_path: PathBuf,
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
    #[allow(unused_mut)]
    let mut app = App::new(
        &event_loop,
        #[cfg(not(target_arch = "wasm32"))]
        config,
    );
    EVENT_LOOP_PROXY.with(|cell| cell.set(app.proxy.clone()).unwrap());

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

/// For Mobile Phones
#[cfg(target_arch = "wasm32")]
fn map_key(key_code: &str) -> Option<winit::keyboard::KeyCode> {
    use winit::keyboard::KeyCode::*;
    Some(match key_code {
        "Digit1" => Digit1,
        "Digit2" => Digit2,
        "Digit3" => Digit3,
        "Digit4" => Digit4,
        "KeyQ" => KeyQ,
        "KeyW" => KeyW,
        "KeyE" => KeyE,
        "KeyR" => KeyR,
        "KeyA" => KeyA,
        "KeyS" => KeyS,
        "KeyD" => KeyD,
        "KeyF" => KeyF,
        "KeyZ" => KeyZ,
        "KeyX" => KeyX,
        "KeyC" => KeyC,
        "KeyV" => KeyV,
        _ => return None,
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn key_event(key_code: &str, val: bool) {
    if let Some(key_code) = map_key(key_code) {
        EVENT_LOOP_PROXY.with(|cell| {
            if let Some(proxy) = cell.get() {
                let _ = proxy.send_event(UserEvent::VirtualKey(key_code, val));
            }
        })
    }
}
