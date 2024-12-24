use std::ops::{Add, Sub, Mul, Div, AddAssign, SubAssign, MulAssign};

// Estructura para representar un vector 3D
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub const ZERO: Self = Self { x: 0.0, y: 0.0, z: 0.0 };
    pub const UNIT_X: Self = Self { x: 1.0, y: 0.0, z: 0.0 };
    pub const UNIT_Y: Self = Self { x: 0.0, y: 1.0, z: 0.0 };
    pub const UNIT_Z: Self = Self { x: 0.0, y: 0.0, z: 1.0 };

    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }   

    #[inline(always)]
    pub fn magnitude(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    pub fn normalize(&self) -> Self {
        let mag = self.magnitude();
        if mag == 0.0 {
            panic!("Attempt to normalize a zero vector");
        } else {
            *self / mag
        }
    }

    pub fn dot(&self, other: &Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: &Self) -> Self {
        if self.magnitude() == 0.0 || other.magnitude() == 0.0 {
            panic!("Cannot compute cross product with zero vector");
        }
    
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }
    

    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        let t_clamped = t.clamp(0.0, 1.0); // Clamp t between 0 and 1
        *self + (*other - *self) * t_clamped
    }    

    pub fn reflect(&self, normal: &Self) -> Self {
        *self - normal.scale(2.0 * self.dot(normal))
    }

    pub fn project(&self, other: &Self) -> Self {
        let scalar = self.dot(other) / other.dot(other);
        other.scale(scalar)
    }

    pub fn scale(&self, scalar: f32) -> Self {
        *self * scalar
    }

    pub fn angle_between(&self, other: &Self) -> f32 {
        let dot_product = self.dot(other);
        let magnitudes = self.magnitude() * other.magnitude();
        (dot_product / magnitudes).acos()
    }
}

// Operadores
impl Add for Vec3 {
    type Output = Self;

    fn add(self, other: Self) -> Self::Output {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }
}

impl Sub for Vec3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self::Output {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }
}

impl Mul<f32> for Vec3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self::Output {
        Self::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }
}

impl Div<f32> for Vec3 {
    type Output = Self;

    fn div(self, scalar: f32) -> Self::Output {
        Self::new(self.x / scalar, self.y / scalar, self.z / scalar)
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::UNIT_X // or Self::ZERO if you prefer
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl MulAssign<f32> for Vec3 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl From<[f32; 3]> for Vec3 {
    fn from(arr: [f32; 3]) -> Self {
        Self::new(arr[0], arr[1], arr[2])
    }
}

impl From<Vec3> for [f32; 3] {
    fn from(vec: Vec3) -> Self {
        [vec.x, vec.y, vec.z]
    }
}

// Pruebas unitarias
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magnitude() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.magnitude(), 5.0);
    }

    #[test]
    fn test_normalize() {
        let v = Vec3::new(0.0, 3.0, 4.0);
        let normalized = v.normalize();
        assert!((normalized.magnitude() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_add() {
        let v1 = Vec3::new(1.0, 2.0, 3.0);
        let v2 = Vec3::new(4.0, 5.0, 6.0);
        let result = v1 + v2;
        assert_eq!(result, Vec3::new(5.0, 7.0, 9.0));
    }

    #[test]
    fn test_scale() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let result = v * 2.0;
        assert_eq!(result, Vec3::new(2.0, 4.0, 6.0));
    }

    #[test]
    fn test_reflect() {
        let v = Vec3::new(1.0, -1.0, 0.0);
        let normal = Vec3::new(0.0, 1.0, 0.0);
        let reflected = v.reflect(&normal);
        assert_eq!(reflected, Vec3::new(1.0, 1.0, 0.0));
    }

    #[test]
    fn test_project() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        let onto = Vec3::new(1.0, 0.0, 0.0);
        let projection = v.project(&onto);
        assert_eq!(projection, Vec3::new(3.0, 0.0, 0.0));
    }

    #[test]
    fn test_angle_between() {
        let v1 = Vec3::new(1.0, 0.0, 0.0);
        let v2 = Vec3::new(0.0, 1.0, 0.0);
        let angle = v1.angle_between(&v2);
        assert!((angle - std::f32::consts::FRAC_PI_2).abs() < 1e-6); // Pi/2
    }

    #[test]
    fn test_normalize_zero() {
        let v = Vec3::ZERO;
        assert_eq!(v.normalize(), Vec3::ZERO);
    }

    #[test]
    fn test_large_magnitude() {
        let v = Vec3::new(1e10, 1e10, 1e10);
        let normalized = v.normalize();
        assert!((normalized.magnitude() - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_from_array() {
        let arr = [1.0, 2.0, 3.0];
        let v: Vec3 = arr.into();
        assert_eq!(v, Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_into_array() {
        let v = Vec3::new(1.0, 2.0, 3.0);
        let arr: [f32; 3] = v.into();
        assert_eq!(arr, [1.0, 2.0, 3.0]);
    }

}