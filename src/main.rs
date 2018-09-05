extern crate glutin;
extern crate gl;
extern crate simple_error;

mod gl_helpers;
mod boilerplate;
use boilerplate::WindowInitData;

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

pub fn window_init() -> Result<WindowInitData, Box<std::error::Error>> {
    Ok(boilerplate::WindowInitData{
        title : String::from("nice window title"),
        window_size : (800, 800),
        default_close_handler : true,
        default_resize_handler : true,
    })
}

#[allow(dead_code)]
pub struct AppData {
}

pub fn init() -> Result<AppData, Box<std::error::Error>> {
    unsafe {
        gl::ClearColor(0.2, 0.2, 0.4, 1.0);
        Ok(AppData{})
    }
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

