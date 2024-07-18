#![allow(unused_imports)]
#![allow(deprecated)]

use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::image::{ImageUsage, SwapchainImage};
use vulkano::swapchain::{Swapchain, SwapchainCreationError, SurfaceTransform, PresentMode, FullscreenExclusive};
use vulkano::sync::{GpuFuture, now};
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, PipelineBindPoint};

use cairo::{Context, ImageSurface, Format};
use egui::Rect;
use poppler::PopplerDocument;
use std::fs::File;
use std::sync::Arc;
use std::ptr::NonNull;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{WindowBuilder};

struct PdfRenderer {
    document:       PopplerDocument,
    current_page:   usize,
    crop_rect:      Option<Rect>,
    bounding_boxes: Vec<Rect>,
}

impl PdfRenderer {
    fn new(pdf_path: &str) -> Option<Self> {
        let document = PopplerDocument::new_from_file(pdf_path, None).ok()?;
        Some(Self {
            document,
            current_page: 0,
            crop_rect: None,
            bounding_boxes: vec![],
        })
    }

    fn render_page_to_surface(&self, page_number: usize) -> Result<ImageSurface, cairo::Error> {
        if let Some(page) = self.document.get_page(page_number) {
            let (width, height) = page.get_size();
            let surface = ImageSurface::create(Format::ARgb32, width as i32, height as i32)?;
            let context = Context::new(&surface)?;

            if let Some(crop_rect) = &self.crop_rect {
                context.rectangle(
                    crop_rect.min.x as f64,
                    crop_rect.min.y as f64,
                    crop_rect.width() as f64,
                    crop_rect.height() as f64,
                );
                context.clip();
            }

            page.render_for_printing(&context);
            Ok(surface)
        } else {
            Err(cairo::Error::InvalidStatus)
        }
    }
}

fn main() {
    // Initialize the event loop and create a window
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    // Initialize Vulkan instance and device
    let instance = Instance::new(None, vulkano::instance::InstanceExtensions::none(), None).unwrap();
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    let queue_family = physical.queue_families().find(|&q| q.supports_graphics()).unwrap();
    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &DeviceExtensions::none(),
        [(queue_family, 0.5)].iter().cloned(),
    ).unwrap();
    let queue = queues.next().unwrap();

    // Create a surface from the window
    let surface = vulkano_win::VkSurfaceBuild::build_vk_surface(window, instance.clone()).unwrap();

    // Create a swapchain
    let (mut swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();
        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;
        let dimensions = caps.current_extent.unwrap_or([1280, 1024]);

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            dimensions,
            1,
            ImageUsage::color_attachment(),
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            FullscreenExclusive::Default,
            true,
            None,
        ).unwrap()
    };

    // Create a PDF renderer
    let pdf_renderer = PdfRenderer::new("maxwell.pdf").unwrap();

    // Main event loop
    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                *control_flow = ControlFlow::Exit;
            },
            Event::MainEventsCleared => {
                // Render PDF page to a Cairo surface
                let surface = pdf_renderer.render_page_to_surface(0).unwrap();

                // Here you would convert the Cairo surface to a format suitable for Vulkano and
                // render it to the swapchain images.
                // This part requires further Vulkan setup and handling.
            },
            _ => (),
        }
    });
}
