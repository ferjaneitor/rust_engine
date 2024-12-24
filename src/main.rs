use camara::Camera;
use gl;
use glutin::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use math::{math4::Math4, vec3::Vec3};

pub mod math;
pub mod camara;

use std::{ffi::CString, ptr, str, time::Instant};

// ----- Shaders (versión con color) -----
static VERTEX_SHADER_SRC: &str = r#"
#version 330 core

layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aColor;

out vec3 vColor;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

void main()
{
    vColor = aColor;
    gl_Position = projection * view * model * vec4(aPos, 1.0);
}
"#;

static FRAGMENT_SHADER_SRC: &str = r#"
#version 330 core

in vec3 vColor;
out vec4 FragColor;

void main()
{
    FragColor = vec4(vColor, 1.0);
}
"#;

// Pequeño helper para compilar shaders
fn compile_shader(src: &str, shader_type: gl::types::GLenum) -> Result<u32, String> {
    unsafe {
        let shader = gl::CreateShader(shader_type);
        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        // Comprobar errores
        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            gl::GetShaderInfoLog(shader, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
            let error = String::from_utf8_lossy(&buffer).to_string();
            return Err(error);
        }

        Ok(shader)
    }
}

fn link_program(vertex_shader: u32, fragment_shader: u32) -> Result<u32, String> {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as gl::types::GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as gl::types::GLint {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buffer = vec![0u8; len as usize];
            gl::GetProgramInfoLog(program, len, ptr::null_mut(), buffer.as_mut_ptr() as *mut i8);
            let error = String::from_utf8_lossy(&buffer).to_string();
            return Err(error);
        }

        // Podemos soltar los shaders enlazados
        gl::DetachShader(program, vertex_shader);
        gl::DetachShader(program, fragment_shader);

        Ok(program)
    }
}

// ---- Datos de nuestro cubo (pos + color) ----
static CUBE_VERTICES: [f32; 36 * 6] = [
    // FRONT (rojo)
    -0.5, -0.5,  0.5,  1.0, 0.0, 0.0, 
     0.5, -0.5,  0.5,  1.0, 0.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 0.0, 0.0,
    -0.5, -0.5,  0.5,  1.0, 0.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 0.0, 0.0,
    -0.5,  0.5,  0.5,  1.0, 0.0, 0.0,

    // BACK (verde)
    -0.5, -0.5, -0.5,  0.0, 1.0, 0.0,
     0.5,  0.5, -0.5,  0.0, 1.0, 0.0,
     0.5, -0.5, -0.5,  0.0, 1.0, 0.0,
    -0.5, -0.5, -0.5,  0.0, 1.0, 0.0,
    -0.5,  0.5, -0.5,  0.0, 1.0, 0.0,
     0.5,  0.5, -0.5,  0.0, 1.0, 0.0,

    // LEFT (azul)
    -0.5,  0.5,  0.5,  0.0, 0.0, 1.0,
    -0.5,  0.5, -0.5,  0.0, 0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 0.0, 1.0,
    -0.5, -0.5, -0.5,  0.0, 0.0, 1.0,
    -0.5, -0.5,  0.5,  0.0, 0.0, 1.0,
    -0.5,  0.5,  0.5,  0.0, 0.0, 1.0,

    // RIGHT (cian)
     0.5,  0.5,  0.5,  0.0, 1.0, 1.0,
     0.5, -0.5, -0.5,  0.0, 1.0, 1.0,
     0.5,  0.5, -0.5,  0.0, 1.0, 1.0,
     0.5, -0.5, -0.5,  0.0, 1.0, 1.0,
     0.5,  0.5,  0.5,  0.0, 1.0, 1.0,
     0.5, -0.5,  0.5,  0.0, 1.0, 1.0,

    // TOP (amarillo)
    -0.5,  0.5, -0.5,  1.0, 1.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 1.0, 0.0,
     0.5,  0.5, -0.5,  1.0, 1.0, 0.0,
     0.5,  0.5,  0.5,  1.0, 1.0, 0.0,
    -0.5,  0.5, -0.5,  1.0, 1.0, 0.0,
    -0.5,  0.5,  0.5,  1.0, 1.0, 0.0,

    // BOTTOM (magenta)
    -0.5, -0.5, -0.5,  1.0, 0.0, 1.0,
     0.5, -0.5, -0.5,  1.0, 0.0, 1.0,
     0.5, -0.5,  0.5,  1.0, 0.0, 1.0,
     0.5, -0.5,  0.5,  1.0, 0.0, 1.0,
    -0.5, -0.5,  0.5,  1.0, 0.0, 1.0,
    -0.5, -0.5, -0.5,  1.0, 0.0, 1.0,
];

