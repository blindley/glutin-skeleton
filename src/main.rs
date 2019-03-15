extern crate glutin;
extern crate gl;
extern crate simple_error;

mod gl_helpers;

fn main() -> Result<(),Box<std::error::Error>> {
    use glutin::GlContext;
    let window_title = "nice window title";
    let window_size = (800, 800);

    let mut ev_loop = glutin::EventsLoop::new();
    let gl_window = {
        let window = glutin::WindowBuilder::new()
            .with_title(window_title)
            .with_dimensions(window_size.into());
        let context = glutin::ContextBuilder::new()
            .with_vsync(true);
        glutin::GlWindow::new(window, context, &ev_loop)?
    };

    unsafe {
        gl_window.make_current()?;
        gl::load_with(|sym| gl_window.get_proc_address(sym) as *const _);
    }

    // let start_time = std::time::Instant::now();

    let mut running = true;
    while running {
        ev_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{event, ..} => {
                    match event {
                        glutin::WindowEvent::CloseRequested => running = false,

                        glutin::WindowEvent::Resized(logical_size) => {
                            let dpi_factor = gl_window.get_hidpi_factor();
                            gl_window.resize(logical_size.to_physical(dpi_factor));
                            unsafe {
                                let w = logical_size.width as i32;
                                let h = logical_size.height as i32;
                                gl::Viewport(0, 0, w, h);
                            }
                        },

                        _ => (),
                    }
                },

                _ => (),
            }
        });

        gl_window.swap_buffers()?;
    }

    Ok(())
}