#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Float3Eps([i32; 3]);

impl Float3Eps {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        let scale = 1e4; // por ejemplo
        let ix = (x * scale).round() as i32;
        let iy = (y * scale).round() as i32;
        let iz = (z * scale).round() as i32;

        Self([ix, iy, iz])
    }
}
