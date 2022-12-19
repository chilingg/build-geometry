use crate::{
    app::State,
    data::prelude::*,
    scene_system::Scene,
    data::prelude::*,
};

use wgpu::util::DeviceExt;

const MSAA_SAMPLES: u32 = 4;
const DEFAULT_VIEW_SIZE: WorldSize = WorldSize::new(1000.0, 1000.0);

pub fn create_multisample_texture(state: &State, size: winit::dpi::PhysicalSize<u32>) -> wgpu::Texture {
    state.device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Game screen texture"),
        size: wgpu::Extent3d { width: size.width, height: size.height, depth_or_array_layers: 1 },
        mip_level_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: state.config.format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

        sample_count: MSAA_SAMPLES,
    })
}

pub fn create_multisample_texture_from_state(state: &State) -> wgpu::Texture {
    create_multisample_texture(state, state.size())
}

pub fn gen_view_data(state: &State) -> ViewData {
    let view_data = ViewData {
        center: WorldPoint::new(0.0, 0.0),
        size: ScreenSize::new(state.config.width as _, state.config.height as _),
        pixel_size: 1.0,
    };
    view_data
}

pub trait Renderer {
    fn sample_texture<'a>(&'a mut self) -> &'a mut wgpu::Texture;

    fn resize(&mut self, size: winit::dpi::PhysicalSize<u32>, state: &State) {
        *self.sample_texture() = create_multisample_texture(state, size);
    }

    fn update_view(&self, view_data: &ViewData, state: &State) {
        self.update_view_matrix(&ProjMatrix::look_to(view_data), state)
    }

    fn update_view_in_resize(&self, view_data: &mut ViewData, state: &State) {
        self.update_view_matrix(
            &ProjMatrix::look_to_range(view_data, DEFAULT_VIEW_SIZE),
            state
        )
    }

    fn update_view_matrix(&self, view_mat: &ProjMatrix, state: &State);

    fn update_scene(&mut self, scene: &Scene, pixel_size: f32, state: &State);

    fn init_in_scene(&mut self, scene: &Scene, pixel_size: f32, state: &State);

    fn render(&mut self, state: &State, view: &wgpu::TextureView);
}

use lyon::tessellation::{
    VertexBuffers,
    FillTessellator,
    FillOptions,
    FillVertex,
    StrokeTessellator,
    StrokeOptions,
    StrokeVertex,
    BuffersBuilder,
};

type DefaultVertexBuffers = VertexBuffers<[f32; 2], u16>;
type GraphMeshStack = Vec<(wgpu::Buffer, wgpu::Buffer, usize)>;

pub struct DefaultRenderer {
    render_pipeline: wgpu::RenderPipeline,
    sample_texture: wgpu::Texture,

    proj_buffer: wgpu::Buffer,
    color_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,

    graphs: GraphMeshStack,
    tips: GraphMeshStack,
}

impl Renderer for DefaultRenderer {
    fn sample_texture<'a>(&'a mut self) -> &'a mut wgpu::Texture {
        &mut self.sample_texture
    }

    fn update_view_matrix(&self, view_mat: &ProjMatrix, state: &State) {
        state.queue.write_buffer(
            &self.proj_buffer,
            0,
            bytemuck::cast_slice(&[view_mat.to_array()]),
        );
    }

    fn init_in_scene(&mut self, scene: &Scene, pixel_size: f32, state: &State) {
        let mut file_tessellator = FillTessellator::new();
        let mut file_options = FillOptions::default();
        file_options.tolerance *= pixel_size;

        let mut stroke_tessellator = StrokeTessellator::new();
        let mut stroke_options = StrokeOptions::default();
        stroke_options.tolerance *= pixel_size;
        stroke_options.line_width = 10.0;

        self.graphs = scene.graph.iter().map(|graph| {
            let mut output = DefaultVertexBuffers::new();
            match graph {
                GraphType::Circle { center, radius } => {
                    let mut builder = BuffersBuilder::new(
                        &mut output, |vertex: StrokeVertex| {
                            vertex.position().to_array()
                        }
                    );
            
                    stroke_tessellator.tessellate_circle(
                        center.cast_unit(), 
                        *radius, &stroke_options, 
                        &mut builder
                    ).expect("Failed tessellation graph stroke!");

                    let vertex_buffer = state.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Vertex Buffer"),
                            contents: bytemuck::cast_slice(&output.vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        }
                    );
                    
                    let index_buffer = state.device.create_buffer_init(
                        &wgpu::util::BufferInitDescriptor {
                            label: Some("Index Buffer"),
                            contents: bytemuck::cast_slice(&output.indices),
                            usage: wgpu::BufferUsages::INDEX,
                        }
                    );
            
                    (vertex_buffer, index_buffer, output.indices.len())
                }
                _ => panic!("undefine Graph!")
            }
        }).collect();
    }

    fn update_scene(&mut self, scene: &Scene, pixel_size: f32, state: &State) {
        ;
    }

    fn render(&mut self, state: &State, view: &wgpu::TextureView) {
        let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Game Render Encoder"),
        });
        let game_view = self.sample_texture.create_view(&wgpu::TextureViewDescriptor::default());

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &game_view,
                    resolve_target: Some(view),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.,
                            g: 0.,
                            b: 0.,
                            a: 1.,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            self.graphs.iter().for_each(|(vertex_buffer, index_buffer, num)| {
                state.queue.write_buffer(&self.color_buffer, 0, bytemuck::cast_slice(&[1.0f32, 0.0f32, 0.0f32, 1.0f32]));
                render_pass.set_bind_group(0, &self.bind_group, &[]);

                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..*num as u32, 0, 0..1);
            })
        }

        state.queue.submit(std::iter::once(encoder.finish()));
    }
}

impl DefaultRenderer {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        static ATTRIBS: [wgpu::VertexAttribute; 1]  = wgpu::vertex_attr_array![0 => Float32x2];
        // [
        //     wgpu::VertexAttribute {
        //         offset: 0,
        //         shader_location: 0,
        //         format: wgpu::VertexFormat::Float32x3,
        //     },
        //     wgpu::VertexAttribute {
        //         offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
        //         shader_location: 1,
        //         format: wgpu::VertexFormat::Float32x3,
        //     },
        // ]
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        }
    }
    
    pub fn new(state: &State) -> Self {
        Self::from_size(state, state.size())
    }

    fn from_size(state: &State, size: winit::dpi::PhysicalSize<u32>) -> Self {
        let proj_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection buffer"),
            size: std::mem::size_of::<[f32; 16]>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let color_buffer = state.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Color buffer"),
            size: std::mem::size_of::<[f32; 4]>() as u64,
            mapped_at_creation: false,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("bind_group_layout"),
        });
        let bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: color_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: proj_buffer.as_entire_binding(),
                },
            ],
            label: Some("bind_group"),
        });

        let render_pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let shader = state.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[DefaultRenderer::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: state.config.format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })]
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // 如果将该字段设置为除了 Fill 之外的任何值，都需要 Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // 需要 Features::DEPTH_CLIP_ENABLE
                unclipped_depth: false,
                // 需要 Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: MSAA_SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let sample_texture = create_multisample_texture(state, size);

        Self {
            render_pipeline,
            sample_texture, 

            color_buffer,
            proj_buffer,
            bind_group,
            graphs: Vec::new(),
            tips: Vec::new(),
        }
    }
}