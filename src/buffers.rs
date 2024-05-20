use std::{ffi::CString, mem, ptr};

use nalgebra_glm::{Scalar, TMat};
use opengl::gl;

use crate::{shaders::Shader, size_of};

pub struct VertexBuilder<'a> {
    next_attribute: u32,
    last_size: u32,
    vao: &'a mut u32,
}

impl VertexBuilder<'_> {
    pub fn bind_buffers<'a, T>(
        buffer: Buffer,
        indices: &[T],
        vao: &'a mut u32,
    ) -> VertexBuilder<'a> {
        let size = mem::size_of_val(indices) as isize;

        unsafe {
            gl::CreateVertexArrays(1, vao);
            gl::VertexArrayVertexBuffer(*vao, 0, buffer.0, size, 5 * size_of(gl::FLOAT) as i32);
            gl::VertexArrayElementBuffer(*vao, buffer.0);
        }

        VertexBuilder {
            next_attribute: 0,
            last_size: 0,
            vao,
        }
    }

    pub fn attribute(mut self, size: u32, _type: gl::types::GLenum) -> Self {
        unsafe {
            gl::EnableVertexArrayAttrib(*self.vao, self.next_attribute);
            gl::VertexArrayAttribFormat(
                *self.vao,
                self.next_attribute,
                size as i32,
                _type,
                gl::FALSE,
                self.last_size * size_of(gl::FLOAT),
            );
            gl::VertexArrayAttribBinding(*self.vao, self.next_attribute, 0);

            self.last_size += size;
            self.next_attribute += 1;
            self
        }
    }
}

#[allow(clippy::upper_case_acronyms)]
pub struct UBO {
    ubo: Buffer,
    offset: isize,
    size: isize,
}

impl UBO {
    pub fn new(size: usize) -> UBO {
        let size = size as isize;
        let ubo = Buffer::create(size);

        UBO {
            ubo,
            offset: 0,
            size,
        }
    }

    pub fn next_attribute<T: Scalar, const R: usize, const C: usize>(
        &mut self,
        data: &TMat<T, R, C>,
    ) {
        let size = mem::size_of::<TMat<T, R, C>>() as isize;

        assert!(
            size + self.offset <= self.size,
            "Attributes larger than specified!"
        );

        unsafe {
            gl::BufferSubData(
                gl::UNIFORM_BUFFER,
                self.offset,
                size,
                data.as_ptr() as *const _,
            );
        }

        self.offset += size;
    }

    pub fn next_attribute_reduced<T, const R: usize, const C: usize>(
        &mut self,
        data: &TMat<T, R, C>,
    ) where
        T: Scalar,
    {
        self.next_attribute(data);
        self.offset -= mem::size_of::<TMat<T, R, C>>() as isize;
    }

    pub fn attach_new_shader(&self, shader: &Shader, ubo_name: &str) {
        let ubo_name = CString::new(ubo_name).unwrap();
        let index = shader.get_uniform_block_index(&ubo_name);
        shader.uniform_block_binding(index);

        unsafe {
            gl::BindBufferRange(gl::UNIFORM_BUFFER, index, self.ubo.0, 0, self.size);
        }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::UNIFORM_BUFFER, self.ubo.0);
        }
    }

    pub fn clear(&mut self) {
        self.offset = 0;
    }
}

pub struct Buffer(u32);

impl Buffer {
    pub fn create(size: isize) -> Buffer {
        let mut buffer = 0;

        unsafe {
            gl::CreateBuffers(1, &mut buffer);
            gl::NamedBufferStorage(buffer, size, ptr::null(), gl::DYNAMIC_STORAGE_BIT);
        }

        Buffer(buffer)
    }

    pub fn create_shared_buffer<A, B>(vertices: &[A], indices: &[B]) -> Buffer {
        let vrt_size = mem::size_of_val(vertices) as isize;
        let ind_size = mem::size_of_val(indices) as isize;

        let buffer = Buffer::create(ind_size + vrt_size);

        unsafe {
            gl::NamedBufferSubData(buffer.0, 0, ind_size, indices.as_ptr() as *const _);
            gl::NamedBufferSubData(buffer.0, ind_size, vrt_size, vertices.as_ptr() as *const _);
        }

        buffer
    }
}
