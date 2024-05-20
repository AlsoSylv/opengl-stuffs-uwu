mod buffers;
mod camera;
mod shaders;
mod textures;

use std::collections::HashSet;
use std::f32::consts::PI;
use std::path::Path;
use std::sync::mpsc::Receiver;
use std::{mem, ptr};

use glfw::{Action, Context, Key};
use glm::{Mat4, Vec3};
use nalgebra_glm as glm;
use opengl::gl;

use buffers::{Buffer, VertexBuilder, UBO};
use camera::Camera;
use shaders::Shader;
use textures::{TextureBuilder, TextureManager};

const VERTEX_SHADER_SOURCE: &str = "./resources/shaders/vertex.vert";
const FRAGMENT_SHADER_SOURCE: &str = "./resources/shaders/fragment.frag";

#[allow(dead_code)]
const RADIANS: f32 = PI / 180.0;

fn gl_enable(cap: gl::types::GLenum) {
    unsafe { gl::Enable(cap) }
}

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

    window.make_current();

    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);
    window.set_cursor_pos_polling(true);

    gl::load_with(|symbol| window.get_proc_address(symbol));

    let mut proj = glm::perspective(1280.0 / 720.0, 45.0 * RADIANS, 0.1, 100.0);

    // Modern OGL: https://github.com/fendevel/Guide-to-Modern-OpenGL-Functions#glnamedbufferdata
    // Learn OGL: https://learnopengl.com/
    // Learn OGL RS: https://github.com/bwasty/learn-opengl-rs
    // ECS: https://www.youtube.com/watch?v=aKLntZcp27M
    let shaders = Shader::new(VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
    let light_shader = Shader::new(
        "./resources/shaders/light_vx.vert",
        "./resources/shaders/light_fx.frag",
    );

    #[rustfmt::skip]
    let vertices: [f32; 20] = [
        // Positions      | Texture coords
         0.25,  0.25, -0.25,  1.0, 1.0,  // 0
         0.25, -0.25, -0.25,  1.0, 0.0,  // 1
        -0.25, -0.25, -0.25,  0.0, 0.0,  // 2
        -0.25,  0.25, -0.25,  0.0, 1.0,  // 3
    ];

    #[rustfmt::skip]
    let indices: [i32; 6] = [
        0, 1, 3,  // Indexes of verts
        1, 2, 3,  // Indexes of verts
    ];

    unsafe {
        gl::Viewport(0, 0, 1280, 720);
    }

    gl_enable(gl::DEPTH_TEST);

    let buffer = Buffer::create_shared_buffer(&vertices, &indices);

    let mut vao = 0;

    VertexBuilder::bind_buffers(buffer, &indices, &mut vao)
        .attribute(3, gl::FLOAT)
        .attribute(2, gl::FLOAT)
        .attribute(3, gl::FLOAT);

    let texture_manager = {
        let img = image::open(Path::new("./resources/textures/wall.jpg")).unwrap();
        let img_2 = image::open(Path::new("./resources/textures/awesomeface.png")).unwrap();

        let texture = TextureBuilder::new(img, gl::RGB, gl::RGB8)
            .texture_parameter_i(gl::TEXTURE_WRAP_S, gl::MIRRORED_REPEAT)
            .texture_parameter_i(gl::TEXTURE_WRAP_T, gl::MIRRORED_REPEAT)
            .texture_parameter_i(gl::TEXTURE_MIN_FILTER, gl::NEAREST)
            .texture_parameter_i(gl::TEXTURE_MAG_FILTER, gl::LINEAR)
            .texture_storage(1)
            .sub_texture(0, 0)
            .build();

        let texture_2 = TextureBuilder::new(img_2, gl::RGBA, gl::RGBA8)
            .texture_storage(1)
            .sub_texture(0, 0)
            .build();

        let mut texture_manager = TextureManager::new();
        texture_manager.add_texture(texture);
        texture_manager.add_texture(texture_2);

        texture_manager
    };

    let cubes = [
        [0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0],
        [0, 0, 0, 0, 0, 0, 0],
    ];

    let mut last_frame = 0.0;

    let camera = Camera::new();

    let mut app = Application::new(&mut window, &events, camera);

    let mut matrix_block = UBO::new(3 * mem::size_of::<glm::Mat4>());
    matrix_block.attach_new_shader(&shaders, "MatrixBlock");
    matrix_block.attach_new_shader(&light_shader, "MatrixBlock");
    matrix_block.bind();
    texture_manager.bind_textures(0);

    let player_pos = Vec3::new(0.0, 0.0, 0.0);

    while !app.should_close() {
        let current_time = glfw.get_time();
        let delta = current_time - last_frame;
        last_frame = current_time;

        app.handle_window_event(&mut proj, delta as f32);

        matrix_block.next_attribute(&proj);
        matrix_block.next_attribute(&app.view());

        app.clear();
        shaders.use_program();
        app.bind_vao(vao);

        let cubes = &cubes[player_pos.x as usize..player_pos.x as usize + 5];

        cubes.iter().enumerate().for_each(|(index, sub_cubes)| {
            let index_isize = index as isize - 2;
            let x = index_isize as f32 * 0.5;

            let sub_cubes = &sub_cubes[player_pos.y as usize..player_pos.y as usize + 5];

            sub_cubes.iter().enumerate().for_each(|(index, _)| {
                let index_isize = index as isize - 2;
                let y = index_isize as f32 * 0.5;

                let mut model = glm::Mat4::identity();

                let position = Vec3::new(x, y, 0.0);

                model = glm::translate(&model, &position);
                matrix_block.next_attribute_reduced(&model);

                app.draw(6);
            });
        });

        matrix_block.clear();
        app.finish_frame();
    }
}

