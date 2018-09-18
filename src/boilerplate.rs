use std;
use glutin;
use gl;
use glutin::dpi::LogicalSize;
use glutin::GlContext;
use gl::types::{GLint};

pub struct WindowInitData {
    pub title : String,
    pub window_size : (i32, i32),
    pub default_close_handler : bool,
    pub default_resize_handler : bool,
}

pub struct InitData {
    pub window_init_data : WindowInitData,
    pub app_data : super::AppData,
}

pub fn execute() -> Result<(),Box<std::error::Error>> {
    let InitData { window_init_data, mut app_data } =
        super::window_init()?;

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
    let gl_window = glutin::GlWindow::new(window, context, &ev_loop)?;

    unsafe {
        gl_window.make_current()?;
        gl::load_with(|sym| gl_window.get_proc_address(sym) as *const _);
    }

    super::gl_init(&mut app_data)?;

    let start_time = std::time::Instant::now();

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
                super::handle_events(event, &mut running, &mut app_data);
            }
        });

        let elapsed = std::time::Instant::now() - start_time;
        super::frame(elapsed, &mut app_data);

        gl_window.swap_buffers()?;
    }

    super::shutdown(&mut app_data);

    Ok(())
}
