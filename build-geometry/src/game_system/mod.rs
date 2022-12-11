use wgpu::util::DeviceExt;

use winit::{
    event::*,
    event_loop::{ ControlFlow, EventLoop },
    window::{ Window, WindowBuilder },
};

use backend::app::{ System, State };

type BezierSegment = lyon_geom::CubicBezierSegment<f32>;
type Point = lyon_geom::Point<f32>;
type Size = lyon_geom::Size<f32>;
type Vector = lyon_geom::Vector<f32>;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct BezierUniform {
    p0: [f32; 2],
    p1: [f32; 2],
    p2: [f32; 2],
    p3: [f32; 2],
    segment_size: u32,
    pading: u32,
}

impl BezierUniform {
    fn update_from_bezier_segment(&mut self, segment: &BezierSegment, tolerance: u32, pixel_ratio: f32) {
        self.p0 = segment.from.into();
        self.p1 = segment.ctrl1.into();
        self.p2 = segment.ctrl2.into();
        self.p3 = segment.to.into();
        self.segment_size = (segment.approximate_length(tolerance as f32) * pixel_ratio).round() as u32;
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MatrixUniform {
    view_proj: [[f32; 4]; 4],
}

impl MatrixUniform {
    pub fn update(&mut self, center: Point, size: Size, pixel_ratio: f32) {
        self.view_proj = [
            [1.0/size.width, 0.0, 0.0, center.x],
            [0.0, 1.0/size.height, 0.0, center.y],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }
}

impl Default for MatrixUniform {
    fn default() -> Self {
        Self {
            view_proj :[
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

pub struct GameSystemStruct {
    render_pipeline: wgpu::RenderPipeline,

    view_uniform: MatrixUniform,
    view_bind_group: wgpu::BindGroup,
    center: Point,
    pixel_ratio: f32,

    bezier_uniform: BezierUniform,
    bezier_bind_group: wgpu::BindGroup,
    bezier_segment: BezierSegment,
    tolerance: u32,
}

pub struct GameSystem {
    data: Option<GameSystemStruct>,
}

impl GameSystem {
    pub fn new() -> Self {
        Self {
            data: None
        }
    }
}

impl System for GameSystem {
    fn start(&mut self, state: &State) {
        let center = Point::new(0.0, 0.0);
        let pixel_ratio = 1.0;
        let tolerance = 10;
    
        let bezier_segment = BezierSegment {
            from: Point::new(-400.0, -400.0),
            ctrl1: Point::new(-400.0, -400.0),
            ctrl2: Point::new(400.0, -400.0),
            to: Point::new(400.0, 400.0),
        };
        let mut bezier_uniform = BezierUniform::default();
        bezier_uniform.update_from_bezier_segment(&bezier_segment, tolerance, pixel_ratio);
        let bezier_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bezier buffer"),
            contents: bytemuck::cast_slice(&[bezier_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let bezier_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("bezier_bind_group_layout"),
        });
        let bezier_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &bezier_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: bezier_buffer.as_entire_binding(),
                },
            ],
            label: Some("bezier_bind_group"),
        });
    
        let mut view_uniform = MatrixUniform::default();
        view_uniform.update(center, Size::new(state.config.width as _, state.config.height as _), pixel_ratio);
        let view_buffer = state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("View Buffer"),
                contents: bytemuck::cast_slice(&[view_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );
        let view_bind_group_layout = state.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            label: Some("view_bind_group_layout"),
        });
        let view_bind_group = state.device.create_bind_group(&wgpu::BindGroupDescriptor{
            layout: &view_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: view_buffer.as_entire_binding(),
                },
            ],
            label: Some("view_bind_group"),
        });
    
        let shader = state.device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout = state.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: &[
                &bezier_bind_group_layout,
                &view_bind_group_layout
            ],
            push_constant_ranges: &[],
        });
    
        let render_pipeline = state.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
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
                topology: wgpu::PrimitiveTopology::LineStrip,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                // 如果将该字段设置为除了 Fill 之外的任何值，都需要 Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // 需要 Features::DEPTH_CLIP_ENABLE
                unclipped_depth: false,
                // 需要 Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        self.data = Some(GameSystemStruct {
            render_pipeline,
        
            view_uniform,
            view_bind_group,
            center,
            pixel_ratio,
        
            bezier_uniform,
            bezier_bind_group,
            bezier_segment,
            tolerance,
        })
    }

    fn finish(&mut self) {
        self.data = None;
    }

    fn render(&mut self, state: &State, view: &wgpu::TextureView) {
        if let Some(ref game) = self.data {
            let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Game Render Encoder"),
            });
    
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Game Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: true,
                        },
                    })],
                    depth_stencil_attachment: None,
                });
    
                render_pass.set_pipeline(&game.render_pipeline);
                render_pass.set_bind_group(0, &game.bezier_bind_group, &[]);
                render_pass.set_bind_group(1, &game.view_bind_group, &[]);
                render_pass.draw(0..game.bezier_uniform.segment_size + 1, 0..1);
            }

            state.queue.submit(std::iter::once(encoder.finish()));
        }
    }
}