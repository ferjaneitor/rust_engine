// src/main.rs

pub mod math;
pub mod graphics;

use graphics::window::Window; // nuestra abstracción de la ventana
use graphics::render::Renderer;
use graphics::scene_object::SceneObject;
use graphics::camara::Camera;

use math::{matrix_4_by_4::Matrix4, vec3::Vec3};

use glutin::event::{Event, MouseButton, ElementState, WindowEvent, DeviceEvent, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use std::time::Instant;

fn main() {
    // 1) Crear event loop
    let event_loop = EventLoop::new();

    // 2) Crear ventana y contexto OpenGL
    let window = Window::new("Rust_Engine", 1200, 900, &event_loop)
        .expect("No se pudo crear la ventana!");

    // 3) Crear un Renderer
    let renderer = Renderer::new("src/graphics/shaders/basic.vert", "src/graphics/shaders/basic.frag")
        .expect("No se pudo inicializar el renderer");

    // 4) Crear lista de objetos
    let mut objects: Vec<SceneObject> = Vec::new();

    // objeto 1
    let mut obj1 = SceneObject::create_object_from_stl("src/assets/pieza.stl");
    obj1.base_transform = Matrix4::translate(0.0, 0.0, 0.0);
    obj1.angle = 0.0;
    obj1.angular_speed = 1.0;
    obj1.scale_factor = 1.0;
    objects.push(obj1);

    // objeto 2
    let mut obj2 = SceneObject::create_object_from_stl("src/assets/pieza1.stl");
    obj2.base_transform = Matrix4::translate(-60.01, 0.01, 0.01);
    obj2.angle = 0.5;
    obj2.angular_speed = -2.0;
    obj2.scale_factor = 1.0;
    objects.push(obj2);

    // 5) Cámara
    let mut camera = Camera::new(Vec3::new(0.0, 0.0, 100.5));

    // 6) Estado de inputs
    let mut right_button_pressed = false;
    let mut scale_factor = 0.05;

    // Para delta_time
    let mut last_frame_time = Instant::now();

    // 7) Event loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            // input de mouse
            Event::DeviceEvent { event, .. } => {
                match event {
                    DeviceEvent::MouseMotion { delta: (dx, dy) } => {
                        if right_button_pressed {
                            camera.process_mouse(dx as f32, dy as f32);
                        }
                    }
                    _ => {}
                }
            }
            // input de ventana
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                }
                WindowEvent::MouseInput { button, state, .. } => {
                    if button == MouseButton::Right {
                        right_button_pressed = state == ElementState::Pressed;
                    }
                }
                WindowEvent::KeyboardInput { input, .. } => {
                    if let Some(key) = input.virtual_keycode {
                        if input.state == ElementState::Pressed {
                            if key == VirtualKeyCode::Escape {
                                *control_flow = ControlFlow::Exit;
                            } else {
                                let now = Instant::now();
                                let dt = (now - last_frame_time).as_secs_f32();
                                last_frame_time = now;

                                camera.process_keyboard(key, dt);

                                // Cambios de escala global
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
                    window.resize(new_size);
                }
                _ => {}
            },
            // Redibujar
            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let dt = (now - last_frame_time).as_secs_f32();
                last_frame_time = now;

                // Actualizar animación de cada objeto
                for obj in &mut objects {
                    obj.angle += obj.angular_speed * dt;
                }

                // Render
                renderer.render_scene(&window, &mut objects, &camera, scale_factor);
            }
            Event::MainEventsCleared => {
                // Forzar un redraw
                window.request_redraw();
            }
            _ => {}
        }
    });
}
