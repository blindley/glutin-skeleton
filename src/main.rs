use glutin;
use gl_helpers;
use cgmath;
use image;

use gl_helpers::gl;

mod simple_text;

macro_rules! cstr {
    ($e:expr) => {
        concat!($e, "\0").as_ptr() as *const std::os::raw::c_char
    };
}

fn main() -> Result<(),Box<dyn std::error::Error>> {
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
        gl::Enable(gl::DEPTH_TEST);
    }

    let text_data = {
        let file = std::fs::File::open("data/shader.multitext")?;
        let file = std::io::BufReader::new(file);
        use std::io::BufRead;
        multitext::parse_lines(file.lines().filter_map(|r| r.ok()))?
    };

    let program = {
        let vcode = text_data.get("vertex shader").expect("no vertex shader found").clone();
        let fcode = text_data.get("fragment shader").expect("no fragment shader found").clone();
        gl_helpers::ProgramBuilder::new()
            .vertex_shader_code(vcode)
            .fragment_shader_code(fcode)
            .build()?
    };

    let vertices: Vec<f32> = {
        let vertices = text_data.get("vertices").expect("no vertices found").clone();
        vertices.split(",").filter_map(|e|
            match e.trim() {
                "" => None,
                e => Some(e.parse().unwrap())
            }
        ).collect()
    };

    let components: Vec<i32> = {
        let components = text_data.get("vertex components").expect("no vertex component data found").clone();
        components.split(",").filter_map(|e|
            match e.trim() {
                "" => None,
                e => Some(e.parse().unwrap())
            }
        ).collect()
    };

    let vertex_count: i32 = (vertices.len() as i32) / components.iter().sum::<i32>();

    let buffer = gl_helpers::create_buffer(&vertices, gl_helpers::BufferUsage::StaticDraw)?;

    let vao = gl_helpers::create_single_buffer_vertex_array(buffer, &components)?;

    let uniform_mvp = gl_helpers::get_uniform_location(program, cstr!("mvp"))?;
    let uniform_texture = gl_helpers::get_uniform_location(program, cstr!("utexture"))?;

    let texture_id = unsafe {
        let image_data = image::open("data/cube.png")?.to_rgba();
        let mut texture_id = 0;
        gl::GenTextures(1, &mut texture_id);
        gl::BindTexture(gl::TEXTURE_2D, texture_id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
        gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32,
            image_data.width() as i32, image_data.height() as i32,
            0, gl::RGBA, gl::UNSIGNED_BYTE,
            image_data.as_ptr() as *const std::os::raw::c_void);
        texture_id
    };

    let mut data = AppData {
        quit: false,
        gl_window, window_size, program, buffer, vao, texture_id,
        uniform_mvp, uniform_texture,
        keystates: KeyStates::new(),
        player_position: cgmath::Point3::new(0.0, 0.0, 10.0),
        player_direction: cgmath::Deg(180.0),
        player_move_vector: cgmath::Vector3::new(0.0, 0.0, 0.0),
    };

    data.fix_move_vector();

    let up_vector = cgmath::Vector3::new(0.0, 1.0, 0.0);

    let text_context = simple_text::SimpleTextContext::new();

    let mut last_update = std::time::Instant::now();
    let mut fps_frame_counter = 0;
    let mut fps_elapsed_seconds = 0.0;
    let mut fps_text = "fps: 0".to_owned();

    while !data.quit {
        ev_loop.poll_events(|event| data.handle_event(event));

        fps_frame_counter += 1;
        let now = std::time::Instant::now();
        let elapsed = now - last_update;
        let elapsed_seconds = elapsed.as_millis() as f32 / 1000.0;
        fps_elapsed_seconds += elapsed_seconds;
        last_update = now;

        {
            use glutin::VirtualKeyCode as Vk;
            let speed = 3.0;
            let turnspeed = cgmath::Deg(120.0);
            if data.keystates[Vk::W] {
                data.player_position += data.player_move_vector * speed * elapsed_seconds;
            } else if data.keystates[Vk::S] {
                data.player_position -= data.player_move_vector * speed * elapsed_seconds;
            }

            if data.keystates[Vk::Q] {
                let right = vector_zx_from_angle(data.player_direction + cgmath::Deg(90.0));
                data.player_position += right * speed * elapsed_seconds;
            } else if data.keystates[Vk::E] {
                let left = vector_zx_from_angle(data.player_direction - cgmath::Deg(90.0));
                data.player_position += left * speed * elapsed_seconds;
            }

            if data.keystates[Vk::A] {
                data.player_direction += turnspeed * elapsed_seconds;
                data.fix_move_vector();
            } else if data.keystates[Vk::D] {
                data.player_direction -= turnspeed * elapsed_seconds;
                data.fix_move_vector();
            }
        }

        let aspect = (window_size.0 as f32) / (window_size.1 as f32);
        let projection = cgmath::perspective(cgmath::Deg(85.0), aspect, 0.2, 1000.0);
        let view = cgmath::Matrix4::look_at_dir(data.player_position,
            data.player_move_vector, up_vector);

        let mvp = projection * view;

        unsafe {
            use cgmath::Matrix;
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(data.program);
            gl::UniformMatrix4fv(data.uniform_mvp, 1, gl::FALSE, mvp.as_ptr());
            gl::BindVertexArray(data.vao);
            gl::ActiveTexture(gl::TEXTURE0);
            gl::Uniform1i(data.uniform_texture, 0);
            gl::BindTexture(gl::TEXTURE_2D, data.texture_id);
            gl::DrawArrays(gl::TRIANGLES, 0, vertex_count);
        }

        if fps_frame_counter == 100 {
            let fps = 100.0 / fps_elapsed_seconds;
            fps_text = format!("fps: {}", fps);
            fps_frame_counter = 0;
            fps_elapsed_seconds = 0.0;
        }

        text_context.draw_text(&fps_text, -0.95, -0.95, 0.035, 0.035);

        data.gl_window.swap_buffers()?;
    }

    unsafe {
        gl::DeleteBuffers(1, &data.buffer);
        gl::DeleteVertexArrays(1, &data.vao);
        gl::DeleteProgram(data.program);
    }

    Ok(())
}

