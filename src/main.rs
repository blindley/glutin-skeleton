extern crate glutin;
extern crate gl;
extern crate simple_error;

mod gl_helpers;
mod boilerplate;
use boilerplate::{InitData, WindowInitData};

fn main() {
    match boilerplate::execute() {
        Err(e) => {
            println!("Error: {}", e);
        },
        _ => (),
    }
}

#[allow(unused_imports)]
use gl::types::{GLuint, GLint, GLchar, GLsizei};

pub fn window_init() -> Result<InitData, Box<std::error::Error>> {
    Ok(InitData {
        window_init_data : WindowInitData{
            title : String::from("nice window title"),
            window_size : (800, 800),
            default_close_handler : true,
            default_resize_handler : true,
        },
        app_data : AppData {}
    })
}

#[allow(dead_code)]
pub struct AppData {
}

#[allow(unused_variables)]
pub fn gl_init(app_data : &mut AppData) -> Result<(), Box<std::error::Error>> {
    unsafe {
        gl::ClearColor(0.2, 0.2, 0.4, 1.0);
    }
    Ok(())
}

#[allow(unused_variables)]
pub fn frame(total_elapsed : std::time::Duration, data : &mut AppData) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

#[allow(unused_variables)]
pub fn shutdown(data : &mut AppData) {
}

#[allow(unused_variables)]
pub fn handle_events(event : glutin::Event, keep_running : &mut bool,
data : &mut AppData) {
}

