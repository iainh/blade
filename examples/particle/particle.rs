use std::mem;

struct Particle {
    pos: [f32; 2],
    pos_vel: [f32; 2],
    rot: [f32; 2],
    rot_vel: [f32; 2],
    scale: f32,
    scale_vel: f32,
    color: u32,
    age: f32,
}

pub struct System {
    capacity: usize,
    particle_buf: blade::Buffer,
    emit_pipeline: blade::ComputePipeline,
    update_pipeline: blade::ComputePipeline,
    draw_pipeline: blade::RenderPipeline,
}

pub struct SystemDesc<'a> {
    pub name: &'a str,
    pub capacity: usize,
}

impl System {
    pub fn new(context: &blade::Context, desc: SystemDesc) -> Self {
        let particle_buf = context.create_buffer(blade::BufferDesc {
            name: desc.name,
            size: (desc.capacity * mem::size_of::<Particle>()) as u64,
            memory: blade::Memory::Device,
        });
        let source = std::fs::read_to_string("examples/particle/particle.wgsl").unwrap();
        let shader = context.create_shader(blade::ShaderDesc { source: &source });
        let emit_pipeline = context.create_compute_pipeline(blade::ComputePipelineDesc {
            name: &format!("{} - emit", desc.name),
            data_layouts: &[],
            compute: shader.at("emit"),
        });
        let update_pipeline = context.create_compute_pipeline(blade::ComputePipelineDesc {
            name: &format!("{} - update", desc.name),
            data_layouts: &[],
            compute: shader.at("update"),
        });
        let draw_pipeline = context.create_render_pipeline(blade::RenderPipelineDesc {
            name: &format!("{} - draw", desc.name),
            data_layouts: &[],
            primitive: blade::PrimitiveState {
                topology: blade::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            vertex: shader.at("draw_vs"),
            fragment: shader.at("draw_fs"),
            color_targets: &[],
            depth_stencil: None,
        });
        Self {
            capacity: desc.capacity,
            particle_buf,
            emit_pipeline,
            update_pipeline,
            draw_pipeline,
        }
    }

    pub fn reset(&mut self, encoder: &mut blade::TransferCommandEncoder) {
        encoder.fill_buffer(
            self.particle_buf.into(),
            (self.capacity * mem::size_of::<Particle>()) as u64,
            0,
        );
    }

    pub fn update(&mut self, encoder: &mut blade::CommandEncoder) {
        let mut pass = encoder.compute();
        let mut pc = pass.with(&self.update_pipeline);
        let group_size = self.update_pipeline.get_workgroup_size();
        let group_count = (self.capacity as u32 + group_size[0] - 1) / group_size[0];
        pc.dispatch([group_count, 1, 1]);
    }

    pub fn delete(self, context: &blade::Context) {
        context.destroy_buffer(self.particle_buf);
    }
}