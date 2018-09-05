#![allow(dead_code)]

use std;
use gl;
use simple_error::{SimpleError, SimpleResult};

#[allow(unused_imports)]
use gl::types::{GLuint, GLint, GLsizei, GLchar, GLenum, GLsizeiptr, GLvoid};

pub struct ProgramBuilder {
    vertex_shader_code : Option<String>,
    fragment_shader_code : Option<String>,
}

impl ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        ProgramBuilder {
            vertex_shader_code : None,
            fragment_shader_code : None,
        }
    }

    pub fn vertex_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.vertex_shader_code = Some(code.into());
        self
    }

    pub fn fragment_shader_code<T>(&mut self, code : T) -> &mut ProgramBuilder
        where T : Into<String>
    {
        self.fragment_shader_code = Some(code.into());
        self
    }

    pub fn code(&mut self, code : ShaderCode) -> &mut ProgramBuilder {
        self.vertex_shader_code = Some(code.vertex);
        self.fragment_shader_code = Some(code.fragment);
        self
    }

    pub fn build(&self) -> SimpleResult<GLuint> {
        let mut shaders = Vec::new();
        if let Some(ref code) = self.vertex_shader_code {
            shaders.push(compile_shader(&code, ShaderType::Vertex)?);
        }

        if let Some(ref code) = self.fragment_shader_code {
            shaders.push(compile_shader(&code, ShaderType::Fragment)?);
        }

        create_program(&shaders, true)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Vertex, Fragment,
}

pub fn compile_shader(code : &str, shader_type : ShaderType) -> SimpleResult<GLuint> {
    unsafe {
        let gl_type = match shader_type {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        };

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
            
            let shader_name = match shader_type {
                ShaderType::Vertex => "vertex",
                ShaderType::Fragment => "fragment"
            };

            let message = format!("error in {} shader : {}", shader_name, log);

            gl::DeleteShader(shader);
            Err(SimpleError::new(message))
        } else {
            Ok(shader)
        }
    }
}

pub fn create_program(shaders : &[GLuint], delete_shaders : bool)
    -> SimpleResult<GLuint>
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
            Err(SimpleError::new(message))
        } else {
            Ok(program)
        }
    }
}

pub fn create_buffer<T>(data : &[T]) -> SimpleResult<GLuint> {
    unsafe {
        let mut buffer = std::mem::uninitialized();
        gl::GenBuffers(1, &mut buffer);
        let size = (data.len() * std::mem::size_of::<T>()) as GLsizeiptr;
        gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
        gl::BufferData(gl::ARRAY_BUFFER, size, data.as_ptr() as _,
            gl::STATIC_DRAW);
        Ok(buffer)
    }
}

pub fn create_vertex_array(buffer : GLuint, components : &[GLint]) -> SimpleResult<GLuint> {
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

pub struct ShaderCode {
    vertex : String,
    fragment : String,
}

pub fn load_and_parse_shaders(path : &str)
-> Result<ShaderCode, Box<std::error::Error>> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use gl_helpers::ShaderType;

    let mut code = ShaderCode {
        vertex : String::new(),
        fragment : String::new()
    };

    let mut shader_type_opt : Option<ShaderType> = None;

    let file = File::open(path)?;
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.contains("#shader") {
            shader_type_opt = 
                if line.contains("vertex") {
                    Some(ShaderType::Vertex)
                } else if line.contains("fragment") {
                    Some(ShaderType::Fragment)
                } else {
                    None
                };
        } else {
            match shader_type_opt {
                Some(shader_type) => {
                    let shader =
                        match shader_type {
                            ShaderType::Vertex => &mut code.vertex,
                            ShaderType::Fragment => &mut code.fragment,
                        };
                    shader.push_str(&line);
                    shader.push('\n');
                },
                _ => (),
            }
        }
    }

    Ok(code)
}