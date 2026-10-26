//! GUI v0.1: a winit window + OpenGL (via glow), background painted with
//! theme.bg.
//!
//! This is a scaffold: the real grid rendering pipeline will come from the
//! Zig module `zuchu-render` (glyph atlas + compositing). This file owns the
//! OpenGL context and the main event loop.

use glow::HasContext;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::{Window, WindowId};

use crate::config::Config;

struct App {
    cfg: Config,
    window: Option<Window>,
    gl: Option<glow::Context>,
    bg: [f32; 3],
}

impl App {
    fn new(cfg: Config) -> Self {
        let bg = [cfg.theme.bg.0, cfg.theme.bg.1, cfg.theme.bg.2];
        App { cfg, window: None, gl: None, bg }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        let attrs = Window::default_attributes()
            .with_title("zuchu v0.1")
            .with_inner_size(winit::dpi::LogicalSize::new(self.cfg.cols * 9, self.cfg.rows * 18));
        let window = match el.create_window(attrs) {
            Ok(w) => w,
            Err(e) => { eprintln!("zuchu: failed to create window: {e}"); el.exit(); return; }
        };

        // OpenGL context setup (the real EGL/GLX loader + surface handling
        // lands in v0.2; for now this is a placeholder)
        let gl = unsafe {
            glow::Context::from_loader_function(|_s| {
                std::ptr::null() // TODO: real loader via EGL in v0.2
            })
        };

        unsafe {
            gl.clear_color(self.bg[0], self.bg[1], self.bg[2], 1.0);
        }

        self.window = Some(window);
        self.gl = Some(gl);
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::Resized(_) => {
                // TODO: update viewport + reflow the grid
            }
            WindowEvent::KeyboardInput { .. } => {
                // TODO: translate keys into bytes -> PtySession::write
            }
            WindowEvent::CloseRequested => el.exit(),
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let (Some(window), Some(gl)) = (self.window.as_ref(), self.gl.as_ref()) {
            unsafe {
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
            window.request_redraw();
        }
    }
}

pub fn run(cfg: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new(cfg.clone());
    event_loop.run_app(&mut app)?;
    Ok(())
}
