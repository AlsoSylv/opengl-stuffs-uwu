mod shaders;
mod textures;

use std::f32::consts::PI;
use std::ffi::c_void;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::{mem, ptr};

use glfw::{Action, Context, Key};

use nalgebra_glm as glm;
use opengl::gl;

use shaders::Shader;
use textures::TextureBuilder;

const VERTEX_SHADER_SOURCE: &str = "./resources/shaders/vertex.vert";

const FRAGMENT_SHADER_SOURCE: &str = "./resources/shaders/fragment.frag";

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
            // Positions      | Colors RGB    | Texture coords
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

        let (vao, buffer, ind_size) = Buffer::create_shared_buffer(&verticies, &indices);

        let vb = VertexBuilder::default()
            .vao(vao, buffer, ind_size)
            .attribute(3, gl::FLOAT)
            .attribute(3, gl::FLOAT)
            .attribute(2, gl::FLOAT);

        let img = image::open(Path::new("./resources/textures/wall.jpg")).unwrap();
        let img_2 = image::open(Path::new("./resources/textures/awesomeface.png")).unwrap();

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

    // let transform = CString::new("trans").unwrap();

    while !window.should_close() {
        let (width, height) = window.get_size();
        handle_window_event(&mut window, &events);

        unsafe {
            // let ortho = glm::ortho(0.0, 800.0, 0.0, 600.0, 0.1, 140.0);
            let proj = glm::perspective(45.0 * RADIANS, (width / height) as f32, 0.1, 100.0);
            let mut model = glm::Mat4::identity();
            model = glm::rotate(&model, -55.0 * RADIANS, &glm::vec3(1.0, 0.0, 0.0));

            let mut view = glm::Mat4::identity();
            view = glm::translate(&view, &glm::vec3(0.0, 0.0, -3.0));

            program.uniform_matrix_4fv("model", 1, gl::FALSE, glm::value_ptr(&model).as_ptr());

            program.uniform_matrix_4fv("view", 1, gl::FALSE, glm::value_ptr(&view).as_ptr());

            program.uniform_matrix_4fv("projection", 1, gl::FALSE, glm::value_ptr(&proj).as_ptr());

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

#[derive(Default)]
struct VertexBuilder {
    next_attribute: u32,
    last_size: u32,
    vao: u32,
}

impl VertexBuilder {
    fn vao(mut self, mut vao: u32, buffer: u32, size: isize) -> Self {
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            gl::VertexArrayVertexBuffer(vao, 0, buffer, size, 8 * size_of(gl::FLOAT) as i32);
            gl::VertexArrayElementBuffer(vao, buffer);
            self.vao = vao;
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
    fn create(size: isize) -> u32 {
        unsafe {
            let mut buffer = 0;
            gl::CreateBuffers(1, &mut buffer);
            gl::NamedBufferStorage(buffer, size, ptr::null(), gl::DYNAMIC_STORAGE_BIT);
            buffer
        }
    }

    fn create_shared_buffer<A, B>(verticies: &[A], indicies: &[B]) -> (u32, u32, isize) {
        unsafe {
            let mut alignment = 0;
            gl::GetIntegerv(gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT, &mut alignment);

            let vao: u32 = 0;

            let ind_size = (indicies.len() * mem::size_of::<B>()) as isize;
            let vrt_size = (verticies.len() * mem::size_of::<A>()) as isize;

            let buffer = Buffer::create(ind_size + vrt_size);

            Buffer::named_buffer_sub_data(indicies, buffer, 0, ind_size);
            Buffer::named_buffer_sub_data(verticies, buffer, ind_size, vrt_size);

            (vao, buffer, ind_size)
        }
    }

    fn named_buffer_sub_data<T>(array: &[T], buffer: u32, offset: isize, size: isize) {
        unsafe {
            gl::NamedBufferSubData(buffer, offset, size, array.as_ptr() as *const c_void);
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
