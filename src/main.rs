use winit::{
    event::{Event, WindowEvent, ElementState, DeviceEvent, MouseButton},
    event_loop::EventLoop,
    window::{WindowBuilder, CursorGrabMode},
    keyboard::{PhysicalKey, KeyCode},
};

use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod renderer;
mod world;
mod texture;
mod player;
mod logger;
mod noise_gen;
mod network;
mod ngrok_utils;

use renderer::Renderer;
use world::{World, BlockPos, BlockType, ItemEntity};
use player::Player;
use logger::log_renderer_init;
use log::info;
use glam::Vec3;
use network::{NetworkManager, Packet};
use std::io::{self, Write};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--join-localhost" {
        logger::init_logger();
        run_game(NetworkManager::join("127.0.0.1:7878".to_string()), 0, "Minecraft Clone");
        return;
    }

    println!("╔══════════════════════════════════════════════╗");
    println!("║       MINECRAFT RUST MULTIPLAYER             ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("1. HOST GAME (Internet / Ngrok)");
    println!("2. HOST GAME (LAN / Singleplayer)");
    println!("3. JOIN GAME (Client)");
    println!("4. TEST MODE (Auto-Join)");
    print!("\n> ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    logger::init_logger();
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;

    if choice == "4" {
        let exe = std::env::current_exe().unwrap();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            std::process::Command::new(exe).arg("--join-localhost").spawn().unwrap();
        });
        run_game(NetworkManager::host("7878".to_string()), seed, "HOST");
    } else if choice == "1" {
        if let Some(addr) = ngrok_utils::start_ngrok_tunnel("7878") {
             println!("✅ SERVER READY: {}", addr);
        } else {
             println!("❌ Ngrok failed. Hosting Local only.");
        }
        run_game(NetworkManager::host("7878".to_string()), seed, "HOST");
    } else if choice == "2" {
        println!("✅ HOSTING LOCALLY on Port 7878");
        run_game(NetworkManager::host("7878".to_string()), seed, "HOST (LAN)");
    } else {
        print!("Enter IP: ");
        io::stdout().flush().unwrap();
        let mut ip = String::new();
        std::io::stdin().read_line(&mut ip).unwrap();
        run_game(NetworkManager::join(ip.trim().to_string()), 0, "CLIENT");
    };
}

