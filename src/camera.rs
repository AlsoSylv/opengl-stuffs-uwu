use std::ops::Mul;

use glm::Vec3;
use nalgebra_glm as glm;

use crate::RADIANS;

pub struct Camera {
    camera_pos: Vec3,
    camera_front: Vec3,
    camera_up: Vec3,
    direction: Vec3,
    pitch: f32,
    yaw: f32,
}

impl Camera {
    pub fn new() -> Camera {
        let camera_pos = glm::vec3(0.0, 0.0, 3.0);
        let camera_front = glm::vec3(0.0, 0.0, -1.0);
        let camera_up = glm::vec3(0.0, 1.0, 0.0);
        let direction = Vec3::identity();

        Camera {
            camera_pos,
            camera_front,
            camera_up,
            direction,
            yaw: -90.0,
            pitch: 0.0,
        }
    }

    pub fn forward(&mut self, speed: f32) {
        self.add(self.camera_front, speed)
    }

    pub fn backwards(&mut self, speed: f32) {
        self.camera_pos -= self.camera_front.mul(speed)
    }

    pub fn left(&mut self, speed: f32) {
        self.camera_pos -=
            glm::normalize(&glm::cross(&self.camera_front, &self.camera_up)).mul(speed)
    }

    pub fn right(&mut self, speed: f32) {
        self.add(
            glm::normalize(&glm::cross(&self.camera_front, &self.camera_up)),
            speed,
        )
    }

    pub fn add(&mut self, value: Vec3, speed: f32) {
        self.camera_pos += value.mul(speed)
    }

    pub fn update_camer_pos(&mut self, x_offset: f64, y_offset: f64) {
        self.yaw += x_offset as f32;
        self.pitch += y_offset as f32;

        self.pitch = clamp(self.pitch, 89.0, -89.0);

        let rads = self.pitch *RADIANS;
        let yar = self.yaw * RADIANS;

        self.direction.x = (yar).cos() * (rads).cos();
        self.direction.y = (rads).sin();
        self.direction.z = (yar).sin() * (rads).cos();
        self.camera_front = glm::normalize(&self.direction);
    }

    pub fn view(&mut self) -> glm::Mat4 {
        glm::look_at(
            &self.camera_pos,
            &(self.camera_pos + self.camera_front),
            &self.camera_up,
        )
    }
}

pub fn clamp<T>(val: T, max: T, min: T) -> T where T: PartialOrd<T> {
    let t = if val < min { min } else { val };
    if t > max { max } else { t }
}
