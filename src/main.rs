use glutin;
use gl;

mod gl_helpers;

fn main() -> Result<(),Box<std::error::Error>> {
    use glutin::GlContext;
    let window_title = "nice window title";
    let window_size = (800,800);
    let gl_version = (4,3);

    let mut ev_loop = glutin::EventsLoop::new();
    let gl_window = {
        let window = glutin::WindowBuilder::new()
            .with_title(window_title)
            .with_dimensions(window_size.into());
        let gl_request = glutin::GlRequest::Specific(
            glutin::Api::OpenGl, gl_version);
        let context = glutin::ContextBuilder::new()
            .with_gl_profile(glutin::GlProfile::Core)
            .with_gl(gl_request)
            .with_vsync(true);
        glutin::GlWindow::new(window, context, &ev_loop)?
    };

    unsafe {
        gl_window.make_current()?;
        gl::load_with(|sym| gl_window.get_proc_address(sym) as *const _);
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(gl_debug_callback, std::ptr::null_mut());
    }

    let program = {
        let code = {
            let file = std::fs::File::open("res/shader.multitext")?;
            let file = std::io::BufReader::new(file);
            use std::io::BufRead;
            multitext::parse_lines(file.lines().filter_map(|r| r.ok()))?
        };

        let vcode = code.get("vertex shader").expect("no vertex shader found").clone();
        let fcode = code.get("fragment shader").expect("no fragment shader found").clone();
        gl_helpers::ProgramBuilder::new()
            .vertex_shader_code(vcode)
            .fragment_shader_code(fcode)
            .build()?
    };

    let buffer = {
        let vertices: [f32;15] = [
            0., 0.8, 1.0, 0.0, 0.0,
            0.8, -0.8, 0.0, 1.0, 0.0,
            -0.8, -0.8, 0.0, 0.0, 1.0,
        ];
        gl_helpers::create_buffer(&vertices, gl_helpers::BufferUsage::StaticDraw)?
    };

    let vao = gl_helpers::create_single_buffer_vertex_array(buffer, &[2,3])?;

    // let start_time = std::time::Instant::now();

    let mut running = true;
    while running {
        ev_loop.poll_events(|event| {
            match event {
                glutin::Event::WindowEvent{event, ..} => match event {
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
                },

                _ => (),
            }
        });

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        gl_window.swap_buffers()?;
    }

    Ok(())
}

extern "system" fn gl_debug_callback(
    source: gl::types::GLenum, ty: gl::types::GLenum, id: u32, severity: gl::types::GLenum, length: i32,
    message: *const std::os::raw::c_char, user_param: *mut std::os::raw::c_void)
{
    let _ = (source, ty, id, severity, user_param);

    unsafe {
        let message = std::slice::from_raw_parts(message as *const u8, length as usize);
        let message = std::str::from_utf8(message).expect("bad opengl error message");
        println!("{}", message);
    }
}
