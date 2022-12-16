use wgpu::util::DeviceExt;

use winit::event::*;

use backend::app::{ System, State };

type BezierSegment = lyon_geom::CubicBezierSegment<f32>;
type Point = lyon_geom::Point<f32>;
type Size = lyon_geom::Size<f32>;
type Vector = lyon_geom::Vector<f32>;

pub struct DirtyFlag<T> {
    is_dirty: bool,
    data: T,
}

impl<T> DirtyFlag<T> {
    pub fn new(data: T) -> Self {
        Self { is_dirty: false, data }
    }

    pub fn read(&self) -> &T {
        if self.is_dirty {
            panic!("Read dirtied data!");
        }

        &self.data
    }

    pub fn unchecked_read(&self) -> &T {
        &self.data
    }

    pub fn write(&mut self) -> &mut T {
        self.is_dirty = true;
        &mut self.data
    }

    pub fn get_all(&mut self) -> (&mut T, &mut bool) {
        (&mut self.data, &mut self.is_dirty)
    }

    pub fn is_dirty(&mut self) -> bool {
        self.is_dirty
    }

    pub fn clean_flag(&mut self) {
        self.is_dirty = false;
    }

    pub fn set_dirty(&mut self) {
        self.is_dirty = true;
    }
}

pub struct BezierData {
    pub segment: BezierSegment,
    pub stroke_width: f32,
    pub subdivide: u32,
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BezierUniform {
    pub p0: [f32; 2],
    pub p1: [f32; 2],
    pub p2: [f32; 2],
    pub p3: [f32; 2],
    pub subdivide: u32,
    pub stroke_width: f32,
}

impl BezierUniform {
    // 每1/RATIO个像素长度为一渲染段
    pub const PER_PIXELS: f32 = 10.0;
    pub const TOLERANCE: f32 = 1.0;

    fn subdivide_size(bezier_segment: &BezierSegment) -> u32 {
        (bezier_segment.approximate_length(Self::TOLERANCE) / Self::PER_PIXELS).max(1.0) as u32
    }

