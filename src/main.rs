use std::ffi::{c_char, c_void, CString};
use std::{ffi::c_float, sync::mpsc::Receiver};
use std::{mem, ptr};

use glfw::{Action, Context, Key};

use opengl::gl;

const VERTEX_SHADER_SOURCE: &str = "
#version 330 core
layout (location = 0) in vec3 aPos;
layout (location = 1) in vec3 aColor;

out vec3 ourColor;

void main()
{
    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);
    ourColor = aColor;
}
";

const FRAGMENT_SHADER_SOURCE: &str = "
#version 330 core
out vec4 FragColor;
in vec3 ourColor;

void main()
{
    FragColor = vec4(ourColor, 1.0);
}
";

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

    glfw.make_context_current(Some(&window));

    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    framebuffer_size_callback(1280, 720);

    let (program, vao) = unsafe {
        let mut success = gl::FALSE as i32;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);

        let vertex_shader: u32 = gl::CreateShader(gl::VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as i32 {
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut c_char,
            );
            println!("Vertex Compliation Failed");
        }

        let fragment_shader: u32 = gl::CreateShader(gl::FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT_SHADER_SOURCE.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as i32 {
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut c_char,
            );
            println!("Fragment Compliation Failed");
        }

        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as i32 {
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut c_char,
            )
        }

        gl::DeleteShader(fragment_shader);
        gl::DeleteShader(vertex_shader);

        let verticies: [f32; 24] = [
            // Positions   | Colors
             0.5,  0.5, 0.0,  1.0, 0.0, 0.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
             0.5, -0.5, 0.0,  0.0, 1.0, 0.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
            -0.5, -0.5, 0.0,  0.0, 0.0, 1.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
            -0.5,  0.5, 0.0,  0.0, 0.0, 0.0, // STOP FUCKING FORMATTING ME THANK YOU CARGO
        ];

        let indicies: [i32; 6] = [
            0, 1, 3, // Stop
            1, 2, 3, // Stop
        ];

        let (mut vbo, mut vao, mut ebo) = (0, 0, 0);

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::GenBuffers(1, &mut ebo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);

        // VBO Buffer Data
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (verticies.len() * mem::size_of::<c_float>()) as isize,
            &verticies[0] as *const f32 as *const c_void,
            gl::STATIC_DRAW,
        );

        // EBO Buffer Data
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (indicies.len() * mem::size_of::<c_float>()) as isize,
            &indicies[0] as *const i32 as *const c_void,
            gl::STATIC_DRAW,
        );

        // VAO Buffer Data
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<c_float>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            6 * mem::size_of::<c_float>() as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
        (shader_program, vao)
    };

    while !window.should_close() {
        handle_window_event(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // let time_value = glfw.get_time();
            // let green_value = (time_value.sin() / 2.0) + 0.5;
            //
            // let vertex_color_location = gl::GetUniformLocation(program, CString::new("ourColor").unwrap().as_ptr());

            gl::UseProgram(program);
            // gl::Uniform4f(vertex_color_location, 0.0, green_value as c_float, 0.0, 1.0);
            gl::BindVertexArray(vao);
            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, ptr::null());
        }

        window.swap_buffers();
        glfw.poll_events();
    }
}

fn handle_window_event(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, events) in glfw::flush_messages(&events) {
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
