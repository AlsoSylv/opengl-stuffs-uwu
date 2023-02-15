mod shaders;

use std::f32::consts::PI;
use std::ffi::{c_void, CString};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::{mem, ptr};

use glfw::{Action, Context, Key};

use glm::Vec3;
use image::DynamicImage;
use nalgebra_glm as glm;
use opengl::gl;

use crate::shaders::Shader;

const VERTEX_SHADER_SOURCE: &str = "resources/shaders/vertex.vert";

const FRAGMENT_SHADER_SOURCE: &str = "resources/shaders/fragment.frag";

#[allow(dead_code)]
const RADIANS: f32 = PI / 180.0;

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

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const c_void);

    framebuffer_size_callback(1280, 720);

    // Modern OGL: https://github.com/fendevel/Guide-to-Modern-OpenGL-Functions#glnamedbufferdata
    // Learn OGL: https://learnopengl.com/
    // Learn OGL RS: https://github.com/bwasty/learn-opengl-rs
    // ECS: https://www.youtube.com/watch?v=aKLntZcp27M
    let shaders = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    #[allow(unused_unsafe)]
    let (program, vao, texture, texture_2) = unsafe {
        #[rustfmt::skip]
        let verticies: [f32; 32] = [
            // Positions     | Colors RGB     | Texture coords
             0.5,  0.5, 0.0,  1.0, 0.0, 0.0,  1.0, 1.0,  // 0
             0.5, -0.5, 0.0,  0.0, 1.0, 0.0,  1.0, 0.0,  // 1
            -0.5, -0.5, 0.0,  0.0, 0.0, 1.0,  0.0, 0.0,  // 2
            -0.5,  0.5, 0.0,  0.5, 0.0, 0.5,  0.0, 1.0,  // 3
        ];

        #[rustfmt::skip]
        let indices: [i32; 6] = [
            0, 1, 3,  // Indexes of verts
            1, 2, 3,  // Indexes of verts
        ];

        let vbo = Buffer::create(&verticies);

        let ibo = Buffer::create(&indices);

        let vb = VertexBuilder::default()
            .vao(vbo, ibo)
            .attribute(3, gl::FLOAT)
            .attribute(3, gl::FLOAT)
            .attribute(2, gl::FLOAT);

        let img = image::open(Path::new("resources\\textures\\wall.jpg")).unwrap();
        let img_2 = image::open(Path::new("resources\\textures\\awesomeface.png")).unwrap();

        let texture = TextureBuilder::new(img, gl::RGB, gl::RGB8)
            .texture_paramater_i(gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT)
            .texture_paramater_i(gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT)
            .texture_paramater_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST)
            .texture_paramater_i(gl::TEXTURE_MAG_FILTER, gl::LINEAR)
            .texture_storage(1)
            .sub_texture(0, 0);

        let texture_2 = TextureBuilder::new(img_2, gl::RGBA, gl::RGBA8)
            .texture_storage(1)
            .sub_texture(0, 0);

        (shaders, vb.vao, texture.texture, texture_2.texture)
    };

    let transform = CString::new("trans").unwrap();

    while !window.should_close() {
        handle_window_event(&mut window, &events);

        unsafe {
            let mut trans = glm::Mat4::identity();
            trans = glm::translate(&trans, &glm::vec3(0.5, -0.5, 0.0));
            trans = glm::rotate(&trans, glfw.get_time() as f32, &glm::vec3(0.0, 0.0, 1.0));
            trans = glm::scale(&trans, &Vec3::new(0.5, 0.5, 0.5));

            let trans_loc = gl::GetUniformLocation(program.id, transform.as_ptr());
            gl::UniformMatrix4fv(trans_loc, 1, gl::FALSE, trans.as_ptr());
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::BindTextures(0, 2, [texture, texture_2].as_ptr());

            program.use_program();
            gl::BindVertexArray(vao);
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

#[allow(dead_code)]
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

    fn flip(mut self) -> Self {
        self.image = self.image.flipv();
        self
    }
}

#[derive(Default)]
struct VertexBuilder {
    next_attribute: u32,
    last_size: u32,
    vao: u32,
}

impl VertexBuilder {
    fn vao(mut self, vbo: u32, ibo: u32) -> Self {
        unsafe {
            gl::CreateVertexArrays(1, &mut self.vao);
            gl::VertexArrayVertexBuffer(self.vao, 0, vbo, 0, 8 * size_of(gl::FLOAT) as i32);
            gl::VertexArrayElementBuffer(self.vao, ibo);
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

struct Buffer;

impl Buffer {
    fn create<T>(array: &[T]) -> u32 {
        unsafe {
            let mut buffer = 0;
            gl::CreateBuffers(1, &mut buffer);
            gl::NamedBufferStorage(
                buffer,
                (array.len() * mem::size_of::<T>()) as isize,
                array.as_ptr() as *const c_void,
                gl::DYNAMIC_STORAGE_BIT,
            );
            buffer
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

fn size_of(glenum: gl::types::GLenum) -> u32 {
    match glenum {
        gl::FLOAT => 4,
        _ => unreachable!(),
    }
}
