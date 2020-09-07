use crate::gl_utils::shaders::errors::ShaderProgramError;
use crate::gl_utils::bindable::Bindable;
use super::{shader_type::ShaderType, errors::GlUniformError};

use gl::types::{
    GLboolean, 
    GLsizei, 
    GLuint, 
    GLint
};

use std::{
    collections::HashMap, 
    ffi::CString, 
    ptr, 
    path::Path
};

pub struct Program {
    pub program_id: u32,
    uniforms: HashMap<String, GLint> 
}

impl Bindable for Program {
    fn bind(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }
}

impl Program {
    pub fn locate_uniform(&mut self, name: &str) -> Result<(), ShaderProgramError> {
        if let Some(_) = self.uniforms.get(name) {
            return Ok(());
        }

        let target_location = unsafe { 
            let c_name = match CString::new(name) {
                Ok(c_name) => c_name,
                Err(e) => return Err(ShaderProgramError::CStr(e))
            };

            let location = gl::GetUniformLocation(self.program_id, c_name.as_ptr());
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                return Err(ShaderProgramError::GlUniform(GlUniformError::new(error)));
            }

            location
        };

        if target_location < 0 {
            return Err(ShaderProgramError::UniformNotFound);
        }

        self.uniforms.insert(name.to_string(), target_location);

        return Ok(());
    }

    // TODO: solve duplicate code in set_uniform...

    pub fn set_uniform1<T>(&self, name: &str, value: T, assign_fn: unsafe fn(GLint, T) -> ()) -> Result<(), ShaderProgramError> {
        let uniform_location: i32 = match self.uniforms.get(name) {
            Some(u) => *u,
            None => return Err(ShaderProgramError::UniformNotFound)
        };

        unsafe {
            let mut active_program: GLint = 0;
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut active_program);

            gl::UseProgram(self.program_id);
            assign_fn(uniform_location, value);
            let error = gl::GetError();
            gl::UseProgram(active_program as GLuint);

            if error != gl::NO_ERROR {
                return Err(ShaderProgramError::GlUniform(GlUniformError::new(error)));
            }
        }

        Ok(())
    }

    pub fn set_uniform_matrix<T>(&self, name: &str, value: T, assign_fn: unsafe fn(GLint, GLsizei, GLboolean, T) -> ()) -> Result<(), ShaderProgramError> {
        let uniform_location: i32 = match self.uniforms.get(name) {
            Some(u) => *u,
            None => return Err(ShaderProgramError::UniformNotFound)
        };

        unsafe {
            let mut active_program: GLint = 0;
            gl::GetIntegerv(gl::CURRENT_PROGRAM, &mut active_program);

            gl::UseProgram(self.program_id);
            assign_fn(uniform_location, 1, gl::FALSE, value);
            let error = gl::GetError();
            gl::UseProgram(active_program as GLuint);

            if error != gl::NO_ERROR {
                return Err(ShaderProgramError::GlUniform(GlUniformError::new(error)));
            }
        }

        Ok(())
    }
   
}


pub struct ProgramBuilder {
    program_id: u32,
    shaders: Vec::<u32>,
}

impl ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        let program_id = unsafe {
            gl::CreateProgram()
        };

        ProgramBuilder {
            program_id,
            shaders: vec![],
        }
    }

    pub fn attach_file(self, shader_path: &str) -> ProgramBuilder {
        let path = Path::new(shader_path);
        if let Some(extension) = path.extension() {
            let shader_type = ShaderType::from_ext(extension)
                .expect("Failed to parse file extension.");
            let shader_src = std::fs::read_to_string(path)
                .expect(&format!("Failed to read shader source. {}", shader_path));

            self.compile_shader(&shader_src, shader_type)
        } else {
            panic!("Failed to read extension of file with path: {}", shader_path);
        }
    }

    pub fn compile_shader(mut self, shader_src: &str, shader_type: ShaderType) -> ProgramBuilder {
        let shader = unsafe {
            let shader = gl::CreateShader(shader_type.into());
            let c_str_shader = CString::new(shader_src.as_bytes()).unwrap();

            gl::ShaderSource(shader, 1, &c_str_shader.as_ptr(), ptr::null());
            gl::CompileShader(shader);
            if !self.check_shader_errors(shader) {
                panic!("Shader failed to compile.");
            }

            shader
        };

        self.shaders.push(shader);

        self
    }

    unsafe fn check_shader_errors(&self, shader_id: u32) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);

        info_log.set_len(512 - 1);
        gl::GetShaderiv(shader_id, gl::COMPILE_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetShaderInfoLog(
                shader_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!("ERROR::Shader Compilation Failed!\n{}", String::from_utf8_lossy(&info_log));
            return false;
        }

        true
    }

    unsafe fn check_linker_errors(&self) -> bool {
        let mut success = i32::from(gl::FALSE);
        let mut info_log = Vec::with_capacity(512);

        info_log.set_len(512 - 1);
        gl::GetProgramiv(self.program_id, gl::LINK_STATUS, &mut success);
        if success != i32::from(gl::TRUE) {
            gl::GetProgramInfoLog(
                self.program_id,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            println!("ERROR::SHADER::PROGRAM::COMPILATION_FAILED\n{}", String::from_utf8_lossy(&info_log));
            return false;
        }
        true
    }

    #[must_use = "Program can only be built using the link() function"]
    pub fn link(self) -> Program {
        unsafe {
            for &shader in &self.shaders {
                gl::AttachShader(self.program_id, shader);
            }
            gl::LinkProgram(self.program_id);
    
            // todo:: use this to make safer abstraction
            self.check_linker_errors();
    
            for &shader in &self.shaders {
                gl::DeleteShader(shader);
            }
        }

        Program {
            program_id: self.program_id,
            uniforms: HashMap::new()
        }
    }
}