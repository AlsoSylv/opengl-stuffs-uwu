use std::ffi::c_void;

use image::DynamicImage;
use opengl::gl;

pub(crate) struct TextureBuilder {
    texture: u32,
    image: DynamicImage,
    internal_format: gl::types::GLenum,
    internalformat: gl::types::GLenum,
}

#[allow(dead_code)]
impl TextureBuilder {
    pub fn new(
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

    pub fn texture_storage(self, levels: i32) -> Self {
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

    pub fn sub_texture(self, x_offset: i32, y_offset: i32) -> Self {
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

    pub fn texture_paramater_i(self, p_name: gl::types::GLenum, param: gl::types::GLenum) -> Self {
        unsafe {
            gl::TextureParameteri(self.texture, p_name, param as i32);
            self
        }
    }

    pub fn flip(mut self) -> Self {
        self.image = self.image.flipv();
        self
    }

    pub fn build(&self) -> u32 {
        self.texture
    }
}
