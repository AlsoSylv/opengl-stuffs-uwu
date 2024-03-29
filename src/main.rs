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

use buffers::{Buffer, UBO, VertexBuilder};
use camera::Camera;
use shaders::Shader;
use textures::{TextureBuilder, TextureManager};

const VERTEX_SHADER_SOURCE: &str = "./resources/shaders/vertex.vert";
const FRAGMENT_SHADER_SOURCE: &str = "./resources/shaders/fragment.frag";

#[allow(dead_code)]
const RADIANS: f32 = PI / 180.0;

fn gl_enable(cap: gl::types::GLenum) {
    unsafe {
        gl::Enable(cap)
    }
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
    let (mut last_x, mut last_y) = (400.0, 300.0);

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
    let indicies: [i32; 36] = [
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

    unsafe {
        gl::Viewport(0, 0, 1280, 720);
    }

    gl_enable(gl::DEPTH_TEST);

    let buffer = Buffer::create_shared_buffer(&verticies, &indicies);

    let mut vao = 0;
    let mut light_vao = 0;

    VertexBuilder::bind_buffers(buffer, &indicies, &mut vao, &mut light_vao)
        .attribute(3, gl::FLOAT)
        .attribute(2, gl::FLOAT)
        .attribute(3, gl::FLOAT);

    let texture_manager = {
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

        let mut texture_manager = TextureManager::new();
        texture_manager.add_texture(texture);
        texture_manager.add_texture(texture_2);

        texture_manager
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

    let mut last_frame = 0.0;

    let camera = Camera::new();

    let mut app = Application::new(&mut window, &events, camera);

    let mut matrix_block = UBO::new(3 * mem::size_of::<glm::Mat4>());
    matrix_block.attach_new_shader(&shaders, "MatrixBlock");
    matrix_block.attach_new_shader(&light_shader, "MatrixBlock");
    matrix_block.bind();
    texture_manager.bind_texutres(0);

    while !app.should_close() {
        let current_time = glfw.get_time();
        let delta = current_time - last_frame;
        last_frame = current_time;

        app.handle_window_event(&mut proj, delta as f32, &mut last_x, &mut last_y);

        matrix_block.next_attribute::<glm::Mat4, f32>(glm::value_ptr(&proj));
        matrix_block.next_attribute::<glm::Mat4, f32>(glm::value_ptr(&app.view()));

        app.clear();
        shaders.use_program();
        app.bind_vao(vao);

        for (x, position) in cube_positions.iter().enumerate() {
            let mut model = glm::Mat4::identity();
            let angle = 20.0 * x as f32;

            model = glm::translate(&model, position);
            model = glm::rotate(&model, angle * RADIANS, &glm::vec3(1.0, 0.3, 0.5));

            matrix_block.next_attribute_reduced::<glm::Mat4, f32>(glm::value_ptr(&model));
            app.draw(36);
        }

        light_shader.use_program();
        app.bind_vao(light_vao);

        light_shader.set_vec3("objectColor", 1, &Vec3::new(1.0, 0.5, 0.31));
        light_shader.set_vec3("lightColor", 1, &Vec3::new(1.0, 1.0, 1.0));

        for (x, position) in cube_positions.iter().enumerate() {
            let light_pos = glm::vec3(1.2, 1.0, 2.0);
            let mut model = glm::Mat4::identity();
            let angle = 20.0 * x as f32;

            model = glm::translate(&model, &position);
            model = glm::translate(&model, &light_pos);
            model = glm::scale(&model, &glm::vec3(0.2, 0.2, 0.2));
            model = glm::rotate(&model, angle * RADIANS, &glm::vec3(1.0, 0.3, 0.5));

            matrix_block.next_attribute_reduced::<glm::Mat4, f32>(glm::value_ptr(&model));
            app.draw(36);
        }

        matrix_block.clear();

        app.finish_frame();
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
        last_x: &mut f64,
        last_y: &mut f64,
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
                glfw::WindowEvent::CursorPos(x_pos, y_pos) => {
                    const SENSATIVITY: f64 = 0.1;
                    let x_offset = SENSATIVITY * (x_pos - *last_x);
                    let y_offset = SENSATIVITY * (y_pos - *last_y);
                    *last_x = x_pos;
                    *last_y = y_pos;

                    self.camera.update_camer_pos(x_offset, -y_offset);
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
            self.camera.forward(speed)
        }
        if self.keys.contains(&Key::S) {
            self.camera.backwards(speed)
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
