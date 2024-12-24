use crate::math::vec3::Vec3;

#[derive(Copy, Clone)]
pub struct Math4 {
    pub m: [f32; 16], // almacenamos en columna mayor (OpenGL style)
}

impl Math4 {
    pub fn identity() -> Self {
        Self {
            m: [
                1.0, 0.0, 0.0, 0.0, // Columna 0
                0.0, 1.0, 0.0, 0.0, // Columna 1
                0.0, 0.0, 1.0, 0.0, // Columna 2
                0.0, 0.0, 0.0, 1.0, // Columna 3
            ],
        }
    }

    pub fn multiply(&self, other: &Math4) -> Math4 {
        let mut result = Math4 { m: [0.0; 16] };
        for col in 0..4 {
            for row in 0..4 {
                let mut sum = 0.0;
                for i in 0..4 {
                    sum += self.m[row + i * 4] * other.m[i + col * 4];
                }
                result.m[row + col * 4] = sum;
            }
        }
        result
    }

    pub fn translate(tx: f32, ty: f32, tz: f32) -> Math4 {
        let mut math = Math4::identity();
        math.m[12] = tx;
        math.m[13] = ty;
        math.m[14] = tz;
        math
    }

    pub fn rotate_y(angle_radians: f32) -> Math4 {
        let mut math = Math4::identity();
        let c = angle_radians.cos();
        let s = angle_radians.sin();
        math.m[0] = c;
        math.m[2] = s;
        math.m[8] = -s;
        math.m[10] = c;
        math
    }

    pub fn rotate_x(angle: f32) -> Math4 {
        let mut math = Math4::identity();
        let c = angle.cos();
        let s = angle.sin();
        math.m[5] = c;
        math.m[6] = -s;
        math.m[9] = s;
        math.m[10] = c;
        math
    }

    pub fn perspective(fov_radians: f32, aspect: f32, near: f32, far: f32) -> Math4 {
        let f = 1.0 / (fov_radians / 2.0).tan();
        let mut math = Math4 { m: [0.0; 16] };
        math.m[0] = f / aspect;
        math.m[5] = f;
        math.m[10] = (far + near) / (near - far);
        math.m[11] = -1.0;
        math.m[14] = (2.0 * far * near) / (near - far);
        math
    }

    /// Cámara "LookAt" con `Vec3`
    /// eye    = posición de la cámara
    /// center = a dónde mira
    /// up     = vector "arriba"
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) -> Math4 {
        // forward: dirección de la cámara
        let f = (center - eye).normalize();
        // right
        let s = f.cross(&up).normalize();
        // verdadero up
        let u = s.cross(&f);

        let mut math = Math4::identity();
        math.m[0] = s.x;
        math.m[4] = s.y;
        math.m[8] = s.z;

        math.m[1] = u.x;
        math.m[5] = u.y;
        math.m[9] = u.z;

        // En OpenGL se asume Z "hacia adentro", por eso -f
        math.m[2] = -f.x;
        math.m[6] = -f.y;
        math.m[10] = -f.z;

        // Trasladar la escena al opuesto de eye
        math.multiply(&Math4::translate(-eye.x, -eye.y, -eye.z))
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }
}
