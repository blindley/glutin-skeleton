#![allow(dead_code)]

use std;
use gl;

#[allow(unused_imports)]
use gl::types::{GLuint, GLint, GLsizei, GLchar, GLenum, GLsizeiptr, GLvoid};

type BoxResult<T> = std::result::Result<T, Box<std::error::Error>>;
pub type Error = simple_error::SimpleError;
pub type Result<T> = std::result::Result<T, Error>;

macro_rules! assert_or_error {
    ($cond:expr) => (
        if !$cond {
            let msg = format!("assertion failed: {}", stringify!($cond));
            Err(Error::new(msg))
        } else {
            Ok(())
        }
    );
    ($cond:expr, $($arg:tt)+) => (
        if !$cond {
            let msg = format!($($arg)+);
            Err(Error::new(msg))
        } else {
            Ok(())
        }
    );
}

pub struct ProgramBuilder {
    code: ShaderCode,
}

impl ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        ProgramBuilder {
            code: ShaderCode::new(),
        }
    }

    pub fn vertex_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.code[ShaderType::Vertex] = Some(code.into());
        self
    }

    pub fn fragment_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.code[ShaderType::Fragment] = Some(code.into());
        self
    }

    pub fn code(&mut self, code : &ShaderCode) -> &mut ProgramBuilder {
        for i in ShaderType::each().iter() {
            if let Some(ref code) = code[*i] {
                self.code[*i] = Some(code.clone());
            }
        }
        self
    }

    pub fn build(&self) -> Result<GLuint> {
        let mut shaders = Vec::new();
        for shader_type in ShaderType::each().iter() {
            if let Some(ref code) = self.code[*shader_type] {
                shaders.push(compile_shader(&code, *shader_type)?);
            }
        }

        create_program(&shaders, true)
    }
}

pub fn compile_shader(code : &str, shader_type : ShaderType) -> Result<GLuint> {
    unsafe {
        let gl_type = shader_type.as_gl_enum();
        let shader = gl::CreateShader(gl_type);
        let len = code.len() as GLint;
        let code_ptr = code.as_ptr() as *const GLchar;
        gl::ShaderSource(shader, 1, &code_ptr, &len);
        gl::CompileShader(shader);

        let mut success = std::mem::uninitialized();
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            let mut loglen = std::mem::uninitialized();
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut loglen);
            let buffer_len = (loglen - 1) as usize;
            let mut buffer : Vec<u8> = Vec::with_capacity(buffer_len + 1);
            gl::GetShaderInfoLog(shader, loglen, std::ptr::null_mut(),
                buffer.as_ptr() as *mut GLchar);
            buffer.set_len(loglen as usize - 1);
            let log = String::from_utf8(buffer).unwrap();
            
            let shader_name = shader_type.short_name();
            let message = format!("error in {} shader : {}", shader_name, log);

            gl::DeleteShader(shader);
            Err(Error::new(message))
        } else {
            Ok(shader)
        }
    }
}

pub fn create_program(shaders : &[GLuint], delete_shaders : bool)
    -> Result<GLuint>
{
    unsafe {
        let program = gl::CreateProgram();
        for &shader in shaders {
            gl::AttachShader(program, shader);
        }
        gl::LinkProgram(program);

        if delete_shaders {
            for &shader in shaders {
                gl::DeleteShader(shader);
            }
        }

        let mut success = std::mem::uninitialized();
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            let mut loglen = std::mem::uninitialized();
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut loglen);
            let buffer_len = (loglen - 1) as usize;
            let mut buffer : Vec<u8> = Vec::with_capacity(buffer_len + 1);
            gl::GetProgramInfoLog(program, loglen, std::ptr::null_mut(),
                buffer.as_ptr() as *mut GLchar);
            buffer.set_len(loglen as usize - 1);
            let message = String::from_utf8(buffer).unwrap();
            gl::DeleteProgram(program);
            Err(Error::new(message))
        } else {
            Ok(program)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferUsage {
    StreamDraw, StreamRead, StreamCopy,
    StaticDraw, StaticRead, StaticCopy,
    DynamicDraw, DynamicRead, DynamicCopy,
}

pub fn create_buffer<T>(data : &[T], usage: BufferUsage) -> Result<GLuint> {
    unsafe {
        let mut buffer = std::mem::uninitialized();
        gl::GenBuffers(1, &mut buffer);
        let size = (data.len() * std::mem::size_of::<T>()) as GLsizeiptr;
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);

        use self::BufferUsage::*;
        let gl_usage = match usage {
            StreamDraw => gl::STREAM_DRAW,
            StreamRead => gl::STREAM_READ,
            StreamCopy => gl::STREAM_COPY,
            StaticDraw => gl::STATIC_DRAW,
            StaticRead => gl::STATIC_READ,
            StaticCopy => gl::STATIC_COPY,
            DynamicDraw => gl::DYNAMIC_DRAW,
            DynamicRead => gl::DYNAMIC_READ,
            DynamicCopy => gl::DYNAMIC_COPY,
        };

        gl::BufferData(gl::ARRAY_BUFFER, size, data.as_ptr() as _,
            gl_usage);
        Ok(buffer)
    }
}

