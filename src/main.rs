use std::ffi::{c_char, c_void, CString};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::{fs, mem, ptr};

use glfw::{Action, Context, Key};

use image::DynamicImage;
use opengl::gl;

const VERTEX_SHADER_SOURCE: &str = "resources/shaders/vertex.vert";

const FRAGMENT_SHADER_SOURCE: &str = "resources/shaders/fragment.frag";

fn main() {
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();

    glfw.window_hint(glfw::WindowHint::ContextVersionMajor(4));
    glfw.window_hint(glfw::WindowHint::ContextVersionMinor(5));

    glfw.window_hint(glfw::WindowHint::OpenGlProfile(
        glfw::OpenGlProfileHint::Core,
    ));

    let (mut window, events) = glfw
        .create_window(1280, 720, "Hello World", glfw::WindowMode::Windowed)
        .expect("Failed to create GLFW Window");

    window.set_key_polling(true);
    window.make_current();
    window.set_framebuffer_size_polling(true);

    // glfw.make_context_current(Some(&window));

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    framebuffer_size_callback(1280, 720);

    // Modern OGL: https://github.com/fendevel/Guide-to-Modern-OpenGL-Functions#glnamedbufferdata
    // Learn OGL: https://learnopengl.com/
    // Learn OGL RS: https://github.com/bwasty/learn-opengl-rs
    // ECS: https://www.youtube.com/watch?v=aKLntZcp27M
    let shaders = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    #[allow(unused_unsafe)]
    let (program, vao, texture) = unsafe {
        #[rustfmt::skip]
        let verticies: [f32; 32] = [
            // Positions     | Colors RGB     | Texture coords
             0.5,  0.5, 0.0,  1.0, 0.0, 0.0,  1.0, 1.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
             0.5, -0.5, 0.0,  0.0, 1.0, 0.0,  1.0, 0.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
            -0.5, -0.5, 0.0,  0.0, 0.0, 1.0,  0.0, 0.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
            -0.5,  0.5, 0.0,  0.0, 1.0, 0.0,  0.0, 1.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
        ];

        let indices: [i32; 6] = [
            0, 1, 3, // Stop
            1, 2, 3, // Stop
        ];

        let vb = VertexBuilder::new()
            .ebo(&indices)
            .vbo(&verticies)
            .attribute(3, gl::FLOAT)
            .attribute(3, gl::FLOAT)
            .attribute(2, gl::FLOAT);

        let img = image::open(Path::new("resources\\textures\\wall.jpg")).unwrap();

        let texture = TextureBuilder::new(img, gl::RGB, gl::RGB8)
            .texture_paramater_i(gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT)
            .texture_paramater_i(gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT)
            .texture_paramater_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST)
            .texture_paramater_i(gl::TEXTURE_MAG_FILTER, gl::LINEAR)
            .texture_storage(1)
            .sub_texture(0, 0);

        (shaders, vb, texture.texture)
    };

    while !window.should_close() {
        handle_window_event(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindTextureUnit(0, texture);

            program.use_program();
            gl::BindVertexArray(vao.vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

struct TextureBuilder {
    texture: u32,
    image: DynamicImage,
    internal_format: gl::types::GLenum,
    internalformat: gl::types::GLenum,
}

impl TextureBuilder {
    fn new(
        image: DynamicImage,
        internal_format: gl::types::GLenum,
        internalformat: gl::types::GLenum,
    ) -> TextureBuilder {
        unsafe {
            let mut texture = 0;
            gl::CreateTextures(gl::TEXTURE_2D, 1, &mut texture);
            TextureBuilder {
                texture,
                image,
                internal_format,
                internalformat,
            }
        }
    }

    fn texture_storage(self, levels: i32) -> Self {
        unsafe {
            gl::TextureStorage2D(
                self.texture,
                levels,
                self.internalformat,
                self.image.width() as i32,
                self.image.height() as i32,
            );
            self
        }
    }

    fn sub_texture(self, x_offset: i32, y_offset: i32) -> Self {
        unsafe {
            gl::TextureSubImage2D(
                self.texture,
                0,
                x_offset,
                y_offset,
                self.image.width() as i32,
                self.image.height() as i32,
                self.internal_format,
                gl::UNSIGNED_BYTE,
                self.image.as_bytes().as_ptr() as *const c_void,
            );
            self
        }
    }

    fn texture_paramater_i(self, p_name: gl::types::GLenum, param: gl::types::GLenum) -> Self {
        unsafe {
            gl::TextureParameteri(self.texture, p_name, param as i32);
            self
        }
    }
}

struct VertexBuilder {
    next_attribute: u32,
    last_size: u32,
    vao: u32,
    ebo: u32,
    vbo: u32,
}

#[allow(dead_code)]
impl VertexBuilder {
    fn new() -> VertexBuilder {
        unsafe {
            let (vbo, mut vao, ebo) = (0, 0, 0);
            // let (mut vbo, mut vao, mut ebo) = (0, 0, 0);
            gl::CreateVertexArrays(1, &mut vao);
            VertexBuilder {
                vao,
                ebo,
                vbo,
                next_attribute: 0,
                last_size: 0,
            }
        }
    }

    fn ebo(mut self, indices: &[i32]) -> Self {
        unsafe {
            let mut ebo = 0;
            gl::CreateBuffers(1, &mut ebo);
            gl::NamedBufferStorage(
                ebo,
                (indices.len() * mem::size_of::<f32>()) as isize,
                indices.as_ptr() as *const c_void,
                gl::DYNAMIC_STORAGE_BIT,
            );
            gl::VertexArrayElementBuffer(self.vao, ebo);
            self.ebo = ebo;
            self
        }
    }

    fn vbo(mut self, verticies: &[f32]) -> Self {
        unsafe {
            let mut vbo = 0;
            gl::CreateBuffers(1, &mut vbo);
            gl::NamedBufferStorage(
                vbo,
                (verticies.len() * mem::size_of::<f32>()) as isize,
                verticies.as_ptr() as *const c_void,
                gl::DYNAMIC_STORAGE_BIT,
            );
            gl::VertexArrayVertexBuffer(self.vao, 0, vbo, 0, 8 * size_of(gl::FLOAT) as i32);
            self.vbo = vbo;
            self
        }
    }

    fn attribute(mut self, size: u32, _type: gl::types::GLenum) -> Self {
        unsafe {
            gl::EnableVertexArrayAttrib(self.vao, self.next_attribute);
            gl::VertexArrayAttribFormat(
                self.vao,
                self.next_attribute,
                size as i32,
                _type,
                gl::FALSE,
                self.last_size * size_of(gl::FLOAT),
            );
            gl::VertexArrayAttribBinding(self.vao, self.next_attribute, 0);
            self.last_size += size;
            self.next_attribute += 1;
            self
        }
    }
}

fn handle_window_event(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, events) in glfw::flush_messages(events) {
        match events {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            glfw::WindowEvent::FramebufferSize(width, height) => {
                framebuffer_size_callback(width, height)
            }
            _ => {}
        }
    }
}

fn framebuffer_size_callback(width: i32, height: i32) {
    unsafe { gl::Viewport(0, 0, width, height) }
}

struct Shader {
    id: u32,
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

                        let mut success = gl::FALSE as i32;
                        let mut info_log = Vec::with_capacity(512);
                        info_log.set_len(512 - 1);

                        gl::GetProgramiv(id, gl::LINK_STATUS, &mut success);
                        if success != gl::TRUE as i32 {
                            gl::GetProgramInfoLog(
                                id,
                                512,
                                ptr::null_mut(),
                                info_log.as_mut_ptr() as *mut c_char,
                            )
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
            let mut success = gl::FALSE as i32;
            let mut info_log = Vec::with_capacity(512);
            info_log.set_len(512 - 1);

            gl::ShaderSource(shader, count, &shader_code.as_ptr(), ptr::null());
            gl::CompileShader(shader);

            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success != gl::TRUE as i32 {
                gl::GetShaderInfoLog(
                    shader,
                    512,
                    ptr::null_mut(),
                    info_log.as_mut_ptr() as *mut c_char,
                );
                println!("Vertex Compliation Failed");
            }
        }
    }

    unsafe fn use_program(&self) {
        gl::UseProgram(self.id);
    }

    fn set_bool(&self, name: &str, value: bool) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            gl::Uniform1i(
                gl::GetUniformLocation(self.id, c_str.as_ptr()),
                value as i32,
            )
        }
    }

    fn set_int(&self, name: &str, value: i32) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            gl::Uniform1i(gl::GetUniformLocation(self.id, c_str.as_ptr()), value)
        }
    }

    fn set_float(&self, name: &str, value: f32) {
        unsafe {
            let c_str = CString::new(name).unwrap();
            gl::Uniform1f(gl::GetUniformLocation(self.id, c_str.as_ptr()), value)
        }
    }
}

fn size_of(glenum: gl::types::GLenum) -> u32 {
    match glenum {
        gl::FLOAT => 4,
        _ => unreachable!(),
    }
}
