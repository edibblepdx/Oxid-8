use std::{path::PathBuf, sync::Arc, time::Instant};

#[cfg(not(target_arch = "wasm32"))]
use crate::Config;
use crate::{event::UserEvent, wgpu_context::WgpuContext};

use oxid8_core::{Oxid8, TIMER_TICK};
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId},
};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub enum State {
    Suspended,
    Resumed {
        emu: Oxid8,
        last_frame: Option<Instant>,
    },
}

impl State {
    pub fn handle_key(&mut self, key_code: KeyCode, val: bool) {
        use KeyCode::*;

        if let State::Resumed { emu, .. } = self {
            match key_code {
                Digit1 => emu.set_key(0x1, val),
                Digit2 => emu.set_key(0x2, val),
                Digit3 => emu.set_key(0x3, val),
                Digit4 => emu.set_key(0xC, val),
                KeyQ => emu.set_key(0x4, val),
                KeyW => emu.set_key(0x5, val),
                KeyE => emu.set_key(0x6, val),
                KeyR => emu.set_key(0xD, val),
                KeyA => emu.set_key(0x7, val),
                KeyS => emu.set_key(0x8, val),
                KeyD => emu.set_key(0x9, val),
                KeyF => emu.set_key(0xE, val),
                KeyZ => emu.set_key(0xA, val),
                KeyX => emu.set_key(0x0, val),
                KeyC => emu.set_key(0xB, val),
                KeyV => emu.set_key(0xF, val),
                _ => (),
            }
        }
    }
}

pub struct App {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    ctx: Option<WgpuContext>,
    state: State,
    #[cfg(not(target_arch = "wasm32"))]
    config: Config,
    #[cfg(target_arch = "wasm32")]
    document: Option<web_sys::Document>,
}

impl App {
    pub fn new(
        event_loop: &EventLoop<UserEvent>,
        #[cfg(not(target_arch = "wasm32"))] config: Config,
    ) -> Self {
        Self {
            proxy: event_loop.create_proxy(),
            ctx: None,
            state: State::Suspended,
            #[cfg(not(target_arch = "wasm32"))]
            config,
            #[cfg(target_arch = "wasm32")]
            document: None,
        }
    }

    // WARN: check this implementation
    pub fn resume(&mut self, rom_path: PathBuf) {
        if let Some(ctx) = &self.ctx {
            let mut emu = Oxid8::default();
            ctx.texture.update(&ctx.queue, emu.screen_ref());

            emu.load_font();

            // WARN: what to do if this fails?
            if emu.load_rom_path(&rom_path).is_ok() {
                self.state = State::Resumed {
                    emu,
                    last_frame: None,
                };
                // WARN: test
                println!("rom loaded");
            }
        }
    }
}

/*
fn temp() {
    use web_sys::HtmlInputElement;

    const INPUT_ID: &str = "input";

    let input = document.get_element_by_id(INPUT_ID).unwrap_throw();
    let html_input_element = input.unchecked_into::<HtmlInputElement>();
}
*/

