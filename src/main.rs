use camara::Camera;
use gl;
use glutin::{
    dpi::LogicalSize,
    event::{DeviceEvent, ElementState, Event, MouseButton, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use math::{matrix_4_by_4::Matrix4, vec3::Vec3, float3_eps::Float3Eps};

pub mod math;
pub mod camara;

use std::{
    ffi::CString,
    fs::File,
    ptr,
    str,
    time::Instant,
    collections::HashMap,
};

use stl_io::{self};

/// Estructura para acumular datos de cada vértice
/// - pos: posición (x, y, z)
/// - normal: normal acumulada (nx, ny, nz)
#[derive(Debug)]
struct VertexData {
    pos: [f32; 3],
    normal: [f32; 3],
}


/// Carga un STL y calcula normales "smooth" promediadas.
/// Devuelve (positions, normals, indices).
/// - `positions`: [x0, y0, z0, x1, y1, z1, ...]
/// - `normals`:   [nx0, ny0, nz0, nx1, ny1, nz1, ...]
/// - `indices`:   [i0, i1, i2, ...] (u32)
pub fn load_stl_model_smooth(path: &str) -> (Vec<f32>, Vec<f32>, Vec<u32>) {
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
        .with_title("Visualizador STL en Rust")
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
    let (stl_vertices, stl_normals, stl_indices) = load_stl_model_smooth("src/assets/pieza.stl");
    // Ajusta la ruta según dónde tengas tu "pieza.stl"

    // Generar VAO, VBO pos, VBO normal, EBO
    let mut vao = 0;
    let mut vbo_pos = 0;
    let mut vbo_nor = 0;
    let mut ebo = 0;
    unsafe {
        // 1) Generar IDs
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo_pos);
        gl::GenBuffers(1, &mut vbo_nor);
        gl::GenBuffers(1, &mut ebo);

        // 2) Enlazar VAO
        gl::BindVertexArray(vao);

        // --- Subir posiciones (location = 0) ---
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_pos);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (stl_vertices.len() * std::mem::size_of::<f32>()) as isize,
            stl_vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        // decimos: (location=0), 3 floats, stride=0 (array comprimido)
        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            0,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        // --- Subir normales (location = 1) ---
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo_nor);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            (stl_normals.len() * std::mem::size_of::<f32>()) as isize,
            stl_normals.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );
        // decimos: (location=1), 3 floats, stride=0
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            0,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(1);

        // --- Subir índices (EBO) ---
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (stl_indices.len() * std::mem::size_of::<u32>()) as isize,
            stl_indices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        // 3) Desenlazar buffers
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    // 8) Variables para animación
    let start_time = Instant::now();

    // 9) Crear tu cámara
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 5.0));
    // Con z=5 para alejarse un poco más si la pieza es grande

    // Para medir delta_time
    let mut last_frame_time = std::time::Instant::now();

    let mut scale_factor = 1.0;

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
                last_frame_time = now;

                unsafe {
                    gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
                }

                // Rotaciones
                let elapsed = start_time.elapsed().as_secs_f32();
                let rot_y = Matrix4::rotate_y(elapsed);
                let rot_x = Matrix4::rotate_x(elapsed * 0.5);
                let rotate_mat = Matrix4::multiply(&rot_y, &rot_x);

                // Escala
                let scale_mat = Matrix4::scale(scale_factor);
                // model = scale * rotate
                let model = Matrix4::multiply(&scale_mat, &rotate_mat);

                let view = camera.get_view_matrix();

                let size = windowed_context.window().inner_size();
                let aspect = size.width as f32 / size.height as f32;
                let projection = Matrix4::perspective(45.0_f32.to_radians(), aspect, 0.1, 100.0);

                unsafe {
                    // Activar el shader
                    gl::UseProgram(program);

                    // Ubicaciones
                    let light_dir_loc = gl::GetUniformLocation(program, b"lightDir\0".as_ptr() as *const i8);
                    let light_color_loc = gl::GetUniformLocation(program, b"lightColor\0".as_ptr() as *const i8);
                    let object_color_loc = gl::GetUniformLocation(program, b"objectColor\0".as_ptr() as *const i8);

                    // Ejemplo: luz direccional desde arriba/derecha
                    // Si quieres que la luz vaya desde (1,1,1) hacia el origen, define "lightDir = (1,1,1)" o su normalizado
                    // O ponle la dirección inversa según tu convención.
                    gl::Uniform3f(light_dir_loc, 1.0, 1.0, 1.0);

                    // Color de la luz (blanca)
                    gl::Uniform3f(light_color_loc, 1.0, 1.0, 1.0);

                    // Color del objeto (gris claro)
                    gl::Uniform3f(object_color_loc, 0.8, 0.8, 0.8);

                    // Localizar uniforms
                    let model_loc = gl::GetUniformLocation(program, b"model\0".as_ptr() as *const i8);
                    let view_loc  = gl::GetUniformLocation(program, b"view\0".as_ptr() as *const i8);
                    let proj_loc  = gl::GetUniformLocation(program, b"projection\0".as_ptr() as *const i8);

                    // Subir las matrices
                    gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.as_ptr());
                    gl::UniformMatrix4fv(view_loc, 1, gl::FALSE, view.as_ptr());
                    gl::UniformMatrix4fv(proj_loc, 1, gl::FALSE, projection.as_ptr());

                    // Dibujar la malla STL
                    gl::BindVertexArray(vao);
                    gl::DrawElements(gl::TRIANGLES, stl_indices.len() as i32, gl::UNSIGNED_INT, std::ptr::null());
                }

                // Intercambiar buffers
                windowed_context.swap_buffers().unwrap();
            }
            Event::MainEventsCleared => {
                // Forzar un nuevo frame
                windowed_context.window().request_redraw();
            }
            _ => {}
        }
    });
}