pub fn create_single_buffer_vertex_array(buffer : GLuint, components : &[GLint]) -> Result<GLuint> {
    unsafe {
        let mut vertex_array = std::mem::uninitialized();
        gl::GenVertexArrays(1, &mut vertex_array);
        gl::BindVertexArray(vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        let total_components : GLsizei = components.iter().sum();
        let stride = total_components * std::mem::size_of::<f32>() as GLsizei;
        let mut offset : usize = 0;
        for (index, &comp) in components.iter().enumerate() {
            let index = index as GLuint;
            let offset_ptr = offset as *mut GLvoid;
            gl::VertexAttribPointer(index, comp, gl::FLOAT, gl::FALSE,
                stride, offset_ptr);
            gl::EnableVertexAttribArray(index);
            offset = offset + (comp as usize) * std::mem::size_of::<f32>();
        }

        Ok(vertex_array)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ShaderCode {
    code: [Option<String>;6],
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex, Fragment, TessControl, TessEval, Geometry, Compute,
}

impl ShaderType {
    pub fn each() -> [ShaderType;6] {
        use self::ShaderType::*;
        [Vertex, Fragment, TessControl, TessEval, Geometry, Compute,]
    }

    pub fn as_gl_enum(self) -> u32 {
        use self::ShaderType::*;
        match self {
            Vertex => gl::VERTEX_SHADER,
            Fragment => gl::FRAGMENT_SHADER,
            TessControl => gl::TESS_CONTROL_SHADER,
            TessEval => gl::TESS_EVALUATION_SHADER,
            Geometry => gl::GEOMETRY_SHADER,
            Compute => gl::COMPUTE_SHADER,
        }
    }

    pub fn short_name(self) -> &'static str {
        use self::ShaderType::*;
        match self {
            Vertex => "vertex",
            Fragment => "fragment",
            TessControl => "tess control",
            TessEval => "tess eval",
            Geometry => "geometry",
            Compute => "compute",
        }
    }
}

impl ShaderCode {
    pub fn new() -> ShaderCode {
        ShaderCode::default()
    }
}

impl std::ops::Index<ShaderType> for ShaderCode {
    type Output = Option<String>;
    fn index(&self, ty: ShaderType) -> &Option<String> {
        &self.code[ty as usize]
    }
}

impl std::ops::IndexMut<ShaderType> for ShaderCode {
    fn index_mut(&mut self, ty: ShaderType) -> &mut Option<String> {
        &mut self.code[ty as usize]
    }
}

pub fn get_attribute_location(program: u32, name: *const std::os::raw::c_char)
-> Result<i32> {
    unsafe {
        let loc = gl::GetAttribLocation(program, name);
        assert_or_error!(loc != -1, "could not bind attribute {:?}", std::ffi::CStr::from_ptr(name))?;
        Ok(loc)
    }
}

pub fn get_uniform_location(program: u32, name: *const std::os::raw::c_char)
-> Result<i32> {
    unsafe {
        let loc = gl::GetUniformLocation(program, name);
        assert_or_error!(loc != -1, "could not bind uniform {:?}", std::ffi::CStr::from_ptr(name))?;
        Ok(loc)
    }
}