    fn from(bezier_data: &BezierData) -> Self {
        Self {
            p0: bezier_data.segment.from.into(),
            p1: bezier_data.segment.ctrl1.into(),
            p2: bezier_data.segment.ctrl2.into(),
            p3: bezier_data.segment.to.into(),
            subdivide: bezier_data.subdivide,
            stroke_width: bezier_data.stroke_width.abs(),
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct MatrixUniform {
    view_proj: [[f32; 4]; 4],
}

impl MatrixUniform {
    pub fn from_view(view_data: &ViewData) -> Self {
        let c0r0 = 2.0 / view_data.size.width / view_data.pixel_size;
        let c1r1 = 2.0 / view_data.size.height / view_data.pixel_size;
        let c3r0 = -view_data.center.x * c0r0;
        let c3r1 = -view_data.center.y * c1r1;

        MatrixUniform {
            view_proj: [
                [c0r0, 0.0, 0.0, 0.0],
                [0.0, c1r1, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [c3r0, c3r1, 0.0, 1.0],
            ]
        }
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

pub struct ViewData {
    center: Point,
    pixel_size: f32,
    size: Size,
}

pub struct InputState {
    pub left: ElementState,
    pub right: ElementState,
    pub middle: ElementState,
}

#[derive(Default)]
pub struct CursorState {
    pub pos: Point,
    pub moved: Option<Vector>,
}

pub struct GameSystemStruct {
    render_pipeline: wgpu::RenderPipeline,
    pub screen_texture: DirtyFlag<wgpu::Texture>,

    view_bind_group: wgpu::BindGroup,
    view_bind_buffer: wgpu::Buffer,
    pub view_data: DirtyFlag<ViewData>,

    bezier_bind_group: wgpu::BindGroup,
    bezier_buffer: wgpu::Buffer,
    pub bezier_data: DirtyFlag<BezierData>,

    input_state: InputState,
    cursor_state: CursorState,
}

impl GameSystemStruct {
    pub fn create_screen_texture(state: &State) -> wgpu::Texture {
        state.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game screen texture"),
            size: wgpu::Extent3d { width: state.config.width, height: state.config.height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: state.config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,

            sample_count: GameSystem::MSAA_SAMPLES,
        })
    }

    pub fn point_from_window(&self, point: &winit::dpi::PhysicalPosition<f64>) -> Point {
        let view_data = self.view_data.unchecked_read();
        Point::new(
            (point.x as f32 - view_data.size.width / 2.0) * view_data.pixel_size + view_data.center.x,
            (view_data.size.height / 2.0 - point.y as f32) * view_data.pixel_size + view_data.center.y,
        )
    }
}

pub struct GameSystem {
    pub data: Option<GameSystemStruct>,
}

impl GameSystem {
    const MSAA_SAMPLES: u32 = 4;
    
    pub fn new() -> Self {
        Self {
            data: None
        }
    }
}

impl System for GameSystem {
    fn start(&mut self, state: &State) {    
        let input_state = InputState {
            left: ElementState::Released,
            right: ElementState::Released,
            middle: ElementState::Released,
        };
        let cursor_state = CursorState::default(); 

        let bezier_segment = BezierSegment {
            from: Point::new(0.0, 0.0),
            ctrl1: Point::new(0.0, 200.0),
            ctrl2: Point::new(400.0, 400.0),
            to: Point::new(0.0, 400.0),
        };
        let bezier_data = BezierData {
            segment: bezier_segment,
            stroke_width: 10.0,
            subdivide: BezierUniform::subdivide_size(&bezier_segment),
        };
        let bezier_buffer = state.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Bezier buffer"),
            contents: bytemuck::cast_slice(&[BezierUniform::from(&bezier_data)]),
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
    
        let view_data = ViewData {
            center: Point::new(0.0, 0.0),
            pixel_size: 1.0,
            size: Size::new(state.config.width as _, state.config.height as _),
        };
        let view_bind_buffer = state.device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("View Buffer"),
                contents: bytemuck::cast_slice(&[MatrixUniform::from_view(&view_data)]),
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
                    resource: view_bind_buffer.as_entire_binding(),
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
                count: Self::MSAA_SAMPLES,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let screen_texture = GameSystemStruct::create_screen_texture(state);

        self.data = Some(GameSystemStruct {
            render_pipeline,
            screen_texture: DirtyFlag::new(screen_texture),
        
            view_data: DirtyFlag::new(view_data),
            view_bind_group,
            view_bind_buffer,
        
            bezier_data: DirtyFlag::new(bezier_data),
            bezier_bind_group,
            bezier_buffer,

            input_state,
            cursor_state,
        })
    }

    fn finish(&mut self) {
        self.data = None;
    }

    fn update(&mut self, state: &State) {
        if let Some(game) = self.data.as_mut() {
            if let (bezier_data, true) = game.bezier_data.get_all() {
                bezier_data.subdivide = BezierUniform::subdivide_size(&bezier_data.segment);
                state.queue.write_buffer(&game.bezier_buffer, 0, bytemuck::cast_slice(&[BezierUniform::from(bezier_data)]));
                game.bezier_data.clean_flag();
            }

            if game.input_state.middle == ElementState::Pressed {
                if let Some(moved) = game.cursor_state.moved.as_mut() {
                    game.view_data.write().center -= *moved;
                    game.cursor_state.pos -= *moved;
                }
            }

            if let (view_data, true) = game.view_data.get_all() {
                state.queue.write_buffer(
                    &game.view_bind_buffer,
                    0,
                    bytemuck::cast_slice(&[MatrixUniform::from_view(&view_data)]),
                );
                game.view_data.clean_flag();
            }

            if game.screen_texture.is_dirty() {
                game.screen_texture = DirtyFlag::new(GameSystemStruct::create_screen_texture(state));
            }

            game.cursor_state.moved = None;
        }
    }

    fn precess(&mut self, event: &WindowEvent) -> bool {
        if let Some(game) = &mut self.data {
            match event {
                WindowEvent::MouseWheel {
                    delta: MouseScrollDelta::LineDelta(_, v),
                    ..
                } => {
                    game.view_data.write().pixel_size *= 1.0 + -v * 0.2;
                    true
                },
                WindowEvent::MouseInput {
                    state,
                    button,
                    ..
                } => {
                    match button {
                        MouseButton::Left => game.input_state.left = *state,
                        MouseButton::Right => game.input_state.right = *state,
                        MouseButton::Middle => game.input_state.middle = *state,
                        _ => return false
                    }
                    true
                },
                WindowEvent::CursorMoved {
                    position,
                    ..
                } => {
                    let position = game.point_from_window(position);
                    *game.cursor_state.moved.get_or_insert(Vector::zero()) += position - game.cursor_state.pos;
                    game.cursor_state.pos = position;
                    
                    false
                },
                WindowEvent::Resized(physical_size) => {
                    game.screen_texture.set_dirty();
                    game.view_data.write().size = Size::new(physical_size.width as _, physical_size.height as _);
                    false
                },
                WindowEvent::KeyboardInput {
                    input: KeyboardInput {
                        virtual_keycode: Some(key),
                        state: ElementState::Pressed,
                        ..
                    },
                    ..
                } => {
                    const ADVANCE: f32 = 40.0;
                    match key {
                        VirtualKeyCode::W | VirtualKeyCode::Up => {
                            game.view_data.write().center.y -= ADVANCE;
                            true
                        }
                        VirtualKeyCode::A | VirtualKeyCode::Left => {
                            game.view_data.write().center.x += ADVANCE;
                            true
                        }
                        VirtualKeyCode::S | VirtualKeyCode::Down => {
                            game.view_data.write().center.y += ADVANCE;
                            true
                        }
                        VirtualKeyCode::D | VirtualKeyCode::Right => {
                            game.view_data.write().center.x -= ADVANCE;
                            true
                        }
                        _ => false,
                    }
                },
                _ => false
            }
        } else {
            false
        }
    }

    fn render(&mut self, state: &State, view: &wgpu::TextureView) {
        if let Some(ref game) = self.data {
            let mut encoder = state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Game Render Encoder"),
            });

            let game_view = game.screen_texture.read().create_view(&wgpu::TextureViewDescriptor::default() );
    
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
    
                render_pass.set_pipeline(&game.render_pipeline);
                render_pass.set_bind_group(0, &game.bezier_bind_group, &[]);
                render_pass.set_bind_group(1, &game.view_bind_group, &[]);

                render_pass.draw(0..game.bezier_data.read().subdivide * 2 + 2, 0..1);
            }

            state.queue.submit(std::iter::once(encoder.finish()));
        }
    }
}