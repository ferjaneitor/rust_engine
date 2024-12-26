use glutin::event::VirtualKeyCode;

use crate::math::{matrix_4_by_4::Matrix4, vec3::Vec3};

pub struct Camera {
    pub position: Vec3,
    pub yaw: f32,   // rotación alrededor de Y
    pub pitch: f32, // rotación alrededor de X
    pub speed: f32, // velocidad de movimiento
}

impl Camera {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: 0.0,
            pitch: 0.0,
            speed: 2.0, // unidades por segundo (por ejemplo)
        }
    }

    /// Retorna la matriz de vista, calculada a partir de position, yaw y pitch
    pub fn get_view_matrix(&self) -> Matrix4 {
        // 1. Calcular la dirección "forward" según yaw/pitch
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

        // Dirección "right", perpendicular en el plano XZ
        let right = Vec3::new(cos_yaw, 0.0, -sin_yaw);
        // Arriba se obtiene con cross, pero en este caso,
        // definimos un "world up" = (0,1,0) y ajustamos con pitch si lo deseas

        // 2. "Target" = position + forward
        let target = self.position + forward;

        // 3. Generar la view con `look_at`
        Matrix4::look_at(self.position, target, Vec3::UNIT_Y)
    }

    /// Actualizar la posición de la cámara según la tecla presionada
    pub fn process_keyboard(&mut self, key: VirtualKeyCode, delta_time: f32) {
        let velocity = self.speed * delta_time;

        // Calcular la dirección "forward" y "right" para movernos
        let cos_pitch = self.pitch.cos();
        let sin_pitch = self.pitch.sin();
        let cos_yaw = self.yaw.cos();
        let sin_yaw = self.yaw.sin();

        let forward = Vec3::new(sin_yaw * cos_pitch, -sin_pitch, cos_yaw * cos_pitch);
        let right   = Vec3::new(cos_yaw, 0.0, -sin_yaw);

        match key {
            VirtualKeyCode::S => {
                self.position = self.position + forward * velocity;
            }
            VirtualKeyCode::W => {
                self.position = self.position - forward * velocity;
            }
            VirtualKeyCode::A => {
                self.position = self.position - right * velocity;
            }
            VirtualKeyCode::D => {
                self.position = self.position + right * velocity;
            }
            _ => {}
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
