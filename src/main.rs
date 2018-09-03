extern crate glutin;
extern crate gl;
extern crate simple_error;

mod application;
mod gl_helpers;

use glutin::dpi::LogicalSize;
use glutin::GlContext;
use gl::types::{GLint};

fn main() {
    let window_init_data = application::window_init().unwrap();

    let mut ev_loop = glutin::EventsLoop::new();
    let window = {
        let width = window_init_data.window_size.0 as f64;
        let height = window_init_data.window_size.1 as f64;
        glutin::WindowBuilder::new()
            .with_title(window_init_data.title.as_str())
            .with_dimensions(LogicalSize::new(width, height))
    };

    let context = glutin::ContextBuilder::new()
        .with_vsync(true);
    let gl_window = glutin::GlWindow::new(window, context, &ev_loop).unwrap();

    unsafe {
        gl_window.make_current().unwrap();
        gl::load_with(|sym| gl_window.get_proc_address(sym) as *const _);
    }

    let mut app_data = application::app_init().unwrap();

    let mut running = true;
    while running {
        ev_loop.poll_events(|event| {
            let mut event_handled = false;
            use glutin::Event::WindowEvent;
            use glutin::WindowEvent::{CloseRequested, Resized};
            if window_init_data.default_close_handler {
                if let WindowEvent{event : CloseRequested, ..} = event {
                    running = false;
                    event_handled = true;
                }
            }

            if window_init_data.default_resize_handler {
                if let WindowEvent{event : Resized(logical_size), ..} = event {
                    let dpi_factor = gl_window.get_hidpi_factor();
                    gl_window.resize(logical_size.to_physical(dpi_factor));
                    unsafe {
                        let w = logical_size.width as GLint;
                        let h = logical_size.height as GLint;
                        gl::Viewport(0, 0, w, h);
                    }
                    event_handled = true;
                }
            }

            if !event_handled {
                application::handle_events(event, &mut running, &mut app_data);
            }
        });

        application::app_frame(&mut app_data);

        gl_window.swap_buffers().unwrap();
    }

    application::app_shutdown(&mut app_data);
}

pub struct WindowInitData {
    title : String,
    window_size : (i32, i32),
    default_close_handler : bool,
    default_resize_handler : bool,
}
