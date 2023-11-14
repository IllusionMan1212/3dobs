use anyhow::Result;
use glad_gl::gl;
use log::{debug, error};

pub struct ShaderSource {
    pub name: String,
    pub source: String,
}

pub struct Shader {
    pub program_id: gl::GLuint,
}

impl Shader {
    pub fn new(
        vertex_obj: &mut ShaderSource,
        frag_obj: &mut ShaderSource,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        vertex_obj.source.push('\0');
        let vertex_shader_source =
            std::ffi::CStr::from_bytes_with_nul(vertex_obj.source.as_bytes())?;

        frag_obj.source.push('\0');
        let frag_shader_source = std::ffi::CStr::from_bytes_with_nul(frag_obj.source.as_bytes())?;

        unsafe {
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            gl::CreateShader(vertex_shader);
            gl::ShaderSource(
                vertex_shader,
                1,
                &vertex_shader_source.as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(vertex_shader);
            let mut success1 = 0;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success1);
            #[cfg(debug_assertions)]
            debug!(
                "vertex shader {:?} compiled with status: {}",
                vertex_obj.name, success1
            );
            if success1 == 0 {
                let info_buf = [0u8; 512];
                gl::GetShaderInfoLog(
                    vertex_shader as u32,
                    512,
                    std::ptr::null_mut(),
                    info_buf.as_ptr() as *mut i8,
                );
                #[cfg(debug_assertions)]
                error!(
                    "vertex shader info: {}",
                    std::str::from_utf8(&info_buf).unwrap()
                );
            }

            let frag_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::CreateShader(frag_shader);
            gl::ShaderSource(
                frag_shader,
                1,
                &frag_shader_source.as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(frag_shader);

            let mut success2 = 0;
            gl::GetShaderiv(frag_shader, gl::COMPILE_STATUS, &mut success2);
            #[cfg(debug_assertions)]
            debug!(
                "frag shader {:?} compiled with status: {}",
                frag_obj.name, success2
            );
            if success2 == 0 {
                let info_buf2 = [0u8; 512];
                gl::GetShaderInfoLog(
                    frag_shader as u32,
                    512,
                    std::ptr::null_mut(),
                    info_buf2.as_ptr() as *mut i8,
                );
                #[cfg(debug_assertions)]
                error!(
                    "frag shader info: {}",
                    std::str::from_utf8(&info_buf2).unwrap()
                );
            }

            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, frag_shader);
            gl::LinkProgram(shader_program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(frag_shader);

            Ok(Self {
                program_id: shader_program,
            })
        }
    }

    pub fn use_shader(&self) {
        unsafe {
            gl::UseProgram(self.program_id);
        }
    }

    pub fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            let c_str = std::ffi::CString::new(name).unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                value as i32,
            );
        }
    }

    pub fn set_int(&self, name: &str, value: i32) {
        let c_str = std::ffi::CString::new(name).unwrap();
        unsafe {
            gl::Uniform1i(
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                value,
            );
        }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        let c_str = std::ffi::CString::new(name).unwrap();
        unsafe {
            gl::Uniform1f(
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                value,
            );
        }
    }

    pub fn get_float(&self, name: &str) -> f32 {
        let c_str = std::ffi::CString::new(name).unwrap();
        let mut value = 0.0;

        unsafe {
            gl::GetUniformfv(
                self.program_id,
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                &mut value,
            );
            value
        }
    }

    pub fn set_mat3fv(&self, name: &str, value: &glm::Mat3) {
        let c_str = std::ffi::CString::new(name).unwrap();
        unsafe {
            gl::UniformMatrix3fv(
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                1,
                gl::FALSE,
                value.as_array().as_ptr() as *const f32,
            );
        }
    }

    pub fn set_mat4fv(&self, name: &str, value: &glm::Mat4) {
        let c_str = std::ffi::CString::new(name).unwrap();
        unsafe {
            gl::UniformMatrix4fv(
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                1,
                gl::FALSE,
                value.as_array().as_ptr() as *const f32,
            );
        }
    }

    pub fn set_3fv(&self, name: &str, value: glm::Vec3) {
        let c_str = std::ffi::CString::new(name).unwrap();
        unsafe {
            gl::Uniform3fv(
                gl::GetUniformLocation(self.program_id, c_str.as_ptr()),
                1,
                value.as_array() as *const f32,
            );
        }
    }
}