fn run_game(network: NetworkManager, initial_seed: u32, window_title: &str) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title(window_title).with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)).build(&event_loop).unwrap());
    
    let set_cursor_grab = |grab: bool, win: &Arc<winit::window::Window>| {
        if grab {
            let _ = win.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| win.set_cursor_grab(CursorGrabMode::Locked));
            win.set_cursor_visible(false);
        } else {
            let _ = win.set_cursor_grab(CursorGrabMode::None);
            win.set_cursor_visible(true);
        }
    };
    
    set_cursor_grab(true, &window);

    let mut renderer = pollster::block_on(Renderer::new(&window));
    let window_clone = window.clone();

    let mut current_seed = initial_seed;
    let mut world = World::new(current_seed);
    
    for (pos, _) in &world.chunks { renderer.update_chunk(pos.0, pos.1, &world); }
    log_renderer_init(1280, 720);

    let mut player = Player::new(Vec3::new(0.0, 80.0, 0.0));
    let mut last_frame_time = Instant::now();
    let mut frame_count = 0;
    let mut last_log_time = Instant::now();
    
    let mut mouse_delta_x = 0.0;
    let mut mouse_delta_y = 0.0;
    let mut breaking_block_pos: Option<BlockPos> = None;
    let mut breaking_progress = 0.0;
    let mut network_timer = 0.0;

    event_loop.run(move |event, target| {
        match event {
            Event::WindowEvent { event: WindowEvent::Resized(new_size), .. } => {
                renderer.resize(new_size);
            },
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let now = Instant::now();
                let dt = (now - last_frame_time).as_secs_f32();
                last_frame_time = now;

                if !player.inventory_open {
                    player.handle_mouse_input(dt, mouse_delta_x, mouse_delta_y);
                    player.update(dt, &world);
                }
                mouse_delta_x = 0.0; mouse_delta_y = 0.0;
                
                world.update_entities(dt, &mut player);
                
                // Underwater Check
                let cam_pos = player.position + Vec3::new(0.0, player.height * 0.9, 0.0);
                let head_block = world.get_block(BlockPos { x: cam_pos.x.floor() as i32, y: cam_pos.y.floor() as i32, z: cam_pos.z.floor() as i32 });
                let is_underwater = head_block == BlockType::Water;

                let view = player.build_view_projection_matrix(renderer.config.width as f32 / renderer.config.height as f32);
                renderer.update_camera(view);
                renderer.render(&player, &world, is_underwater);

                frame_count += 1;
                if now.duration_since(last_log_time).as_secs_f32() >= 5.0 {
                    info!("FPS: {:.1} | Delta: {:.3}ms", frame_count as f32 / 5.0, dt * 1000.0);
                    frame_count = 0; last_log_time = now;
                }
            }
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta, .. }, .. } => {
                if !player.inventory_open {
                    mouse_delta_x += delta.0 as f32; mouse_delta_y += delta.1 as f32;
                }
            }
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event, .. }, .. } => {
                if let PhysicalKey::Code(key) = event.physical_key {
                     if key == KeyCode::Escape && event.state == ElementState::Pressed {
                         player.inventory_open = !player.inventory_open;
                         set_cursor_grab(!player.inventory_open, &window_clone);
                     } else {
                        player.handle_key_input(key, event.state == ElementState::Pressed);
                     }
                }
            }
            Event::WindowEvent { event: WindowEvent::MouseInput { state, button, .. }, .. } => {
                if !player.inventory_open {
                    match button {
                        MouseButton::Left => player.mouse.left = state == ElementState::Pressed,
                        MouseButton::Right => player.mouse.right = state == ElementState::Pressed,
                        _ => {}
                    }
                }
            }
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => { target.exit(); }
            Event::AboutToWait => {
                while let Some(packet) = network.try_recv() {
                    match packet {
                        Packet::Handshake { seed, .. } => {
                            if !network.is_server && seed != current_seed {
                                info!("Syncing World Seed: {}", seed);
                                current_seed = seed;
                                world = World::new(seed);
                                renderer.chunk_meshes.clear();
                                for (pos, _) in &world.chunks { renderer.update_chunk(pos.0, pos.1, &world); }
                                player.position.y = 80.0; 
                            }
                        },
                        Packet::PlayerMove { id, x, y, z, ry } => {
                            if let Some(rp) = world.remote_players.iter_mut().find(|p| p.id == id) {
                                rp.position = Vec3::new(x, y, z); rp.rotation = ry;
                            } else {
                                world.remote_players.push(world::RemotePlayer { id, position: Vec3::new(x, y, z), rotation: ry });
                            }
                        },
                        Packet::BlockUpdate { pos, block } => {
                            let chunks = world.place_block(pos, block);
                            for (cx, cz) in chunks { renderer.update_chunk(cx, cz, &world); }
                        },
                        _ => {}
                    }
                }

                let now = Instant::now();
                network_timer += (now - last_frame_time).as_secs_f32();
                let dt = (now - last_frame_time).as_secs_f32(); // Approximate for logic

                if network_timer > 0.05 { 
                    network.send_packet(Packet::PlayerMove {
                        id: network.my_id,
                        x: player.position.x, y: player.position.y, z: player.position.z, ry: player.rotation.y
                    });
                }
                if network.is_server && network_timer > 1.0 {
                     network.send_packet(Packet::Handshake { username: "Host".to_string(), seed: current_seed });
                     network_timer = 0.0;
                }

                if !player.inventory_open && (player.mouse.left || player.mouse.right) {
                    let (sin, cos) = player.rotation.x.sin_cos();
                    let (y_sin, y_cos) = player.rotation.y.sin_cos();
                    let dir = Vec3::new(y_cos * cos, sin, y_sin * cos).normalize();
                    let eye = player.position + Vec3::new(0.0, player.height * 0.9, 0.0);

                    if let Some((hit, place)) = world.raycast(eye, dir, 5.0) {
                        if player.mouse.right && !player.mouse.left { 
                             if let Some(blk) = player.inventory.get_selected_item() {
                                 if blk.is_solid() {
                                     let chunks = world.place_block(place, blk);
                                     network.send_packet(Packet::BlockUpdate { pos: place, block: blk });
                                     for (cx, cz) in chunks { renderer.update_chunk(cx, cz, &world); }
                                     player.mouse.right = false; 
                                 }
                             }
                        } else if player.mouse.left { 
                            if breaking_block_pos != Some(hit) {
                                breaking_block_pos = Some(hit);
                                breaking_progress = 0.0;
                            }
                            // SLOW MINING: ~0.25 seconds to break
                            breaking_progress += 0.02; // Tuned for 60-144hz, roughly
                            
                            if breaking_progress >= 1.0 {
                                let chunks = world.break_block(hit);
                                if !chunks.is_empty() { 
                                    network.send_packet(Packet::BlockUpdate { pos: hit, block: BlockType::Air });
                                    for (cx, cz) in chunks { renderer.update_chunk(cx, cz, &world); }
                                }
                                breaking_block_pos = None;
                                breaking_progress = 0.0;
                                player.mouse.left = false; 
                            }
                        }
                    } else { breaking_block_pos = None; breaking_progress = 0.0; }
                } else { breaking_block_pos = None; breaking_progress = 0.0; }
                
                if let Some(dropped) = player.inventory.drop_item(player.keys.down) {
                     let (sin, cos) = player.rotation.x.sin_cos();
                     let (y_sin, y_cos) = player.rotation.y.sin_cos();
                     let fwd = Vec3::new(y_cos * cos, sin, y_sin * cos).normalize();
                     world.entities.push(ItemEntity {
                         position: player.position + Vec3::new(0.0, player.height * 0.8, 0.0),
                         velocity: fwd * 6.0 + Vec3::new(0.0, 1.5, 0.0),
                         item_type: dropped.item, count: dropped.count,
                         pickup_delay: 1.5, lifetime: 300.0, rotation: 0.0, bob_offset: 0.0,
                     });
                }
                window_clone.request_redraw();
            }
            _ => {}
        }
    }).unwrap();
}