extern "system" fn gl_debug_callback(
    source: u32, ty: u32, id: u32, severity: u32, length: i32,
    message: *const std::os::raw::c_char, user_param: *mut std::os::raw::c_void)
{
    let _ = (source, ty, id, severity, user_param);

    unsafe {
        let message = std::slice::from_raw_parts(message as *const u8, length as usize);
        let message = std::str::from_utf8(message).expect("bad opengl error message");
        println!("{}", message);
    }
}

fn vector_zx_from_angle<A: cgmath::Angle<Unitless = f32>>(angle: A) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(angle.sin(), 0.0, angle.cos())
}

pub struct AppData {
    quit: bool,
    gl_window: glutin::GlWindow,
    window_size: (u32, u32),

    program: u32,
    buffer: u32,
    vao: u32,
    texture_id: u32,
    uniform_mvp: i32,
    uniform_texture: i32,

    keystates: KeyStates,

    player_position: cgmath::Point3<f32>,
    player_direction: cgmath::Deg<f32>,
    player_move_vector: cgmath::Vector3<f32>,
}

impl AppData {
    pub fn handle_event(&mut self, event: glutin::Event) {
        match event {
            glutin::Event::WindowEvent{event, ..} => match event {
                glutin::WindowEvent::CloseRequested => self.quit = true,

                glutin::WindowEvent::Resized(logical_size) => {
                    let dpi_factor = self.gl_window.get_hidpi_factor();
                    self.gl_window.resize(logical_size.to_physical(dpi_factor));
                    unsafe {
                        let w = logical_size.width as i32;
                        let h = logical_size.height as i32;
                        gl::Viewport(0, 0, w, h);
                        self.window_size = (w as u32, h as u32);
                    }
                },

                glutin::WindowEvent::KeyboardInput{input, ..} => {
                    if let Some(key) = input.virtual_keycode {
                        self.keystates[key] = input.state == glutin::ElementState::Pressed;
                    }
                },

                _ => (),
            },

            _ => (),
        }
    }

    pub fn fix_move_vector(&mut self) {
        self.player_move_vector = vector_zx_from_angle(self.player_direction);
    }
}

pub struct KeyStates {
    pressed: [bool;188],
}

impl KeyStates {
    pub fn new() -> KeyStates {
        KeyStates {
            pressed: [false;188],
        }
    }
}

impl std::ops::Index<glutin::VirtualKeyCode> for KeyStates {
    type Output = bool;
    fn index(&self, key: glutin::VirtualKeyCode) -> &bool {
        &self.pressed[key as usize]
    }
}

impl std::ops::IndexMut<glutin::VirtualKeyCode> for KeyStates {
    fn index_mut(&mut self, key: glutin::VirtualKeyCode) -> &mut bool {
        &mut self.pressed[key as usize]
    }
}