pub fn clamp(value: usize, min: usize, max: usize) -> usize {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

struct Application<'a> {
    window: &'a mut glfw::Window,
    events: &'a Receiver<(f64, glfw::WindowEvent)>,
    camera: Camera,
    keys: HashSet<Key>,
}

impl Application<'_> {
    fn new<'a>(
        window: &'a mut glfw::Window,
        events: &'a Receiver<(f64, glfw::WindowEvent)>,
        camera: Camera,
    ) -> Application<'a> {
        Application {
            window,
            events,
            camera,
            keys: HashSet::new(),
        }
    }

    fn handle_window_event(
        &mut self,
        proj: &mut Mat4,
        delta: f32,
    ) {
        let speed = delta * 2.5;
        for (_, events) in glfw::flush_messages(self.events) {
            match events {
                glfw::WindowEvent::Key(key, _, action, _) => match (key, action) {
                    (Key::Escape, _) => self.window.set_should_close(true),
                    (Key::W | Key::A | Key::S | Key::D, Action::Release) => {
                        self.keys.remove(&key);
                    }
                    (Key::W | Key::A | Key::S | Key::D, _) => {
                        self.keys.insert(key);
                    }
                    _ => (),
                },

                glfw::WindowEvent::FramebufferSize(width, height) => {
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }

                    let width = width as f32;
                    let height = height as f32;
                    *proj = glm::perspective(width / height, 45.0 * RADIANS, 0.1, 100.0);
                }
                glfw::WindowEvent::CursorPos(_, _) => {

                }
                _ => (),
            }
        }

        self.key_presses(speed);
    }

    fn should_close(&mut self) -> bool {
        self.window.should_close()
    }

    fn finish_frame(&mut self) {
        self.window.swap_buffers();
        self.window.glfw.poll_events();
    }

    fn view(&mut self) -> Mat4 {
        self.camera.view()
    }

    fn key_presses(&mut self, speed: f32) {
        if self.keys.contains(&Key::W) {
            self.camera.forward(speed);
            self.keys.remove(&Key::W);
        }
        if self.keys.contains(&Key::S) {
            self.camera.backwards(speed);
            self.keys.remove(&Key::S);
        }
        if self.keys.contains(&Key::A) {
            self.camera.left(speed)
        }
        if self.keys.contains(&Key::D) {
            self.camera.right(speed)
        }
    }

    fn draw(&self, count: i32) {
        unsafe {
            gl::DrawElements(gl::TRIANGLES, count, gl::UNSIGNED_INT, ptr::null());
        }
    }

    fn clear(&self) {
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn bind_vao(&self, vao: u32) {
        unsafe {
            gl::BindVertexArray(vao);
        }
    }
}

fn size_of(glenum: gl::types::GLenum) -> u32 {
    match glenum {
        gl::FLOAT => mem::size_of::<f32>() as u32,
        gl::INT => mem::size_of::<i32>() as u32,
        _ => unreachable!(),
    }
}
