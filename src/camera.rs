use std::ops::Mul;

use glm::Vec3;
use nalgebra_glm as glm;

pub struct Camera {
    camera_pos: Vec3,
    camera_front: Vec3,
    camera_up: Vec3,
}

impl Camera {
    pub fn new() -> Camera {
        let camera_pos = glm::vec3(0.0, 0.0, 3.0);
        let camera_front = glm::vec3(0.0, 0.0, -1.0);
        let camera_up = glm::vec3(0.0, 1.0, 0.0);

        Camera {
            camera_pos,
            camera_front,
            camera_up,
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

    pub fn view(&mut self) -> glm::Mat4 {
        glm::look_at(
            &self.camera_pos,
            &(self.camera_pos + self.camera_front),
            &self.camera_up,
        )
    }
}
