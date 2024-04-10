mod render_pipeline;
mod mesh;
mod ortho_camera;
mod vector_graphics;
use vector_graphics::VectorGraphics;

use crate::render_pipeline::*;

pub fn main() {
    env_logger::init();
    let event_loop = winit::event_loop::EventLoop::new().unwrap();
    let (x,y,width,height) = render_pipeline::get_window_rect();
    let window = winit::window::WindowBuilder::new()
        .with_position(winit::dpi::Position::Logical(winit::dpi::LogicalPosition{x, y}))
        .with_inner_size(winit::dpi::Size::Logical(winit::dpi::LogicalSize{width, height}))
        .build(&event_loop)
        .unwrap();

    let size = window.inner_size();
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
        backends: wgpu::Backends::GL,
        ..Default::default()
    });
    let surface = instance.create_surface(&window).unwrap();
    let adapter = futures::executor::block_on(instance.request_adapter(
        &wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        },
    )).unwrap();
    let (device, queue) = futures::executor::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            label: None,
        },
        None,
    )).unwrap();
    let surface_caps = surface.get_capabilities(&adapter);
    let surface_format = surface_caps.formats.iter()
        .copied()
        .filter(|f| f.is_srgb())
        .next()
        .unwrap_or(surface_caps.formats[0]);
    let mut config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: size.width,
        height: size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency:1,
    };
    surface.configure(&device, &config);
    let mut render_pipeline = RenderPipeline::new(&device, &config);
    let mut vector_graphics = VectorGraphics::new();

    event_loop.run( |event, target|{
        match event {
            winit::event::Event::WindowEvent { window_id:_window_id, event } => {
                match event {
                    winit::event::WindowEvent::CursorMoved { device_id:_device_id, position } => {
                        vector_graphics.mousemove(vector_graphics::Vector2 { x: position.x as f32, y: position.y as f32 });
                    }
                    winit::event::WindowEvent::KeyboardInput { device_id:_device_id, event, is_synthetic:_is_synthetic } =>{
                        match event.physical_key{
                            winit::keyboard::PhysicalKey::Code(code) => {
                                match event.state{
                                    winit::event::ElementState::Pressed => vector_graphics.keydown(code),
                                    winit::event::ElementState::Released => vector_graphics.keyup(code),
                                };
                            }
                            _=>{},
                        }
                    }
                    winit::event::WindowEvent::RedrawRequested => {
                        vector_graphics.update(&mut render_pipeline.mesh, &queue);
                        render_pipeline.render(&surface, &device, &queue);
                        window.request_redraw();
                    }
                    winit::event::WindowEvent::Resized(new_size) =>{
                        config.width = new_size.width;
                        config.height = new_size.height;
                        surface.configure(&device, &config);
                        vector_graphics.resize(new_size.width as f32, new_size.height as f32);
                        render_pipeline.resize(&queue, new_size.width as f32, new_size.height as f32);
                    }
                    winit::event::WindowEvent::CloseRequested => {
                        target.exit();
                    }
                    _=>{}
                }
            }
            _ => {}
        }
    }).unwrap();
}