use std::hash::{Hash, Hasher};

#[derive(Clone, Copy)]
struct Float3([f32; 3]);

impl Float3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self([x, y, z])
    }
}

// Implementar PartialEq, Eq, Hash en base a los bits de f32:
impl PartialEq for Float3 {
    fn eq(&self, other: &Self) -> bool {
        self.0[0].to_bits() == other.0[0].to_bits() &&
        self.0[1].to_bits() == other.0[1].to_bits() &&
        self.0[2].to_bits() == other.0[2].to_bits()
    }
}
impl Eq for Float3 {}

impl Hash for Float3 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Para cada componente, escribe los bits
        state.write_u32(self.0[0].to_bits());
        state.write_u32(self.0[1].to_bits());
        state.write_u32(self.0[2].to_bits());
    }
}
