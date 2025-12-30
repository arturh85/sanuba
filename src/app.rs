//! Application state and main game loop

use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
};
use anyhow::Result;

use crate::world::World;
use crate::render::Renderer;

pub struct App {
    window: Window,
    event_loop: EventLoop<()>,
    renderer: Renderer,
    world: World,
}

impl App {
    pub async fn new() -> Result<Self> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Sunaba")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)?;
        
        let renderer = Renderer::new(&window).await?;
        let world = World::new();
        
        Ok(Self {
            window,
            event_loop,
            renderer,
            world,
        })
    }
    
    pub fn run(self) -> Result<()> {
        let Self { window, event_loop, mut renderer, mut world } = self;
        
        event_loop.run(move |event, elwt| {
            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    elwt.exit();
                }
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    renderer.resize(size.width, size.height);
                }
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    // Update simulation
                    world.update(1.0 / 60.0);
                    
                    // Render
                    if let Err(e) = renderer.render(&world) {
                        log::error!("Render error: {e}");
                    }
                }
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })?;
        
        Ok(())
    }
}
