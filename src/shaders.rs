use std::ffi::{c_char, CString};
use std::{fs, ptr};

use opengl::gl;

pub struct Shader {
    pub id: u32,
}

#[allow(dead_code)]
impl Shader {
    pub fn new(v_shader_file: &str, f_shader_file: &str) -> Shader {
        let v_shader = fs::read_to_string(v_shader_file);
        let f_shader = fs::read_to_string(f_shader_file);
        match v_shader {
            Ok(v_shader) => match f_shader {
                Ok(f_shader) => {
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
                Err(err) => panic!("Fragment Shader Error: {err}"),
            },
            Err(err) => panic!("Vertex Shader Error: {err}"),
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

    pub unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    fn set_bool(&self, name: &str, value: bool) {
        self.set_int(name, value as i32)
    }

    pub fn set_int(&self, name: &str, value: i32) {
        unsafe {
            let location = self.uniform_location(name);
            gl::Uniform1i(location, value)
        }
    }

    pub fn set_float(&self, name: &str, value: f32) {
        unsafe {
            let location = self.uniform_location(name);
            gl::Uniform1f(location, value)
        }
    }

    pub fn uniform_matrix_4fv(
        &self,
        name: &str,
        count: i32,
        transpose: gl::types::GLboolean,
        value: *const f32,
    ) {
        unsafe {
            let location = self.uniform_location(name);
            gl::UniformMatrix4fv(location, count, transpose, value);
        }
    }

    fn uniform_location(&self, name: &str) -> i32 {
        unsafe {
            let name = CString::new(name).unwrap();
            gl::GetUniformLocation(self.id, name.as_ptr())
        }
    }
}
