use glutin;
use gl_helpers;
use cgmath;
use image;

use gl_helpers::gl;

mod simple_text;
use std::collections::HashMap;

macro_rules! cstr {
    ($e:expr) => {
        concat!($e, "\0").as_ptr() as *const std::os::raw::c_char
    };
}

#[derive(Debug, Clone)]
struct Error {
    details: String,
}

impl Error {
    #[allow(dead_code)]
    pub fn new<S: Into<String>>(details: S) -> Error {
        Error {
            details: details.into(),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl std::error::Error for Error {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut parameters = HashMap::new();
    while let Some(restart_parameters) = main_1(parameters)? {
        parameters = restart_parameters;
    }
    Ok(())
}

fn main_1(parameters: HashMap<String, String>) -> Result<Option<HashMap<String, String>>, Box<dyn std::error::Error>> {
    let window_title = "nice window title";
    let mut window_size = (800, 600);
    let gl_version = (4, 3);
    let vsync = parameters.get("vsync")
        .map_or(Ok(true), |v| v.parse::<bool>())?;

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
            .with_vsync(vsync);
        glutin::GlWindow::new(window, context, &ev_loop)?
    };

    unsafe {
        use glutin::GlContext;
        gl_window.make_current()?;
        gl::load_with(|sym| gl_window.get_proc_address(sym) as *const _);
        gl::Enable(gl::DEBUG_OUTPUT);
        gl::DebugMessageCallback(gl_debug_callback, std::ptr::null_mut());
        gl::Enable(gl::DEPTH_TEST);
    }

    let mut quit = false;
    let mut keystates = KeyStates::new();
    let debug_printer = simple_text::SimpleTextContext::new();

    let multitext_data = {
        use std::io::BufRead;
        let file = std::fs::File::open("data/shader.multitext")?;
        let file = std::io::BufReader::new(file);
        multitext::parse_lines(file.lines().filter_map(|s| s.ok()))?
    };

    let mut gl_data = GlData::default();

    gl_data.program = {
        let vcode = multitext_data.get("vertex shader").ok_or_else(|| Error::new("vertex shader not found"))?;
        let fcode = multitext_data.get("fragment shader").ok_or_else(|| Error::new("fragment shader not found"))?;

        gl_helpers::ProgramBuilder::new()
            .vertex_shader_code(vcode)
            .fragment_shader_code(fcode)
            .build()?
    };

    {
        let vertices: Vec<f32> = {
            multitext_data.get("vertices")
                .ok_or_else(|| Error::new("vertices not found"))?
                .split(",")
                .filter_map(|e| match e.trim() {
                    "" => None,
                    e => Some(e.parse()),
                })
                .collect::<Result<Vec<_>, _>>()?
        };

        let buffer = gl_helpers::create_buffer(&vertices, gl_helpers::BufferUsage::StaticDraw)?;
        gl_data.buffers.push(buffer);
        
        let components = multitext_data.get("vertex components")
            .ok_or_else(|| Error::new("vertex components not found"))?
            .split(",")
            .filter_map(|e| match e.trim() {
                "" => None,
                e => Some(e.parse()),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let vertex_array = gl_helpers::create_single_buffer_vertex_array(buffer, &components)?;
        let vertex_count: i32 = vertices.len() as i32 / components.iter().sum::<i32>();

        gl_data.vertex_arrays.push(VertexArray { 
            id: vertex_array,
            vertex_count,
        });
    }

    {
        let paths_and_names = multitext_data.get("textures").cloned().unwrap_or_default()
            .split(",")
            .filter_map(|e| match e.trim() {
                "" => None,
                e => {
                    let path = format!("data/{}", e);
                    let name = std::ffi::CString::new(e.split(".").nth(0).unwrap()).unwrap();
                    Some((path, name))
                }
            }).collect::<Vec<_>>();

        for (path, name) in paths_and_names.into_iter() {
            let uniform_location = gl_helpers::get_uniform_location(gl_data.program, name.as_ptr())?;

            let image_data = image::open(path)?.to_rgba();
            let width = image_data.width() as i32;
            let height = image_data.height() as i32;
            let mut id = 0;
            let image_ptr = image_data.as_ptr() as *const std::os::raw::c_void;
            unsafe {
                gl::GenTextures(1, &mut id);
                gl::BindTexture(gl::TEXTURE_2D, id);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
                gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, width, height,
                    0, gl::RGBA, gl::UNSIGNED_BYTE, image_ptr);
            }

            gl_data.textures.push(Texture { width, height, id, uniform_location });
        }

        unsafe {
            gl::UseProgram(gl_data.program);
            for (index, tex) in gl_data.textures.iter().enumerate() {
                gl::Uniform1i(tex.uniform_location, index as i32);
            }
        }
    }

    let mvp_uniform_location = gl_helpers::get_uniform_location(gl_data.program, cstr!("mvp")).ok();
    let time_uniform_location = gl_helpers::get_uniform_location(gl_data.program, cstr!("time")).ok();

    let mut camera = Camera {
        position: cgmath::Point3::new(0.0, 0.0, -5.0),
        angle: cgmath::Deg(0.0),
    };

    let mut time_of_last_update = std::time::Instant::now();
    let mut total_seconds_elapsed = 0.0;
    let mut frame_counter = 0;
    let mut fps_frame_start = 0;
    let mut seconds_elapsed_since_last_fps_measurment = 0.0;
    let mut fps_text = "FPS: ???".to_owned();

    let mut restart = None;

    while !quit {
        ev_loop.poll_events(|event|
            if let glutin::Event::WindowEvent{event, ..} = event {
                use glutin::WindowEvent::*;
                match event {
                    CloseRequested => quit = true,

                    Resized(logical_size) => {
                        let dpi_factor = gl_window.get_hidpi_factor();
                        gl_window.resize(logical_size.to_physical(dpi_factor));
                        unsafe {
                            let w = logical_size.width as i32;
                            let h = logical_size.height as i32;
                            gl::Viewport(0, 0, w, h);
                            window_size = (w as u32, h as u32);
                        }
                    },

                    KeyboardInput{input, ..} => {
                        if let Some(key) = input.virtual_keycode {
                            keystates[key] = input.state == glutin::ElementState::Pressed;

                            if key == glutin::VirtualKeyCode::V && keystates[key] {
                                // restart with vsync toggled, because we can't change vsync on an existing window
                                let mut parameters = HashMap::new();
                                parameters.insert("vsync".to_string(), format!("{}", !vsync));
                                restart = Some(parameters);
                                quit = true;
                            }
                        }
                    },

                    _ => (),
                }
            }
        );

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT|gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(gl_data.program);
            if let Some(loc) = time_uniform_location {
                gl::Uniform1f(loc, total_seconds_elapsed as f32);
            }

            if let Some(loc) = mvp_uniform_location {
                let aspect = (window_size.0 as f32) / (window_size.1 as f32);
                let projection = cgmath::perspective(cgmath::Deg(85.0), aspect, 0.2, 1000.0);
                let move_vector = vector_zx_from_angle(camera.angle);
                let up_vector = cgmath::Vector3::new(0.0, 1.0, 0.0);
                let view = cgmath::Matrix4::look_at_dir(camera.position, move_vector, up_vector);

                let mvp = projection * view;
                use cgmath::Matrix;
                gl::UniformMatrix4fv(loc, 1, gl::FALSE, mvp.as_ptr());
            }

            for (index, tex) in gl_data.textures.iter().enumerate() {
                gl::ActiveTexture(gl::TEXTURE0 + index as u32);
                gl::BindTexture(gl::TEXTURE_2D, tex.id);
            }

            for v in gl_data.vertex_arrays.iter() {
                gl::BindVertexArray(v.id);
                gl::DrawArrays(gl::TRIANGLES, 0, v.vertex_count);
            }
        }

        {
            let x = -1.0 + 40.0 / window_size.0 as f32;
            let y = -1.0 + 40.0 / window_size.1 as f32;
            let xscale = 25.0 / window_size.0 as f32;
            let yscale = 25.0 / window_size.1 as f32;
            debug_printer.draw_text(&fps_text, x, y, xscale, yscale);
        }

        gl_window.swap_buffers()?;

        let seconds_elapsed_this_frame = {
            let now = std::time::Instant::now();
            let time_elapsed_this_frame = now - time_of_last_update;
            time_of_last_update = now;
            time_elapsed_this_frame.as_nanos() as f64 / 1000000000.0
        };

        {
            use glutin::VirtualKeyCode as Vk;
            let speed = 3.0;
            let turnspeed = cgmath::Deg(120.0);

            let move_vector = vector_zx_from_angle(camera.angle);
            let seconds_elapsed = seconds_elapsed_this_frame as f32;

            if keystates[Vk::W] {
                camera.position += move_vector * speed * seconds_elapsed;
            } else if keystates[Vk::S] {
                camera.position -= move_vector * speed * seconds_elapsed;
            }

            if keystates[Vk::Q] {
                let right = vector_zx_from_angle(camera.angle + cgmath::Deg(90.0));
                camera.position += right * speed * seconds_elapsed;
            } else if keystates[Vk::E] {
                let left = vector_zx_from_angle(camera.angle - cgmath::Deg(90.0));
                camera.position += left * speed * seconds_elapsed;
            }

            if keystates[Vk::A] {
                camera.angle += turnspeed * seconds_elapsed;
            } else if keystates[Vk::D] {
                camera.angle -= turnspeed * seconds_elapsed;
            }
        }

        frame_counter += 1;
        total_seconds_elapsed += seconds_elapsed_this_frame;
        seconds_elapsed_since_last_fps_measurment += seconds_elapsed_this_frame;

        if seconds_elapsed_since_last_fps_measurment >= 1.0 {
            let frames_for_this_measurement = frame_counter - fps_frame_start;
            let fps = (frames_for_this_measurement as f64) / seconds_elapsed_since_last_fps_measurment;
            fps_text = format!("FPS: {}", fps);
            fps_frame_start = frame_counter;
            seconds_elapsed_since_last_fps_measurment = 0.0;
        }
    }

    Ok(restart)
}

extern "system" fn gl_debug_callback(
    source: u32, ty: u32, id: u32, severity: u32, length: i32,
    message: *const std::os::raw::c_char, user_param: *mut std::os::raw::c_void)
{
    let _ = (source, ty, id, severity, user_param);

    if severity != gl::DEBUG_SEVERITY_NOTIFICATION {
        unsafe {
            let message = std::slice::from_raw_parts(message as *const u8, length as usize);
            let message = std::str::from_utf8(message).expect("bad opengl error message");
            println!("{}", message);
        }
    }
}

fn vector_zx_from_angle<A: cgmath::Angle<Unitless = f32>>(angle: A) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(angle.sin(), 0.0, angle.cos())
}

#[derive(Debug, Clone, Copy)]
struct Texture {
    pub id: u32,
    pub uniform_location: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Copy)]
struct VertexArray {
    pub id: u32,
    vertex_count: i32,
}

#[derive(Default)]
struct GlData {
    textures: Vec<Texture>,
    program: u32,
    buffers: Vec<u32>,
    vertex_arrays: Vec<VertexArray>,
}

impl Drop for GlData {
    fn drop(&mut self) {
        for e in self.textures.iter() {
            unsafe {
                gl::DeleteTextures(1, &e.id);
            }
        }

        for e in self.vertex_arrays.iter() {
            unsafe {
                gl::DeleteVertexArrays(1, &e.id);
            }
        }

        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteBuffers(self.buffers.len() as i32, self.buffers.as_ptr());
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Camera {
    position: cgmath::Point3<f32>,
    angle: cgmath::Deg<f32>,
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