use std;
use super::glutin;
use super::gl;
use super::WindowInitData;

pub fn window_init() -> Result<WindowInitData, Box<std::error::Error>> {
    Ok(WindowInitData{
        title : String::from("nice window title"),
        window_size : (800, 600),
        default_close_handler : true,
        default_resize_handler : true,
    })
}

pub struct AppData {
    // add members to this struct that you want to be available for 
    // the lifetime of the application
}

pub fn app_init() -> Result<AppData, Box<std::error::Error>> {
    unsafe {
        gl::ClearColor(0.0, 0.5, 1.0, 1.0);
    }
    
    Ok(AppData{})
}

#[allow(unused_variables)]
pub fn app_frame(data : &mut AppData) {
    unsafe {
        gl::Clear(gl::COLOR_BUFFER_BIT);
    }
}

#[allow(unused_variables)]
pub fn handle_events(event : glutin::Event, keep_running : &mut bool,
data : &mut AppData) {
}