// ---- Si tienes tus structs Mat4/Vec3, impórtalos aquí. ----
// Usa tu Mat4 con rotate_x(), rotate_y(), perspective() y look_at().
// Para simplificar, muestro "stub" de funciones de transformaciones aquí.

fn main() {
    // 1) EventLoop
    let event_loop = EventLoop::new();

    // 2) Ventana
    let wb = WindowBuilder::new()
        .with_title("Cubo con caras de distinto color")
        .with_inner_size(LogicalSize::new(1200, 900));

    // 3) Context
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();

    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    // 4) Cargar OpenGL
    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

    // 5) Config inicial
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.1, 0.2, 0.3, 1.0);
    }

    // 6) Compilar & linkear shaders
    let vs = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER).unwrap();
    let fs = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER).unwrap();
    let program = link_program(vs, fs).unwrap();

    // 7) Subir el cubo (36 vértices, cada uno con pos + color)
    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);

        // Subir datos
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (CUBE_VERTICES.len() * std::mem::size_of::<f32>()) as isize,
            CUBE_VERTICES.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // Atributo: posición (location=0), 3 floats
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * std::mem::size_of::<f32>()) as i32,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // Atributo: color (location=1), 3 floats
        let color_offset = 3 * std::mem::size_of::<f32>();
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            (6 * std::mem::size_of::<f32>()) as i32,
            color_offset as *const _,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    // Variables para animación
    let start_time = Instant::now();

    // Crea tu cámara
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 3.0));

    // Para medir delta_time
    let mut last_frame_time = std::time::Instant::now();

    // Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            // Capturar eventos de teclado/ratón
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        camera.process_mouse(dx as f32, dy as f32);
                    }
                    _ => {}
                }
            }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            // ESC para salir
                            if key == VirtualKeyCode::Escape {
                                *control_flow = ControlFlow::Exit;
                            } else {
                                // Mover la cámara con W, S, A, D
                                let now = std::time::Instant::now();
                                let delta_time = (now - last_frame_time).as_secs_f32();
                                last_frame_time = now;
                                camera.process_keyboard(key, delta_time);
                            }
                        }
                    }
                }
                WindowEvent::Resized(physical_size) => {
                    windowed_context.resize(physical_size);
                    unsafe {
                        gl::Viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                    }
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                // Calcular delta_time cada frame (otra forma)
                let now = std::time::Instant::now();
                let delta_time = (now - last_frame_time).as_secs_f32();
                last_frame_time = now;

                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                let elapsed = start_time.elapsed().as_secs_f32();

                // 1) Calcula transform 'model' girando el cubo
                //    (usa tus propias funciones rotate_x / rotate_y si tienes Mat4)
                let rot_y = Math4::rotate_y(elapsed);
                let rot_x = Math4::rotate_x(elapsed * 0.7);
                let model = Math4::multiply(&rot_y, &rot_x);

                // 2) Usar camera.get_view_matrix()
                let view = camera.get_view_matrix();

                // 3) Proyección en perspectiva
                let size = windowed_context.window().inner_size();
                let aspect = size.width as f32 / size.height as f32;
                let projection = Math4::perspective(45.0_f32.to_radians(), aspect, 0.1, 100.0);

                unsafe {
                    gl::UseProgram(program);

                    // Localiza uniform (model, view, projection)
                    let model_loc = gl::GetUniformLocation(program, b"model\0".as_ptr() as *const i8);
                    let view_loc = gl::GetUniformLocation(program, b"view\0".as_ptr() as *const i8);
                    let proj_loc = gl::GetUniformLocation(program, b"projection\0".as_ptr() as *const i8);

                    // Sube las matrices (aquí las convertimos en puntero [f32;16])
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
                    gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
                    gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());

                    // Dibujamos 36 vértices (6 caras × 2 triángulos × 3 vértices)
                    gl::BindVertexArray(vao);
                    gl::DrawArrays(gl::TRIANGLES, 0, 36);
                }

                windowed_context.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => {
                // Forzar un redraw
                windowed_context.window().request_redraw();
            }
            _ => {}
        }
    });
}
