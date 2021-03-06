use anyhow::{Context, Result};
use raw_window_handle::HasRawWindowHandle;

use crate::texture::Texture;

#[cfg(target_family = "wasm")]
const BACKEND: wgpu::Backends = wgpu::Backends::BROWSER_WEBGPU;

#[cfg(target_os = "windows")]
const BACKEND: wgpu::Backends = wgpu::Backends::DX12;

#[cfg(target_os = "macos")]
const BACKEND: wgpu::Backends = wgpu::Backends::METAL;

#[cfg(target_os = "linux")]
const BACKEND: wgpu::Backends = wgpu::Backends::VULKAN;

pub struct Renderer {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    dimensions: [u32; 2],
    depth_texture: Texture,
}

impl Renderer {
    pub async fn new(
        window_handle: &impl HasRawWindowHandle,
        dimensions: &[u32; 2],
    ) -> Result<Self> {
        let instance = wgpu::Instance::new(BACKEND);

        let surface = unsafe { instance.create_surface(window_handle) };

        let adapter = Self::create_adapter(&instance, &surface).await?;

        let (device, queue) = Self::request_device(&adapter).await?;

        let swapchain_format = surface
            .get_preferred_format(&adapter)
            .context("Failed to get preferred surface format!")?;

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: dimensions[0],
            height: dimensions[1],
            present_mode: wgpu::PresentMode::Fifo,
        };

        surface.configure(&device, &config);

        let depth_texture =
            Texture::create_depth_texture(&device, dimensions[0], dimensions[1], "Depth Texture");

        Ok(Self {
            surface,
            device,
            queue,
            config,
            dimensions: *dimensions,
            depth_texture,
        })
    }

    async fn create_adapter(
        instance: &wgpu::Instance,
        surface: &wgpu::Surface,
    ) -> Result<wgpu::Adapter> {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to request a GPU adapter!")
    }

    async fn request_device(adapter: &wgpu::Adapter) -> Result<(wgpu::Device, wgpu::Queue)> {
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .context("Failed to request a device!")?;
        Ok((device, queue))
    }

    pub fn resize(&mut self, dimensions: [u32; 2]) {
        if dimensions[0] == 0 || dimensions[1] == 0 {
            return;
        }
        self.dimensions = dimensions;
        self.config.width = dimensions[0];
        self.config.height = dimensions[1];
        self.surface.configure(&self.device, &self.config);
        self.depth_texture = Texture::create_depth_texture(
            &self.device,
            dimensions[0],
            dimensions[1],
            "Depth Texture",
        );
    }

    pub fn render(&mut self, dimensions: &[u32; 2]) -> Result<()> {
        match self.render_frame(dimensions) {
            Ok(_) => {}
            // Recreate the swapchain if lost
            Err(wgpu::SurfaceError::Lost) => self.resize(self.dimensions),
            // The system is out of memory, we should probably quit
            // Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            // All other errors should be resolved by the next frame
            Err(e) => eprintln!("{:?}", e),
        }
        Ok(())
    }

    fn render_frame(&mut self, _dimensions: &[u32; 2]) -> Result<(), wgpu::SurfaceError> {
        // let height = if dimensions[1] > 0 {
        //     dimensions[1] as f32
        // } else {
        //     1.0
        // };
        // let aspect_ratio = dimensions[0] as f32 / height as f32;

        let frame = self.surface.get_current_texture()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();

        Ok(())
    }

    pub fn cleanup(&mut self) -> Result<()> {
        Ok(())
    }
}
