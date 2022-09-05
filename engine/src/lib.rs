mod camera;
mod controller;
mod renderer;
mod resources;
mod model;

use controller::{Controller, ControllerEvent};
use renderer::Renderer;
use winit::{
    dpi::PhysicalPosition,
    event::{DeviceEvent, Event, KeyboardInput, MouseScrollDelta, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub async fn run() {
    env_logger::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .build(&event_loop)
        .expect("Failed to create window");

    let mut renderer = Renderer::new(&window).await;

    let mut last_render_time = std::time::Instant::now();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if !renderer.input(&event) {
            match event {
                Event::DeviceEvent { event, .. } => match event {
                    DeviceEvent::MouseMotion { delta } => {
                        renderer.camera.input(ControllerEvent::MouseMove(delta))
                    }
                    _ => {}
                },
                Event::WindowEvent { window_id, event } if window_id == window.id() => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => renderer.resize(physical_size),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            renderer.resize(*new_inner_size)
                        }
                        WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state,
                                    virtual_keycode: Some(key),
                                    ..
                                },
                            ..
                        } => renderer
                            .camera
                            .input(ControllerEvent::KeyboardInput(state, key)),
                        WindowEvent::MouseInput { state, button, .. } => renderer
                            .camera
                            .input(ControllerEvent::MouseInput(state, button)),
                        WindowEvent::MouseWheel { delta, .. } => {
                            renderer
                                .camera
                                .input(ControllerEvent::MouseScroll(match delta {
                                    MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
                                    MouseScrollDelta::PixelDelta(PhysicalPosition {
                                        y: scroll,
                                        ..
                                    }) => scroll as f32,
                                }))
                        }
                        _ => {}
                    }
                }
                Event::RedrawRequested(window_id) if window_id == window.id() => {
                    let now = std::time::Instant::now();
                    let dt = now - last_render_time;
                    last_render_time = now;
                    renderer.update(dt);
                    match renderer.render() {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(renderer.size),
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => window.request_redraw(),
                _ => {}
            }
        }
    });
}
