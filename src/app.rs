//! Application state and main game loop

use winit::{
    event::{Event, WindowEvent, ElementState, MouseButton},
    event_loop::EventLoop,
    window::{Window, WindowBuilder},
    keyboard::{KeyCode, PhysicalKey},
};
use anyhow::Result;
use glam::Vec2;

use crate::world::World;
use crate::render::Renderer;
use crate::simulation::MaterialId;
use crate::ui::UiState;
use crate::levels::LevelManager;

/// Print controls to console
fn print_controls() {
    println!("=== Sunaba Controls ===");
    println!("Movement: WASD");
    println!("Materials: 1-9 (Stone, Sand, Water, Wood, Fire, Smoke, Steam, Lava, Oil)");
    println!("Spawn: Left Click");
    println!("Toggle Temperature Overlay: T");
    println!("Toggle Stats: F1");
    println!("Toggle Help: H");
    println!("Next/Prev Level: N/P");
    println!("======================");
}

/// Convert screen coordinates to world coordinates
fn screen_to_world(
    screen_x: f64,
    screen_y: f64,
    window_width: u32,
    window_height: u32,
    camera_pos: Vec2,
    camera_zoom: f32,
) -> (i32, i32) {
    // Convert to NDC (Normalized Device Coordinates)
    let ndc_x = (screen_x / window_width as f64) * 2.0 - 1.0;
    let ndc_y = 1.0 - (screen_y / window_height as f64) * 2.0; // Flip Y

    let aspect = window_width as f32 / window_height as f32;

    // Transform to world space
    let world_x = (ndc_x as f32 * aspect / camera_zoom) + camera_pos.x;
    let world_y = (ndc_y as f32 / camera_zoom) + camera_pos.y;

    log::trace!("screen_to_world: screen({:.0},{:.0}) → ndc({:.2},{:.2}) → world({:.1},{:.1}) [aspect={:.2}, zoom={:.2}, cam={:?}]",
               screen_x, screen_y, ndc_x, ndc_y, world_x, world_y, aspect, camera_zoom, camera_pos);

    (world_x as i32, world_y as i32)
}

/// Tracks current input state
pub struct InputState {
    // Movement keys
    pub w_pressed: bool,
    pub a_pressed: bool,
    pub s_pressed: bool,
    pub d_pressed: bool,

    // Material selection (1-9 map to material IDs)
    pub selected_material: u16,

    // Mouse state
    pub mouse_world_pos: Option<(i32, i32)>, // Converted to world coords
    pub left_mouse_pressed: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            w_pressed: false,
            a_pressed: false,
            s_pressed: false,
            d_pressed: false,
            selected_material: MaterialId::SAND, // Start with sand
            mouse_world_pos: None,
            left_mouse_pressed: false,
        }
    }
}

impl Default for InputState {
    fn default() -> Self {
        Self::new()
    }
}

pub struct App {
    window: Window,
    event_loop: EventLoop<()>,
    renderer: Renderer,
    world: World,
    input_state: InputState,
    egui_ctx: egui::Context,
    egui_state: egui_winit::State,
    ui_state: UiState,
    level_manager: LevelManager,
}

impl App {
    pub async fn new() -> Result<Self> {
        let event_loop = EventLoop::new()?;
        let window = WindowBuilder::new()
            .with_title("Sunaba - 2D Physics Sandbox")
            .with_inner_size(winit::dpi::LogicalSize::new(1280, 720))
            .build(&event_loop)?;

        let renderer = Renderer::new(&window).await?;
        let mut world = World::new();

        // Initialize level manager and load first level
        let level_manager = LevelManager::new();
        level_manager.load_current_level(&mut world);

        // Initialize egui
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
        );

        // Print controls to console
        print_controls();
        log::info!("Loaded level: {}", level_manager.current_level_name());

