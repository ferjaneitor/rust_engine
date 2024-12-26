use camara::Camera;
use gl;
use glutin::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use math::{matrix_4_by_4::Matrix4, vec3::Vec3};
use scene_object::SceneObject;

pub mod math;
pub mod camara;
pub mod scene_object;

use std::{
     ffi::CString, ptr, str, vec
};


// ---------------------------------------------------------------------------------
//  SHADERS (sin atributo de color, para dibujar el STL de un color fijo)
// ---------------------------------------------------------------------------------
static VERTEX_SHADER_SRC: &str = r#"
#version 330 core
layout(location = 0) in vec3 aPos;
layout(location = 1) in vec3 aNormal;

uniform mat4 model;
uniform mat4 view;
uniform mat4 projection;

out vec3 vNormal;
out vec3 vWorldPos;

void main()
{
    // Transformar la posición
    vec4 worldPos = model * vec4(aPos, 1.0);
    vWorldPos = worldPos.xyz;

    // Normal Matrix
    mat3 normalMat = mat3(transpose(inverse(model)));
    vNormal = normalize(normalMat * aNormal);

    gl_Position = projection * view * worldPos;
}
"#;

static FRAGMENT_SHADER_SRC: &str = r#"
#version 330 core

in vec3 vNormal;    // Viene del vertex shader
in vec3 vWorldPos;  // no lo usamos mucho ahora, pero podría servir

out vec4 FragColor;

// Uniforms para una luz direccional sencilla
uniform vec3 lightDir;   // dirección de la luz
uniform vec3 lightColor; // color de la luz
uniform vec3 objectColor; // color base del objeto

void main()
{
    // 1) Normalizar la normal
    vec3 N = normalize(vNormal);
    // 2) Direccion de la luz
    //    Si 'lightDir' apunta DESDE el objeto hacia la luz, pon L = -lightDir, o viceversa.
    vec3 L = normalize(lightDir);

    // 3) Difuso (Lambert)
    float diff = max(dot(N, L), 0.0);

    // 4) Color difuso
    vec3 diffuse = diff * lightColor * objectColor;

    // 5) Pequeña componente ambiental
    vec3 ambient = 0.1 * objectColor;

    // 6) Sumar y escribir
    vec3 finalColor = ambient + diffuse;
    FragColor = vec4(finalColor, 1.0);
}
"#;


/// Compila un shader (vertex o fragment) desde el `src`.
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

/// Enlaza un vertex y fragment shader en un programa.
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

        // Liberar shaders
        gl::DetachShader(program, vertex_shader);
        gl::DetachShader(program, fragment_shader);

        Ok(program)
    }
}

