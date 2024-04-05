use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    color: [f32; 3],
}

struct FixedSizeVec<T> where T:Copy{
    data:Vec<T>,
    length:usize,
}

impl<T> FixedSizeVec<T> where T:Copy{
    pub fn new(default:T, capacity:usize) -> Self{
        FixedSizeVec{ data:vec![default;capacity], length:0}
    }

    pub fn push(&mut self, item:T){
        self.data[self.length] = item;
        self.length+=1;
    }

    pub fn clear(&mut self){
        self.length = 0;
    }
}

pub struct Mesh{
    vertices:FixedSizeVec<Vertex>,
    indices:FixedSizeVec<u16>,
    pub vertex_buffer:wgpu::Buffer,
    pub index_buffer:wgpu::Buffer,
    pub num_indices:u32,
}

impl Mesh{
    pub fn new(device:&wgpu::Device)->Self{
        let vertices:FixedSizeVec<Vertex> = FixedSizeVec::new(Vertex { position: [0.0,0.0], color: [0.0,0.0,0.0] }, 2000);
        let indices:FixedSizeVec<u16> = FixedSizeVec::new(0, 2000);

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices.data),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices.data),
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        let num_indices = indices.length as u32;
        Mesh{vertices, indices, vertex_buffer, index_buffer, num_indices}
    }

    pub fn get_vertex_buffer_layout(&self)->wgpu::VertexBufferLayout{
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex, 
            attributes: &[ 
                wgpu::VertexAttribute {
                    offset: 0, 
                    shader_location: 0, 
                    format: wgpu::VertexFormat::Float32x3, 
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                }
            ]
        }
    }

    pub fn add_rect(&mut self, x:f32, y:f32, width:f32, height:f32, r:f32, g:f32, b:f32){
        let vertex_id = self.vertices.length as u16;
        self.vertices.push(Vertex { position: [x,y], color: [r,g,b] });
        self.vertices.push(Vertex { position: [x+width,y], color: [r,g,b] });
        self.vertices.push(Vertex { position: [x+width,y+height], color: [r,g,b] });
        self.vertices.push(Vertex { position: [x,y+height], color: [r,g,b] });

        self.indices.push(vertex_id);
        self.indices.push(vertex_id+2);
        self.indices.push(vertex_id+1);
        self.indices.push(vertex_id);
        self.indices.push(vertex_id+3);
        self.indices.push(vertex_id+2);
    }

    pub fn update_queue(&mut self, queue:&wgpu::Queue){
        queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&self.vertices.data));
        queue.write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&self.indices.data));
        self.num_indices = self.indices.length as u32;
        self.vertices.clear();
        self.indices.clear();
    }
}