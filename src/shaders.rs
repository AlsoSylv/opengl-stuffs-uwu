use std::ffi::{c_char, CString};
use std::{fs, ptr};

use opengl::gl;

use self::sealed::{UniformValue, UniformValueTranspose};

pub struct Shader {
    pub id: u32,
}

#[allow(dead_code)]
impl Shader {
    pub fn new(v_shader_file: &str, f_shader_file: &str) -> Shader {
        let v_shader = fs::read_to_string(v_shader_file);
        let f_shader = fs::read_to_string(f_shader_file);

        let Ok(v_shader) = v_shader else {
            panic!("Vertex Shader Error: {v_shader:?}")
        };

        let Ok(f_shader) = f_shader else {
            panic!("Fragment Shader Error: {f_shader:?}")
        };

        let v_shader_code = CString::new(v_shader).unwrap();
        let f_shader_code = CString::new(f_shader).unwrap();
        unsafe {
            let vertex_shader: u32 = gl::CreateShader(gl::VERTEX_SHADER);
            Shader::compile_shader(vertex_shader, 1, v_shader_code);

            let fragment_shader: u32 = gl::CreateShader(gl::FRAGMENT_SHADER);
            Shader::compile_shader(fragment_shader, 1, f_shader_code);

            let id = gl::CreateProgram();
            gl::AttachShader(id, vertex_shader);
            gl::AttachShader(id, fragment_shader);
            gl::LinkProgram(id);

            if !Shader::get_success(id) {
                let log = Shader::get_info_log(id);
                println!("{log:?}");
            }

            gl::DeleteShader(fragment_shader);
            gl::DeleteShader(vertex_shader);

            Shader { id }
        }
    }

    fn compile_shader(shader: u32, count: i32, shader_code: CString) {
        unsafe {
            gl::ShaderSource(shader, count, &shader_code.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            if !Shader::get_success(shader) {
                let log = Shader::get_info_log(shader);
                println!("{log:?}");
            }
        }
    }

    fn get_success(shader: u32) -> bool {
        unsafe {
            let mut success = gl::FALSE as i32;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            success != gl::FALSE as i32
        }
    }

    unsafe fn get_info_log(shader: u32) -> CString {
        let mut buf_cap = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut buf_cap);
        let mut buf = Vec::with_capacity(buf_cap as usize);
        let mut buf_len = 0;
        gl::GetShaderInfoLog(
            shader,
            buf_cap,
            &mut buf_len,
            buf.as_mut_ptr() as *mut c_char,
        );

        buf.set_len(buf_len as usize);

        CString::from_vec_unchecked(buf)
    }

    pub fn use_program(&self) {
        unsafe {
            gl::UseProgram(self.id);
        }
    }

    pub fn get_uniform_block_index(&self, ubo_name: &CString) -> u32 {
        unsafe { gl::GetUniformBlockIndex(self.id, ubo_name.as_ptr()) }
    }

    pub fn uniform_block_binding(&self, ubo_index: u32) {
        unsafe {
            gl::UniformBlockBinding(self.id, ubo_index, 0);
        }
    }

    pub fn set_uniform(&self, name: &str, value: impl UniformValue) {
        value.set(self, name);
    }

    pub fn set_uniform_transpose(
        &self,
        name: &str,
        value: impl UniformValueTranspose,
        transpose: bool,
    ) {
        value.set(self, name, transpose);
    }

    fn uniform_location(&self, name: &str) -> i32 {
        unsafe {
            let name = CString::new(name).unwrap();
            gl::GetUniformLocation(self.id, name.as_ptr())
        }
    }
}

mod sealed {
    use nalgebra_glm::{TMat, Vec3};
    use opengl::gl;

    use super::Shader;

    pub trait UniformValue: Sized {
        fn set(self, shader: &Shader, name: &str);
    }

    impl UniformValue for f32 {
        fn set(self, shader: &Shader, name: &str) {
            unsafe {
                let location = shader.uniform_location(name);
                gl::Uniform1f(location, self)
            }
        }
    }

    impl UniformValue for i32 {
        fn set(self, shader: &Shader, name: &str) {
            unsafe {
                let location = shader.uniform_location(name);
                gl::Uniform1i(location, self)
            }
        }
    }

    impl UniformValue for bool {
        fn set(self, shader: &Shader, name: &str) {
            (self as i32).set(shader, name);
        }
    }

    impl UniformValue for &Vec3 {
        fn set(self, shader: &Shader, name: &str) {
            unsafe {
                let location = shader.uniform_location(name);
                gl::Uniform3fv(location, self.len() as i32, self.as_ptr())
            }
        }
    }

    pub trait UniformValueTranspose: Sized {
        fn set(self, shader: &Shader, name: &str, transpose: bool);
    }

    impl<const R: usize, const C: usize> UniformValueTranspose for TMat<f32, R, C> {
        fn set(self, shader: &Shader, name: &str, transpose: bool) {
            unsafe {
                let location = shader.uniform_location(name);
                gl::UniformMatrix4fv(location, self.len() as i32, transpose as u8, self.as_ptr());
            }
        }
    }
}