        Ok(Self {
            window,
            event_loop,
            renderer,
            world,
            input_state: InputState::default(),
            egui_ctx,
            egui_state,
            ui_state: UiState::new(),
            level_manager,
        })
    }
    
    pub fn run(self) -> Result<()> {
        let Self { window, event_loop, mut renderer, mut world, mut input_state, egui_ctx, mut egui_state, mut ui_state, mut level_manager } = self;
        
        event_loop.run(move |event, elwt| {
            // Let egui handle events first
            if let Event::WindowEvent { ref event, .. } = event {
                let _ = egui_state.on_window_event(&window, event);
            }

            match event {
                Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => {
                    elwt.exit();
                }
                Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                    renderer.resize(size.width, size.height);
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event: key_event, .. },
                    ..
                } => {
                    // Skip input if egui wants it
                    if egui_ctx.wants_keyboard_input() {
                        return;
                    }
                    if let PhysicalKey::Code(code) = key_event.physical_key {
                        let pressed = key_event.state == ElementState::Pressed;
                        log::debug!("Keyboard: {:?} {}", code, if pressed { "pressed" } else { "released" });

                        match code {
                            // Movement keys
                            KeyCode::KeyW => input_state.w_pressed = pressed,
                            KeyCode::KeyA => input_state.a_pressed = pressed,
                            KeyCode::KeyS => input_state.s_pressed = pressed,
                            KeyCode::KeyD => input_state.d_pressed = pressed,

                            // Material selection (1-9)
                            KeyCode::Digit1 => if pressed {
                                input_state.selected_material = MaterialId::STONE;
                            },
                            KeyCode::Digit2 => if pressed {
                                input_state.selected_material = MaterialId::SAND;
                            },
                            KeyCode::Digit3 => if pressed {
                                input_state.selected_material = MaterialId::WATER;
                            },
                            KeyCode::Digit4 => if pressed {
                                input_state.selected_material = MaterialId::WOOD;
                            },
                            KeyCode::Digit5 => if pressed {
                                input_state.selected_material = MaterialId::FIRE;
                            },
                            KeyCode::Digit6 => if pressed {
                                input_state.selected_material = MaterialId::SMOKE;
                            },
                            KeyCode::Digit7 => if pressed {
                                input_state.selected_material = MaterialId::STEAM;
                            },
                            KeyCode::Digit8 => if pressed {
                                input_state.selected_material = MaterialId::LAVA;
                            },
                            KeyCode::Digit9 => if pressed {
                                input_state.selected_material = MaterialId::OIL;
                            },

                            // UI toggles
                            KeyCode::F1 => if pressed {
                                ui_state.toggle_stats();
                            },
                            KeyCode::KeyH => if pressed {
                                ui_state.toggle_help();
                            },
                            KeyCode::KeyT => if pressed {
                                renderer.toggle_temperature_overlay();
                            },

                            // Level switching
                            KeyCode::KeyN => if pressed {
                                level_manager.next_level(&mut world);
                            },
                            KeyCode::KeyP => if pressed {
                                level_manager.prev_level(&mut world);
                            },

                            _ => {}
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { position, .. },
                    ..
                } => {
                    let (window_width, window_height) = renderer.window_size();
                    let world_pos = screen_to_world(
                        position.x,
                        position.y,
                        window_width,
                        window_height,
                        world.player_pos,
                        renderer.camera_zoom(),
                    );
                    input_state.mouse_world_pos = Some(world_pos);
                    log::trace!("Mouse: screen({:.0}, {:.0}) → world({}, {})",
                               position.x, position.y, world_pos.0, world_pos.1);
                }
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { state, button, .. },
                    ..
                } => {
                    if button == MouseButton::Left {
                        input_state.left_mouse_pressed = state == ElementState::Pressed;
                        log::info!("Mouse: {}", if state == ElementState::Pressed { "clicked" } else { "released" });
                    }
                }
                Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                    // Begin frame timing
                    ui_state.stats.begin_frame();

                    // Update player from input
                    world.update_player(&input_state, 1.0 / 60.0);

                    // Log camera state periodically
                    static mut FRAME_COUNT: u32 = 0;
                    unsafe {
                        FRAME_COUNT += 1;
                        if FRAME_COUNT % 120 == 0 {  // Every 2 seconds at 60fps
                            log::info!("Frame {}: player_pos={:?}, zoom={:.2}, selected_material={}",
                                       FRAME_COUNT, world.player_pos, renderer.camera_zoom(), input_state.selected_material);
                        }
                    }

                    // Spawn material on mouse click
                    if input_state.left_mouse_pressed {
                        if let Some((wx, wy)) = input_state.mouse_world_pos {
                            world.spawn_material(wx, wy, input_state.selected_material);
                        }
                    }

                    // Update simulation with timing
                    ui_state.stats.begin_sim();
                    world.update(1.0 / 60.0, &mut ui_state.stats);
                    ui_state.stats.end_sim();

                    // Collect world stats
                    ui_state.stats.collect_world_stats(&world);

                    // Update tooltip with world data
                    ui_state.update_tooltip(&world, world.materials(), input_state.mouse_world_pos);

                    // Prepare egui frame
                    let raw_input = egui_state.take_egui_input(&window);
                    let full_output = egui_ctx.run(raw_input, |ctx| {
                        // Get cursor position from egui context
                        let cursor_pos = ctx.pointer_hover_pos().unwrap_or(egui::pos2(0.0, 0.0));
                        ui_state.render(ctx, cursor_pos, input_state.selected_material, world.materials(), level_manager.current_level_name());
                    });

                    // Handle egui output
                    egui_state.handle_platform_output(&window, full_output.platform_output);

                    // Update temperature overlay texture
                    renderer.update_temperature_overlay(&world);

                    // Render world + UI
                    if let Err(e) = renderer.render(&world, &egui_ctx, full_output.textures_delta, full_output.shapes) {
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
