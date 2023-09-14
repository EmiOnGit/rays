use wgpu::{Device, BindGroup, TextureView, BufferBinding, BufferUsages, BufferDescriptor, Buffer};

use crate::globals::{self, Globals};

pub struct ComputePipeline {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub bind_group: Option<BindGroup>,
    pub globals_buffer: Buffer,
}

impl ComputePipeline {
    pub fn new(device: &wgpu::Device) -> Self {
        // Load shader
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Compute Shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../../assets/shaders/compute.wgsl").into(),
            ),
        });

        // Bind Groups
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Compute"),
            entries: &[
                // Globals
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        min_binding_size: None,
                        has_dynamic_offset: false,
                    },
                    count: None,
                },
                // Output image
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture { 
                        access: wgpu::StorageTextureAccess::ReadWrite, 
                        format: wgpu::TextureFormat::Rgba32Float,
                        view_dimension:wgpu::TextureViewDimension::D2, 
                    },
                    count: None,
                },
                
                
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "main",
        });
        let globals_buffer = device.create_buffer(&BufferDescriptor { 
            label: "Globals buffer".into(), 
            size: std::mem::size_of::<Globals>() as u64, 
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST, 
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            bind_group_layout,
            bind_group: None,
            globals_buffer,
        }
    }
    pub fn prepare_bind_group(&mut self, device: &Device, output_texture_view: TextureView, globals: &Globals) {
        // self.bind_group = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
        //     label: Some("Render bind group"),
        //     layout: &self.bind_group_layout,
        //     entries: &[
        //         wgpu::BindGroupEntry {
        //             binding: 0,
        //             resource: wgpu::BindingResource::Buffer(BufferBinding {
        //                 buffer: globals.,
        //                 offset: 0,
        //                 size: None,
        //             }),
        //         },
        //         wgpu::BindGroupEntry {
        //             binding: 1,
        //             resource: wgpu::BindingResource::TextureView(&output_texture_view),
        //         }
        //     ],
        // }));
    }
}