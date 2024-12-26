use crate::math::vec3::Vec3;

#[derive(Copy, Clone)]
pub struct Matrix4 {
    pub m: [f32; 16], // almacenamos en columna mayor (OpenGL style)
}

impl Matrix4 {
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

    pub fn multiply(&self, other: &Matrix4) ->Matrix4 {
        let mut result =Matrix4 { m: [0.0; 16] };
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

    pub fn translate(tx: f32, ty: f32, tz: f32) ->Matrix4 {
        let mut matrix =Matrix4::identity();
        matrix.m[12] = tx;
        matrix.m[13] = ty;
        matrix.m[14] = tz;
        matrix
    }

    pub fn rotate_y(angle_radians: f32) ->Matrix4 {
        let mut matrix =Matrix4::identity();
        let c = angle_radians.cos();
        let s = angle_radians.sin();
        matrix.m[0] = c;
        matrix.m[2] = s;
        matrix.m[8] = -s;
        matrix.m[10] = c;
        matrix
    }

    pub fn rotate_x(angle: f32) ->Matrix4 {
        let mut matrix =Matrix4::identity();
        let c = angle.cos();
        let s = angle.sin();
        matrix.m[5] = c;
        matrix.m[6] = -s;
        matrix.m[9] = s;
        matrix.m[10] = c;
        matrix
    }

    pub fn perspective(fov_radians: f32, aspect: f32, near: f32, far: f32) ->Matrix4 {
        let f = 1.0 / (fov_radians / 2.0).tan();
        let mut matrix =Matrix4 { m: [0.0; 16] };
        matrix.m[0] = f / aspect;
        matrix.m[5] = f;
        matrix.m[10] = (far + near) / (near - far);
        matrix.m[11] = -1.0;
        matrix.m[14] = (2.0 * far * near) / (near - far);
        matrix
    }

    /// Cámara "LookAt" con `Vec3`
    /// eye    = posición de la cámara
    /// center = a dónde mira
    /// up     = vector "arriba"
    pub fn look_at(eye: Vec3, center: Vec3, up: Vec3) ->Matrix4 {
        // forward: dirección de la cámara
        let f = (center - eye).normalize();
        // right
        let s = f.cross(&up).normalize();
        // verdadero up
        let u = s.cross(&f);

        let mut matrix =Matrix4::identity();
        matrix.m[0] = s.x;
        matrix.m[4] = s.y;
        matrix.m[8] = s.z;

        matrix.m[1] = u.x;
        matrix.m[5] = u.y;
        matrix.m[9] = u.z;

        // En OpenGL se asume Z "hacia adentro", por eso -f
        matrix.m[2] = -f.x;
        matrix.m[6] = -f.y;
        matrix.m[10] = -f.z;

        // Trasladar la escena al opuesto de eye
        matrix.multiply(&Matrix4::translate(-eye.x, -eye.y, -eye.z))
    }

    pub fn as_ptr(&self) -> *const f32 {
        self.m.as_ptr()
    }
    
    pub fn scale(s: f32) ->Matrix4 {
        let mut matrix =Matrix4::identity();
        matrix.m[0] = s;
        matrix.m[5] = s;
        matrix.m[10] = s;
        matrix
    }
    
    
}
