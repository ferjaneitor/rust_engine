use std::collections::HashSet;

use glutin::event::VirtualKeyCode;

use crate::math::{matrix_4_by_4::Matrix4, vec3::Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // rotación alrededor de Y
    pub pitch: f32, // rotación alrededor de X
    pub speed: f32, // velocidad de movimiento
    pub vertical_speed: f32, // Nueva velocidad para movimiento vertical
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            speed: 10.0,          // Velocidad de movimiento horizontal (Unidades por segundo)
            vertical_speed: 10.0, // Velocidad de movimiento vertical (Unidades por segundo)
        }
    }

    /// Retorna la matriz de vista, calculada a partir de position, yaw y pitch
    pub fn get_view_matrix(&self) -> Matrix4 {
        Matrix4::look_at(self.position, self.position + self.get_forward_vector(), Vec3::UNIT_Y)
    }

    /// Retorna el vector forward basado en yaw y pitch
    fn get_forward_vector(&self) -> Vec3 {
        // . Calcular la dirección "forward" según yaw/pitch
        //    yaw   = rotación en Y
        //    pitch = rotación en X
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        // Dirección "forward" en 3D
        // alternativo, mira en -Z
        let forward = Vec3::new(
            - (sin_yaw * cos_pitch),
            - sin_pitch,
            - (cos_yaw * cos_pitch),
        );
        
        return  forward;
    }

     /// Procesa múltiples teclas presionadas para mover la cámara
     pub fn process_keys(&mut self, pressed: &HashSet<VirtualKeyCode>, dt: f32) {
        let velocity = self.speed * dt;
        let vertical_velocity = self.vertical_speed * dt;

        let forward = self.get_forward_vector();
        let right = forward.cross(&Vec3::UNIT_Y).normalize();
        let up = Vec3::UNIT_Y;

        // Movimiento horizontal
        if pressed.contains(&VirtualKeyCode::W) {
            self.position += forward * velocity;
        }
        if pressed.contains(&VirtualKeyCode::S) {
            self.position -= forward * velocity;
        }
        if pressed.contains(&VirtualKeyCode::A) {
            self.position -= right * velocity;
        }
        if pressed.contains(&VirtualKeyCode::D) {
            self.position += right * velocity;
        }

        // Movimiento vertical
        if pressed.contains(&VirtualKeyCode::Space) {
            self.position += up * vertical_velocity;
        }
        if pressed.contains(&VirtualKeyCode::LShift) || pressed.contains(&VirtualKeyCode::RShift) {
            self.position -= up * vertical_velocity;
        }
    }
    

    /// Actualizar la orientación (yaw/pitch) con el mouse
    pub fn process_mouse(&mut self, delta_x: f32, delta_y: f32) {
        // Ajustar sensibilidad
        let sensitivity = 0.001;
        self.yaw   += delta_x * sensitivity;
        self.pitch -= delta_y * sensitivity; // resta, para que mover mouse arriba gire la cámara hacia arriba

        // Limitar pitch para que no gire 180º
        if self.pitch > 1.5 {
            self.pitch = 1.5;
        }
        if self.pitch < -1.5 {
            self.pitch = -1.5;
        }
    }
}
