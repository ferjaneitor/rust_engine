// src/graphics/window.rs

use glutin::{
    dpi::LogicalSize,
    event_loop::EventLoop,
    window::WindowBuilder,
    ContextBuilder,
    ContextWrapper,
    PossiblyCurrent,
};
use glutin::window::Window as GlutinWindow;

pub struct Window {
    pub context: ContextWrapper<PossiblyCurrent, GlutinWindow>,
}

impl Window {
    pub fn new(title: &str, width: u32, height: u32, event_loop: &EventLoop<()>) 
        -> Result<Self, String> 
    {
        let wb = WindowBuilder::new()
            .with_title(title)
            .with_inner_size(LogicalSize::new(width, height));

        let windowed_context = ContextBuilder::new()
            .with_vsync(true)
            .build_windowed(wb, event_loop)
            .map_err(|e| format!("Error build_windowed: {:?}", e))?;

        // Activar el contexto
        let context = unsafe {
            windowed_context.make_current()
                .map_err(|(_, e)| format!("Error make_current: {:?}", e))?
        };

        // Cargar funciones de OpenGL
        gl::load_with(|s| context.get_proc_address(s) as *const _);

        // Config inicial
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::ClearColor(0.1, 0.2, 0.3, 1.0);
        }

        Ok(Self {
            context
        })
    }

    pub fn request_redraw(&self) {
        self.context.window().request_redraw();
    }

    pub fn resize(&self, new_size: glutin::dpi::PhysicalSize<u32>) {
        self.context.resize(new_size);
        unsafe {
            gl::Viewport(0, 0, new_size.width as i32, new_size.height as i32);
        }
    }
}
