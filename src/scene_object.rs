use stl_io::{self};
use std::{
    collections::HashMap, fs::File, str
};

use crate::math::{float3_eps::Float3Eps, matrix_4_by_4::Matrix4};

/// Estructura para acumular datos de cada vértice
/// - pos: posición (x, y, z)
/// - normal: normal acumulada (nx, ny, nz)
#[derive(Debug)]
pub struct VertexData {
    pos: [f32; 3],
    normal: [f32; 3],
}

pub struct SceneObject {
    pub vao: u32,
    pub index_count: i32,
    pub base_transform: Matrix4,  // posición inicial
    pub angle: f32,               // rotación acumulada
    pub angular_speed: f32,       // rotación por segundo
    pub scale_factor: f32,        // escala actual
}

impl SceneObject{

    pub fn new(vao: u32, index_count: i32) -> SceneObject {
        Self {
            vao,
            index_count,
            base_transform: Matrix4::identity(),
            angle: 0.0,
            angular_speed: 0.0,
            scale_factor: 1.0,
        }
    }

    /// Carga un STL y calcula normales "smooth" promediadas.
    /// Devuelve (positions, normals, indices).
    /// - `positions`: [x0, y0, z0, x1, y1, z1, ...]
    /// - `normals`:   [nx0, ny0, nz0, nx1, ny1, nz1, ...]
    /// - `indices`:   [i0, i1, i2, ...] (u32)
    fn load_stl_model_smooth(path: &str) -> (Vec<f32>, Vec<f32>, Vec<u32>) {
        // 1. Abrir el archivo
        let mut file = File::open(path)
            .unwrap_or_else(|_| panic!("No se pudo abrir el archivo STL: {}", path));

        // 2. Parsear con stl_io
        let mesh = stl_io::read_stl(&mut file)
            .expect("Error parseando el archivo STL");

        // Mapa para unificar vértices:
        //  key: (x, y, z)
        //  val: índice en el vector "unique_vertices"
        let mut vertex_map: HashMap<Float3Eps, u32> = HashMap::new();
        let mut unique_vertices: Vec<VertexData> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        // 3. Recorrer todas las caras
        for face in &mesh.faces {
            let face_normal = face.normal;

            for &idx in &face.vertices {
                let vpos = mesh.vertices[idx];
                let key = Float3Eps::new(vpos[0], vpos[1], vpos[2]);

                // ********** IMPORTANTE **********
                let vert_index = if let Some(&existing_idx) = vertex_map.get(&key) {
                    // Si ya existe, devolvemos su índice
                    existing_idx
                } else {
                    // No existe, creamos uno nuevo
                    let new_idx = unique_vertices.len() as u32;
                    vertex_map.insert(key, new_idx);

                    unique_vertices.push(VertexData {
                        pos: [vpos[0], vpos[1], vpos[2]],
                        normal: [0.0, 0.0, 0.0],
                    });
                    
                    new_idx
                };

                // Acumulamos la normal de la cara en ese vértice
                let vdata_mut = &mut unique_vertices[vert_index as usize];
                vdata_mut.normal[0] += face_normal[0];
                vdata_mut.normal[1] += face_normal[1];
                vdata_mut.normal[2] += face_normal[2];

                // Agregar índice al EBO
                indices.push(vert_index);
            }
        }

        // 4. Normalizar las normales de cada vértice
        for v in &mut unique_vertices {
            let nx = v.normal[0];
            let ny = v.normal[1];
            let nz = v.normal[2];
            let length = (nx * nx + ny * ny + nz * nz).sqrt();
            if length > 1e-8 {
                v.normal[0] /= length;
                v.normal[1] /= length;
                v.normal[2] /= length;
            }
            // si length=0 => dejarla en (0,0,0) => vértice aislado o degenerado
        }

        // 5. Construir los vectores finales (positions, normals)
        let mut positions: Vec<f32> = Vec::with_capacity(unique_vertices.len() * 3);
        let mut normals: Vec<f32>   = Vec::with_capacity(unique_vertices.len() * 3);

        for v in &unique_vertices {
            positions.push(v.pos[0]);
            positions.push(v.pos[1]);
            positions.push(v.pos[2]);

            normals.push(v.normal[0]);
            normals.push(v.normal[1]);
            normals.push(v.normal[2]);
        }

        (positions, normals, indices)
    }

    pub fn create_object_from_stl(path: &str) -> SceneObject {
        // 1) Carga el STL con tus normales "smooth"
        let (positions, normals, indices) = SceneObject::load_stl_model_smooth(path);
    
        // 2) Genera VAO, VBO pos, VBO normal, EBO
        let mut vao = 0;
        let mut vbo_pos = 0;
        let mut vbo_nor = 0;
        let mut ebo = 0;
        let index_count = indices.len() as i32;
    
        unsafe {
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo_pos);
            gl::GenBuffers(1, &mut vbo_nor);
            gl::GenBuffers(1, &mut ebo);
    
            gl::BindVertexArray(vao);
    
            // VBO de posiciones
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_pos);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (positions.len() * std::mem::size_of::<f32>()) as isize,
                positions.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            // (location=0)
            gl::VertexAttribPointer(
                0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);
    
            // VBO de normales
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo_nor);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (normals.len() * std::mem::size_of::<f32>()) as isize,
                normals.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            // (location=1)
            gl::VertexAttribPointer(
                1, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null(),
            );
            gl::EnableVertexAttribArray(1);
    
            // EBO
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (indices.len() * std::mem::size_of::<u32>()) as isize,
                indices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
    
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::BindVertexArray(0);
        }
    
        // 3) Crear el SceneObject
        SceneObject {
            vao,
            index_count,
            base_transform: Matrix4::identity(),
            angle: 0.0,           // <--- valor por defecto
            angular_speed: 0.0,   // <--- valor por defecto
            scale_factor: 1.0,    // <--- valor por defecto
        }
    }
    
}