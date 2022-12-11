use winit::{
    event::*,
    event_loop::{ ControlFlow, EventLoop },
    window::Window,
};

pub fn init() {
    env_logger::init();
}

#[allow(unused)]
pub trait System {
    fn start(&mut self, state: &State) {}
    fn finish(&mut self) {}
    fn update(&mut self, state: &State) {}
    fn render(&mut self, state: &State, view: &wgpu::TextureView) {}
    fn precess(&mut self, event: &winit::event::WindowEvent) -> bool { false }
}

pub struct App<T: 'static> {
    pub state: State,
    pub event_loop: EventLoop<T>,
}

impl<T> App<T> {
    pub fn new(event_loop: EventLoop<T>, window: Window) -> Self {
        Self {
            state: State::new(window),
            event_loop,
        }
    }

    pub fn run<S: System + 'static>(self, mut system: S) -> ! {
        let (mut state, event_loop) = (self.state, self.event_loop);

        system.start(&state);

        event_loop.run(move |event, _, control_flow| {
            match event {
                Event::WindowEvent { window_id, ref event }
                    if window_id == state.window.id()
                    && !system.precess(event)
                    => {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => state.on_resize(*physical_size),
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => state.on_resize(**new_inner_size),
                        _ => {}
                    }
                },
                Event::RedrawRequested(window_id) if window_id == state.window.id() => {
                    match state.clean_screen() {
                        Ok((output, view)) => {
                            system.render(&state, &view);
                            output.present();
                        },
                        // 如果发生上下文丢失，就重新配置 surface
                        Err(wgpu::SurfaceError::Lost) => state.on_resize(state.size()),
                        // 系统内存不足，此时应该退出
                        Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        // 所有其他错误（如过时、超时等）都应在下一帧解决
                        Err(e) => eprintln!("{:?}", e),
                    }
                }
                Event::MainEventsCleared => {
                    system.update(&state);
    
                    // 除非手动请求，否则 RedrawRequested 只会触发一次
                    state.window.request_redraw();
                }
                _ => {}
            }
        })
    }
}

pub struct State {
    pub window: Window,
    pub surface: wgpu::Surface,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
}

impl State {
    pub fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(&window) };
        // 适配器，指向实际显卡的一个handle
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(), // LowPower低功耗，HighPerfirnabce高性能（独显）
                compatible_surface: Some(&surface), // 兼容传入的surface
                force_fallback_adapter: false, // 是否强制wgpu选择某个能在所有硬件上工作的适配器（软渲染系统）
            }
        )).expect("Couldn't create the adapter!");
        // let adapter = instance
        //     .enumerate_adapters(wgpu::Backends::all())
        //     .filter(|adapter| {
        //         // 检查该适配器是否支持我们的 surface
        //         surfaces.get_preferred_format(&adapter).is_some()
        //     })
        //     .next()
        //     .unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::empty(),
                limits: wgpu::Limits::default(),
                label: None,
            },
            None, // 是否追踪APIg调用路径
        )).expect("Couldn't create the device!");

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: *surface.get_supported_formats(&adapter).first().unwrap(),
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo, // VSync
            // alpha_mode: *surface.get_supported_alpha_modes(&adapter).first().unwrap(),
        };
        surface.configure(&device, &config);

        Self {
            window,
            surface,
            device,
            queue,
            config,
        }
    }

    pub fn on_resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width * new_size.height > 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        winit::dpi::PhysicalSize::new(self.config.width, self.config.height)
    }

    pub fn clean_screen(&self) -> Result<(wgpu::SurfaceTexture, wgpu::TextureView), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
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
        }

        // submit 方法能传入任何实现了 IntoIter 的参数
        self.queue.submit(std::iter::once(encoder.finish()));

        Ok((output, view))
    }
}
