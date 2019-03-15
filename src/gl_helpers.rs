#![allow(dead_code)]

use std;
use gl;

type BoxResult<T> = std::result::Result<T, Box<std::error::Error>>;
pub type Error = Box<std::error::Error>;
pub type Result<T> = std::result::Result<T, Error>;

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

    pub fn geometry_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.code[ShaderType::Geometry] = Some(code.into());
        self
    }

    pub fn tess_control_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.code[ShaderType::TessControl] = Some(code.into());
        self
    }

    pub fn tess_eval_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.code[ShaderType::TessEval] = Some(code.into());
        self
    }

    pub fn compute_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.code[ShaderType::Compute] = Some(code.into());
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

    pub fn build(&self) -> Result<u32> {
        let mut shaders = Vec::new();
        for shader_type in ShaderType::each().iter() {
            if let Some(ref code) = self.code[*shader_type] {
                shaders.push(compile_shader(&code, *shader_type)?);
            }
        }

        create_program(&shaders, true)
    }
}

pub fn compile_shader(code : &str, shader_type : ShaderType) -> Result<u32> {
    unsafe {
        let gl_type = shader_type.as_gl_enum();
        let shader = gl::CreateShader(gl_type);
        let len = code.len() as i32;
        let code_ptr = code.as_ptr() as *const std::os::raw::c_char;
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
                buffer.as_ptr() as *mut std::os::raw::c_char);
            buffer.set_len(loglen as usize - 1);
            let log = String::from_utf8(buffer).unwrap();
            
            let shader_name = shader_type.short_name();
            let message = format!("error in {} shader : {}", shader_name, log);

            gl::DeleteShader(shader);
            Err(message.into())
        } else {
            Ok(shader)
        }
    }
}

pub fn create_program(shaders : &[u32], delete_shaders : bool)
    -> Result<u32>
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
                buffer.as_ptr() as *mut std::os::raw::c_char);
            buffer.set_len(loglen as usize - 1);
            let message = String::from_utf8(buffer).unwrap();
            gl::DeleteProgram(program);
            Err(message.into())
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

impl BufferUsage {
    pub fn as_gl_enum(self) -> u32 {
        match self {
            BufferUsage::StreamDraw => gl::STREAM_DRAW,
            BufferUsage::StreamRead => gl::STREAM_READ,
            BufferUsage::StreamCopy => gl::STREAM_COPY,
            BufferUsage::StaticDraw => gl::STATIC_DRAW,
            BufferUsage::StaticRead => gl::STATIC_READ,
            BufferUsage::StaticCopy => gl::STATIC_COPY,
            BufferUsage::DynamicDraw => gl::DYNAMIC_DRAW,
            BufferUsage::DynamicRead => gl::DYNAMIC_READ,
            BufferUsage::DynamicCopy => gl::DYNAMIC_COPY,
        }
    }
}

pub fn create_buffer<T>(data : &[T], usage: BufferUsage) -> Result<u32> {
    unsafe {
        let mut buffer = std::mem::uninitialized();
        gl::CreateBuffers(1, &mut buffer);
        named_buffer_data(buffer, data, usage);
        Ok(buffer)
    }
}

pub fn named_buffer_data<T>(buffer: u32, data: &[T], usage: BufferUsage) {
    let size = (data.len() * std::mem::size_of::<T>()) as isize;
    unsafe {
        gl::NamedBufferData(buffer, size, data.as_ptr() as _, usage.as_gl_enum());
    }
}

pub fn create_single_buffer_vertex_array(buffer : u32, components : &[i32]) -> Result<u32> {
    unsafe {
        let mut vertex_array = std::mem::uninitialized();
        gl::GenVertexArrays(1, &mut vertex_array);
        gl::BindVertexArray(vertex_array);
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        let total_components : i32 = components.iter().sum();
        let stride = total_components * std::mem::size_of::<f32>() as i32;
        let mut offset : usize = 0;
        for (index, &comp) in components.iter().enumerate() {
            let index = index as u32;
            let offset_ptr = offset as *mut std::os::raw::c_void;
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
        if loc == -1 {
            Err(format!("could not bind attribute {:?}", std::ffi::CStr::from_ptr(name)).into())
        } else {
            Ok(loc)
        }
    }
}

pub fn get_uniform_location(program: u32, name: *const std::os::raw::c_char)
-> Result<i32> {
    unsafe {
        let loc = gl::GetUniformLocation(program, name);
        if loc == -1 {
            Err(format!("could not bind uniform {:?}", std::ffi::CStr::from_ptr(name)).into())
        } else {
            Ok(loc)
        }
    }
}