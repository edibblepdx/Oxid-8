use std::{
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{Config, event::UserEvent, wgpu_context::WgpuContext};

use oxid8_core::Oxid8;
use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::{ActiveEventLoop, EventLoop},
    //keyboard::{KeyCode, PhysicalKey},
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

pub struct App {
    proxy: winit::event_loop::EventLoopProxy<UserEvent>,
    ctx: Option<WgpuContext>,
    state: State,
    #[cfg(not(target_arch = "wasm32"))]
    config: Config,
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
            let ctx = pollster::block_on(WgpuContext::new(window.clone()));
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
                    proxy.send_event(
                        UserEvent::ContextCreated(WgpuContext::new(window).await).is_ok()
                    )
                )
            });

            // request redraw in user_event
        }

        // TODO: Load rom from config or on wasm spawn thread local
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
                use State::*;

                match &mut self.state {
                    Suspended => (),
                    Resumed {
                        emu, last_frame, ..
                    } => {
                        *last_frame = Some(Instant::now());
                        if emu.next_frame().is_ok() {
                            // Update texture
                            ctx.texture.update(&ctx.queue, emu.screen_ref());
                        }
                    }
                }
                // TODO: move this into Resumed when done testing
                ctx.render();
                // Emits a new redraw requested event.
                ctx.window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                ctx.resize(size);
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
            ContextCreated(ctx) => {
                if !event.is_surface_configured {
                    // Configure surface for the first time on web
                    event.resize(event.window.inner_size());
                    // Already redraw after resizing so this might be pointless
                    event.window.request_redraw();
                }
                self.ctx = Some(ctx);
            }
            Resumed(rom_path) => self.resume(rom_path),
        }
    }
}
