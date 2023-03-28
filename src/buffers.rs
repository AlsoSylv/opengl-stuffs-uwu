use std::{
    ffi::{c_void, CString},
    mem, ptr,
};

use opengl::gl;

use crate::size_of;

#[derive(Default)]
pub struct VertexBuilder {
    next_attribute: u32,
    last_size: u32,
    vao: u32,
}

impl VertexBuilder {
    pub fn bind_buffers(mut self, mut vao: u32, buffer: u32, size: isize) -> Self {
        unsafe {
            gl::CreateVertexArrays(1, &mut vao);
            gl::VertexArrayVertexBuffer(vao, 0, buffer, size, 5 * size_of(gl::FLOAT) as i32);
            gl::VertexArrayElementBuffer(vao, buffer);
            self.vao = vao;
            self
        }
    }

    pub fn attribute(mut self, size: u32, _type: gl::types::GLenum) -> Self {
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

    pub fn build(self) -> u32 {
        self.vao
    }
}

pub struct Ubo {
    ubo: u32,
    offset: isize,
    size: isize,
}

impl Ubo {
    pub fn new(shader: u32, ubo_name: &str, size: usize) -> Ubo {
        unsafe {
            let size = size as isize;
            let ubo_name = CString::new(ubo_name).unwrap();
            let index = gl::GetUniformBlockIndex(shader, ubo_name.as_ptr());

            gl::UniformBlockBinding(shader, index, 0);

            let ubo = Buffer::create(size);
            gl::BindBuffer(gl::UNIFORM_BUFFER, ubo);

            gl::BindBufferRange(gl::UNIFORM_BUFFER, index, ubo, 0, size);
            Ubo {
                ubo,
                offset: 0,
                size,
            }
        }
    }

    pub fn next_attribute<A, B>(&mut self, data: &[B]) -> &Self {
        unsafe {
            let size = mem::size_of::<A>() as isize;
            if size + self.offset > self.size {
                panic!("Too big")
            }
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                self.offset,
                size,
                data.as_ptr() as *const c_void,
            );
            self.offset += size;
            self
        }
    }

    pub fn bind(&self) -> &Self {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo);
            self
        }
    }

    pub fn clear(&mut self) -> &Self {
        self.offset = 0;
        self
    }

    pub fn reduce_offset<A>(&mut self) -> &Self {
        self.offset -= mem::size_of::<A>() as isize;
        self
    }
}

pub struct Buffer;

impl Buffer {
    pub fn create(size: isize) -> u32 {
        unsafe {
            let mut buffer = 0;
            gl::CreateBuffers(1, &mut buffer);
            gl::NamedBufferStorage(buffer, size, ptr::null(), gl::DYNAMIC_STORAGE_BIT);
            buffer
        }
    }

    pub fn create_shared_buffer<A, B>(verticies: &[A], indicies: &[B], ind_size: isize) -> (u32, u32) {
        unsafe {
            let mut alignment = 0;
            gl::GetIntegerv(gl::UNIFORM_BUFFER_OFFSET_ALIGNMENT, &mut alignment);

            let vao: u32 = 0;

            let vrt_size = (verticies.len() * mem::size_of::<A>()) as isize;

            let buffer = Buffer::create(ind_size + vrt_size);

            gl::NamedBufferSubData(buffer, 0, ind_size, indicies.as_ptr() as *const c_void);
            gl::NamedBufferSubData(buffer, ind_size, vrt_size, verticies.as_ptr() as *const c_void);

            (vao, buffer)
        }
    }
}
