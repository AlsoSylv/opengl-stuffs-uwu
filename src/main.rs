mod camera;
mod shaders;
mod textures;

use std::f32::consts::PI;
use std::ffi::{c_void, CString};
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::{mem, ptr};

use glfw::{Action, Context, Key};

use glm::Mat4;
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

    let mut proj = Mat4::identity();

    framebuffer_size_callback(1280, 720, &mut proj);

    // Modern OGL: https://github.com/fendevel/Guide-to-Modern-OpenGL-Functions#glnamedbufferdata
    // Learn OGL: https://learnopengl.com/
    // Learn OGL RS: https://github.com/bwasty/learn-opengl-rs
    // ECS: https://www.youtube.com/watch?v=aKLntZcp27M
    let shaders = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    #[allow(unused_unsafe)]
    let (program, vao, texture, texture_2, mut ubo) = unsafe {
        #[rustfmt::skip]
        let verticies: [f32; 80] = [
            // Positions      | Texture coords
             0.5,  0.5, -0.5,  1.0, 1.0,  // 0
             0.5, -0.5, -0.5,  1.0, 0.0,  // 1
            -0.5, -0.5, -0.5,  0.0, 0.0,  // 2
            -0.5,  0.5, -0.5,  0.0, 1.0,  // 3

            -0.5,  0.5,  0.5,  1.0, 0.0,  // 4
            -0.5,  0.5, -0.5,  1.0, 1.0,  // 5
            -0.5, -0.5, -0.5,  0.0, 1.0,  // 6
            -0.5, -0.5,  0.5,  0.0, 0.0,  // 7

             0.5, -0.5,  0.5,  1.0, 0.0,  // 8
             0.5,  0.5,  0.5,  1.0, 1.0,  // 9
            -0.5,  0.5,  0.5,  0.0, 1.0,  // 10

             0.5,  0.5,  0.5,  1.0, 0.0,  // 11
             0.5, -0.5, -0.5,  0.0, 1.0,  // 12
             0.5, -0.5,  0.5,  0.0, 0.0,  // 13

             0.5, -0.5, -0.5,  1.0, 1.0,  // 14
            -0.5,  0.5,  0.5,  0.0, 0.0,  // 15
        ];

        #[rustfmt::skip]
        let indices: [i32; 36] = [
            0, 1, 3,  // Indexes of verts
            1, 2, 3,  // Indexes of verts

            4, 5, 6,
            6, 7, 4,

            7, 8, 9,
            9, 10, 7,

            11, 0, 12,
            12, 13, 11,

            6, 14, 8,
            8, 7, 6,

            3, 0, 11,
            11, 15, 3
        ];

        let (vao, buffer, ind_size) = Buffer::create_shared_buffer(&verticies, &indices);

        let vao = VertexBuilder::default()
            .new(vao, buffer, ind_size)
            .attribute(3, gl::FLOAT)
            .attribute(2, gl::FLOAT)
            .build();

        let img = image::open(Path::new("./resources/textures/wall.jpg")).unwrap();
        let img_2 = image::open(Path::new("./resources/textures/awesomeface.png")).unwrap();

        let texture = TextureBuilder::new(img, gl::RGB, gl::RGB8)
            .texture_paramater_i(gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT)
            .texture_paramater_i(gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT)
            .texture_paramater_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST)
            .texture_paramater_i(gl::TEXTURE_MAG_FILTER, gl::LINEAR)
            .texture_storage(1)
            .sub_texture(0, 0)
            .build();

        let texture_2 = TextureBuilder::new(img_2, gl::RGBA, gl::RGBA8)
            .texture_storage(1)
            .sub_texture(0, 0)
            .build();

        let ubo = UBO::new(shaders.id, "MatrixBlock", 3 * mem::size_of::<glm::Mat4>());

        gl::Enable(gl::DEPTH_TEST);

        (shaders, vao, texture, texture_2, ubo)
    };

    let cube_positions = [
        glm::vec3(0.0, 0.0, 0.0),
        glm::vec3(2.0, 5.0, -15.0),
        glm::vec3(-1.5, -2.2, -2.5),
        glm::vec3(-3.8, -2.0, -12.3),
        glm::vec3(2.4, -0.4, -3.5),
        glm::vec3(-1.7, 3.0, -7.5),
        glm::vec3(1.3, -2.0, -2.5),
        glm::vec3(1.5, 2.0, -2.5),
        glm::vec3(1.5, 0.2, -1.5),
        glm::vec3(-1.3, 1.0, -1.5),
    ];

    while !window.should_close() {
        handle_window_event(&mut window, &events, &mut proj);

        let camera_pos = glm::vec3(0.0, 0.0, 3.0);
        let camera_target = glm::vec3(0.0, 0.0, 0.0);
        let camera_direction = glm::normalize(&(camera_pos - camera_target));

        let up = glm::vec3(0.0, 1.0, 0.0);
        let camera_right = glm::normalize(&glm::cross(&up, &camera_direction));

        let camera_up = glm::cross(&camera_direction, &camera_right);

        const RADIUS: f32 = 10.0;
        let cam_x = glfw.get_time().sin() as f32 * RADIUS;
        let cam_z = glfw.get_time().cos() as f32 * RADIUS;

        // let mut view = glm::Mat4::identity();
        let view = glm::look_at(
            &glm::vec3(cam_x, 0.0, cam_z),
            &glm::vec3(0.0, 0.0, 0.0),
            &glm::vec3(0.0, 1.0, 0.0),
        );
        // view = glm::translate(&view, &glm::vec3(0.0, 0.0, -3.0));

        ubo.next_attribute::<glm::Mat4, f32>(glm::value_ptr(&proj), 0);
        ubo.next_attribute::<glm::Mat4, f32>(glm::value_ptr(&view), 1);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            ubo.bind();

            gl::BindTextures(0, 2, [texture, texture_2].as_ptr());

            program.use_program();
            gl::BindVertexArray(vao);
            for x in 0..10 {
                let mut model = glm::Mat4::identity();
                model = glm::translate(&model, &cube_positions[x]);
                let angle = 20.0 * x as f32;
                model = glm::rotate(&model, angle * RADIANS, &glm::vec3(1.0, 0.3, 0.5));

                ubo.next_attribute::<glm::Mat4, f32>(glm::value_ptr(&model), 2);
                gl::DrawElements(gl::TRIANGLES, 36, gl::UNSIGNED_INT, ptr::null());
            }
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
    fn new(mut self, mut vao: u32, buffer: u32, size: isize) -> Self {
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            gl::VertexArrayVertexBuffer(vao, 0, buffer, size, 5 * size_of(gl::FLOAT) as i32);
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

    fn build(self) -> u32 {
        self.vao
    }
}

struct UBO {
    ubo: u32,
}

impl UBO {
    fn new(shader: u32, ubo_name: &str, size: usize) -> UBO {
        unsafe {
            let size = size as isize;
            let ubo_name = CString::new(ubo_name).unwrap();
            let index = gl::GetUniformBlockIndex(shader, ubo_name.as_ptr());

            gl::UniformBlockBinding(shader, index, 0);

            let ubo = Buffer::create(size);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);

            gl::BindBufferRange(gl::UNIFORM_BUFFER, index, ubo, 0, size);
            UBO { ubo }
        }
    }

    fn next_attribute<A, B>(&mut self, data: &[B], offset: isize) -> &Self {
        unsafe {
            let size = mem::size_of::<A>() as isize;
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                offset * size,
                size,
                data.as_ptr() as *const c_void,
            );
            self
        }
    }

    fn bind(&self) -> &Self {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            &self
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

fn handle_window_event(
    window: &mut glfw::Window,
    events: &Receiver<(f64, glfw::WindowEvent)>,
    proj: &mut Mat4,
) {
    for (_, events) in glfw::flush_messages(events) {
        match events {
            glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                window.set_should_close(true)
            }
            glfw::WindowEvent::FramebufferSize(width, height) => {
                framebuffer_size_callback(width, height, proj)
            }
            _ => {}
        }
    }
}

fn framebuffer_size_callback(width: i32, height: i32, proj: &mut Mat4) {
    unsafe {
        gl::Viewport(0, 0, width, height);
        *proj = glm::perspective(width as f32 / height as f32, 45.0 * RADIANS, 0.1, 100.0);
    }
}

fn size_of(glenum: gl::types::GLenum) -> u32 {
    match glenum {
        gl::FLOAT => 4,
        _ => unreachable!(),
    }
}