impl ApplicationHandler<UserEvent> for App {
    /// Emitted when the application has been resumed.
    /// Initialize graphics context and create a window after first resumed event.
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes().with_title("Oxid-8");

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));

            self.document = Some(document);
        }

        // Create window object
        #[rustfmt::skip]
        let window = Arc::new(
            event_loop
                .create_window(window_attributes)
                .unwrap(),
        );

        #[cfg(not(target_arch = "wasm32"))]
        {
            // Create WgpuContext
            let ctx = pollster::block_on(WgpuContext::new(window.clone())).unwrap();
            self.ctx = Some(ctx);

            // Set App state to Resumed
            assert!(
                self.proxy
                    .send_event(UserEvent::Resumed(self.config.rom_path.clone()))
                    .is_ok()
            );

            window.request_redraw();
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Create WgpuContext
            let proxy = self.proxy.clone();
            wasm_bindgen_futures::spawn_local(async move {
                assert!(
                    proxy
                        .send_event(UserEvent::ContextCreated(
                            WgpuContext::new(window)
                                .await
                                .expect("Failed to create window.")
                        ))
                        .is_ok()
                )
            });

            // request redraw in user_event
        }
    }

    /// Emitted when the OS sends an event to a winit window.
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        let ctx = match self.ctx.as_mut() {
            Some(canvas) => canvas,
            None => return,
        };

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let State::Resumed {
                    emu, last_frame, ..
                } = &mut self.state
                {
                    match last_frame {
                        Some(last) if last.elapsed() >= TIMER_TICK => {
                            *last_frame = Some(Instant::now());
                            if emu.next_frame().is_ok() {
                                // Update texture
                                ctx.texture.update(&ctx.queue, emu.screen_ref());
                            }
                        }
                        None => *last_frame = Some(Instant::now()),
                        _ => (), // This case is necessary.
                    }
                }
                ctx.render();
                // Emits a new redraw requested event.
                ctx.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                ctx.resize(size);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state,
                        ..
                    },
                ..
            } => {
                if let State::Resumed { .. } = &mut self.state {
                    // match key state
                    match state {
                        ElementState::Pressed => self.state.handle_key(key_code, true),
                        ElementState::Released => self.state.handle_key(key_code, false),
                    }
                }
            }
            _ => (),
        }
    }

    /// Emitted when an event is sent from EventLoopProxy::send_event.
    #[allow(unused_mut)]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut event: UserEvent) {
        use UserEvent::*;

        match event {
            #[cfg(target_arch = "wasm32")]
            ContextCreated(mut ctx) => {
                if !ctx.is_surface_configured {
                    // Configure surface for the first time on web
                    ctx.resize(ctx.window.inner_size());
                    // Already redraw after resizing so this might be pointless
                    ctx.window.request_redraw();
                }
                self.ctx = Some(ctx);

                // Install input event handler to upload roms on web
                use wasm_bindgen::JsCast;
                use web_sys::{FileReader, HtmlInputElement, js_sys::Uint8Array};

                const INPUT_ID: &str = "input";

                let input = if let Some(document) = &self.document {
                    document.get_element_by_id(INPUT_ID).unwrap_throw()
                } else {
                    panic!("no document");
                };
                let html_input_element = input.unchecked_into::<HtmlInputElement>();

                // TODO: Handle the Err variants

                // Input onchange handler
                let onchange = Closure::<dyn FnMut(_)>::new({
                    let proxy = self.proxy.clone();
                    move |event: web_sys::Event| {
                        if let Some(file) = event
                            .current_target()
                            .and_then(|target| target.dyn_into::<HtmlInputElement>().ok())
                            .and_then(|input| input.files())
                            .and_then(|files| files.item(0))
                        {
                            let reader = FileReader::new().unwrap_throw();
                            reader.read_as_array_buffer(&file);

                            // Reader onload handler
                            let onload = Closure::<dyn FnMut(_)>::new({
                                let reader = reader.clone();
                                move |_event: web_sys::ProgressEvent| {
                                    if let Ok(result) = reader.result() {
                                        let data = Uint8Array::new(&result).to_vec();
                                        web_sys::console::log_1(
                                            &format!("ROM file uploaded: {} bytes.", data.len())
                                                .into(),
                                        );
                                    }
                                }
                            });

                            reader.set_onload(Some(onload.as_ref().unchecked_ref()));

                            // WARN: Leaking memory in rust, but we want a global handler.
                            onload.forget();
                        }
                    }
                });

                html_input_element
                    .add_event_listener_with_callback("change", onchange.as_ref().unchecked_ref());

                // WARN: Leaking memory in rust, but we want a global handler.
                onchange.forget();
            }
            Resumed(rom_path) => self.resume(rom_path),
        }
    }
}
