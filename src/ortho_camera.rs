use wgpu::util::DeviceExt;

pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct OrthoCamera{
    x:f32,
    y:f32,
    width:f32,
    height:f32,
    pub bind_group:wgpu::BindGroup,
    pub bind_group_layout:wgpu::BindGroupLayout, 
    pub buffer:wgpu::Buffer,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view: [[f32; 4]; 4],
}

impl OrthoCamera {
    pub fn set_size(&mut self, queue:&wgpu::Queue, width:f32, height:f32){
        self.width = width;
        self.height = height;
        let view = OPENGL_TO_WGPU_MATRIX * cgmath::ortho(self.x, self.x+self.width, self.y+self.height, self.y, -1.0, 1.0);
        let camera_uniform = CameraUniform { view:view.into() };
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[camera_uniform]));
    }

    pub fn new(device:&wgpu::Device, x:f32, y:f32, width:f32, height:f32) -> Self{
        let view = (OPENGL_TO_WGPU_MATRIX * cgmath::ortho(x, x+width, y+height, y, -1.0, 1.0)).into();
        let camera_uniform = CameraUniform { view };

        let buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });
        
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        OrthoCamera { x, y, width, height, bind_group, bind_group_layout, buffer}
    }
}
 