// ---------------------------------------------------------------------------------
//  MAIN
// ---------------------------------------------------------------------------------
fn main() {
    // 1) Crear EventLoop
    let event_loop = EventLoop::new();

    // 2) Construir la ventana (1200x900)
    let wb = WindowBuilder::new()
        .with_title("Rust_Engine")
        .with_inner_size(LogicalSize::new(1200, 900));

    // 3) Context
    let windowed_context = ContextBuilder::new()
        .with_vsync(true)
        .build_windowed(wb, &event_loop)
        .unwrap();

    // Activar el contexto
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    // 4) Cargar funciones de OpenGL
    gl::load_with(|s| windowed_context.get_proc_address(s) as *const _);

    // 5) Config inicial de OpenGL
    unsafe {
        gl::Enable(gl::DEPTH_TEST);
        gl::ClearColor(0.1, 0.2, 0.3, 1.0);
    }

    // 6) Compilar & linkear shaders
    let vs = compile_shader(VERTEX_SHADER_SRC, gl::VERTEX_SHADER).unwrap();
    let fs = compile_shader(FRAGMENT_SHADER_SRC, gl::FRAGMENT_SHADER).unwrap();
    let program = link_program(vs, fs).unwrap();

    // ----------------------------------------------------------------------------
    // 7) CARGAR tu modelo STL
    // ----------------------------------------------------------------------------
    // Lista de objetos
    let mut objects: Vec<SceneObject> = Vec::new();

    // crear el 1er objeto
    let mut obj1 = SceneObject::create_object_from_stl("src/assets/pieza.stl");
    obj1.base_transform = Matrix4::translate(0.0, 0.0, 0.0);
    obj1.angle = 0.0;
    obj1.angular_speed = 1.0;
    obj1.scale_factor = 1.0 ;
    objects.push(obj1);

    let mut obj2 = SceneObject::create_object_from_stl("src/assets/pieza1.stl");
    obj2.base_transform = Matrix4::translate(-60.01, 0.01, 0.01);
    obj2.angle = 0.5 ;
    obj2.angular_speed = -2.0 ;
    obj2.scale_factor = 1.0 ;
    objects.push(obj2);
    
    // Ajusta la ruta según dónde tengas tu "pieza.stl"

    // 9) Crear tu cámara
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 100.5));
    // Con z=5 para alejarse un poco más si la pieza es grande

    // Para medir delta_time
    let mut last_frame_time = std::time::Instant::now();

    let mut scale_factor = 0.05;

    let mut right_button_pressed = false;

    // ----------------------------------------------------------------------------
    // 10) Loop de eventos
    // ----------------------------------------------------------------------------
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            // Capturar eventos de teclado/ratón a nivel de Device
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        // Rotación de la cámara con el mouse
                        
                        if right_button_pressed {
                            camera.process_mouse(dx as f32, dy as f32);
                        }
                    }
                    _ => {}
                }
            }
            // Eventos de ventana
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Right {
                        if state == ElementState::Pressed {
                            right_button_pressed = true;
                        } else {
                            right_button_pressed = false;
                        }
                    }
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
                                
                                // Q/E para escala
                                match key {
                                    VirtualKeyCode::Q => {
                                        scale_factor *= 1.1;
                                    }
                                    VirtualKeyCode::E => {
                                        scale_factor *= 0.9;
                                    }
                                    _ => {}
                                }
                                
                            }
                        }
                    }
                }
                WindowEvent::Resized(new_size) => {
                    windowed_context.resize(new_size);
                    unsafe {
                        gl::Viewport(0, 0, new_size.width as i32, new_size.height as i32);
                    }
                }
                _ => {}
            },
            // Redibujar
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = (now - last_frame_time).as_secs_f32();
                last_frame_time = now;

                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                // Actualizar animacion de cada objeto
                for obj in &mut objects {
                    obj.angle += obj.angular_speed * dt;
                }

                let view = camera.get_view_matrix();

                let size = windowed_context.window().inner_size();
                let aspect = size.width as f32 / size.height as f32;
                let projection = Matrix4::perspective(45.0_f32.to_radians(), aspect, 0.01, 1000.0);

                unsafe {
                    // Activar el shader
                    gl::UseProgram(program);

                    // Luz direccional
                    let light_dir_loc = gl::GetUniformLocation(program, b"lightDir\0".as_ptr() as *const i8);
                    let light_color_loc = gl::GetUniformLocation(program, b"lightColor\0".as_ptr() as *const i8);
                    let object_color_loc = gl::GetUniformLocation(program, b"objectColor\0".as_ptr() as *const i8);
                    gl::Uniform3f(light_dir_loc, 1.0, 1.0, 1.0);
                    gl::Uniform3f(light_color_loc, 1.0, 1.0, 1.0);
                    gl::Uniform3f(object_color_loc, 0.8, 0.8, 0.8);

                    // Ubicar las locations de las matrices
                    let model_loc = gl::GetUniformLocation(program, b"model\0".as_ptr() as *const i8);
                    let view_loc  = gl::GetUniformLocation(program, b"view\0".as_ptr() as *const i8);
                    let proj_loc  = gl::GetUniformLocation(program, b"projection\0".as_ptr() as *const i8);

                    // Subir view y projection (iguales para todos)
                    gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
                    gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());

                    // Dibujar cada objeto
                    for obj in &objects {
                        // (1) Rota en Y con obj.angle
                        let rot_mat = Matrix4::rotate_y(obj.angle);
                        // (2) Aplica escala global (si quieres) => scale_factor
                        let scale_mat = Matrix4::scale(scale_factor); 
                        let local_anim = Matrix4::multiply(&scale_mat, &rot_mat);
                        // (3) Combinar con la base
                        let final_model = Matrix4::multiply(&local_anim, &obj.base_transform);

                        // Subir final_model al shader
                        gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, final_model.as_ptr());

                        // Bindear VAO
                        gl::BindVertexArray(obj.vao);

                        // Dibujar
                        gl::DrawElements(
                            gl::TRIANGLES,
                            obj.index_count,
                            gl::UNSIGNED_INT,
                            std::ptr::null(),
                        );
                    }

                    windowed_context.swap_buffers().unwrap();
                }
            }

            Event::MainEventsCleared => {
                // Forzar un nuevo frame
                windowed_context.window().request_redraw();
            }
            _ => {}
        }
    });
}
