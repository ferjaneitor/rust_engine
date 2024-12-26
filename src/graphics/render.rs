// src/graphics/render.rs

use crate::graphics::shaders::{compile_shader, link_program};
use crate::graphics::window::Window;
use crate::graphics::scene_object::SceneObject;
use crate::graphics::camara::Camera;
use crate::math::matrix_4_by_4::Matrix4;

use std::{fs, ptr, str};

pub struct Renderer {
    pub program: u32,
    // Podrías guardar uniform locations, etc.
}

impl Renderer {
    pub fn new(vert_path: &str, frag_path: &str) -> Result<Self, String> {
        // 1) leer los archivos .vert y .frag
        let vert_source = fs::read_to_string(vert_path)
            .map_err(|e| format!("No se pudo leer {}: {}", vert_path, e))?;
        let frag_source = fs::read_to_string(frag_path)
            .map_err(|e| format!("No se pudo leer {}: {}", frag_path, e))?;

        // 2) Compilar
        let vs = compile_shader(&vert_source, gl::VERTEX_SHADER)?;
        let fs = compile_shader(&frag_source, gl::FRAGMENT_SHADER)?;
        // 3) Link
        let program = link_program(vs, fs)?;

        Ok(Self {
            program
        })
    }

    pub fn render_scene(
        &self,
        window: &Window,
        objects: &mut [SceneObject],
        camera: &Camera,
        global_scale: f32,
    ) {
        // Limpieza de buffers
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }

        unsafe {
            // Activar shader
            gl::UseProgram(self.program);

            // Ubicar uniformes
            let light_dir_loc = gl::GetUniformLocation(self.program, b"lightDir\0".as_ptr() as *const i8);
            let light_color_loc = gl::GetUniformLocation(self.program, b"lightColor\0".as_ptr() as *const i8);
            let object_color_loc = gl::GetUniformLocation(self.program, b"objectColor\0".as_ptr() as *const i8);

            gl::Uniform3f(light_dir_loc, 1.0, 1.0, 1.0);
            gl::Uniform3f(light_color_loc, 1.0, 1.0, 1.0);
            gl::Uniform3f(object_color_loc, 0.8, 0.8, 0.8);

            let model_loc = gl::GetUniformLocation(self.program, b"model\0".as_ptr() as *const i8);
            let view_loc  = gl::GetUniformLocation(self.program, b"view\0".as_ptr() as *const i8);
            let proj_loc  = gl::GetUniformLocation(self.program, b"projection\0".as_ptr() as *const i8);

            // Construir view y projection
            let view = camera.get_view_matrix();
            let size = window.context.window().inner_size();
            let aspect = size.width as f32 / size.height as f32;
            let projection = Matrix4::perspective(45.0_f32.to_radians(), aspect, 0.01, 1000.0);

            gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
            gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());

            // Dibujar cada objeto
            for obj in objects {
                obj.angle += obj.angular_speed * 0.016; // si deseas dt aquí
                // rotar en Y con obj.angle
                let rot_mat = Matrix4::rotate_y(obj.angle);
                // escala global
                let scale_mat = Matrix4::scale(global_scale);
                let local_anim = Matrix4::multiply(&scale_mat, &rot_mat);

                let final_model = Matrix4::multiply(&local_anim, &obj.base_transform);

                gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, final_model.as_ptr());
                gl::BindVertexArray(obj.vao);
                gl::DrawElements(gl::TRIANGLES, obj.index_count, gl::UNSIGNED_INT, ptr::null());
            }

            // Intercambiar buffers
            window.context.swap_buffers().unwrap();
        }
    }
}
