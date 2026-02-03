use winit::{
    event::{Event, WindowEvent, ElementState, DeviceEvent, MouseButton},
    event_loop::EventLoop,
    window::{WindowBuilder, CursorGrabMode},
    keyboard::PhysicalKey,
};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::io::{self, Write};

mod renderer; mod world; mod texture; mod player; mod logger; mod noise_gen; mod network; mod ngrok_utils;
use renderer::Renderer; use world::{World, BlockPos, BlockType}; use player::Player; use network::{NetworkManager, Packet};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--join-localhost" { logger::init_logger(); run_game(NetworkManager::join("127.0.0.1:7878".to_string()), "Minecraft Clone (Test Client)"); return; }
    println!("╔══════════════════════════════════════════════╗");
    println!("║       MINECRAFT RUST MULTIPLAYER             ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("1. HOST GAME (Server)");
    println!("2. JOIN GAME (Client)");
    println!("3. TEST SERVER (Host + Auto-Join Client)");
    print!("\n> "); io::stdout().flush().unwrap();
    let mut input = String::new(); std::io::stdin().read_line(&mut input).unwrap();
    let choice = input.trim();

    if choice == "3" {
        let exe = std::env::current_exe().unwrap();
        std::thread::spawn(move || { std::thread::sleep(std::time::Duration::from_secs(2)); std::process::Command::new(exe).arg("--join-localhost").spawn().unwrap(); });
        logger::init_logger(); run_game(NetworkManager::host("7878".to_string()), "HOST");
    } else if choice == "1" {
        if let Some(addr) = ngrok_utils::start_ngrok_tunnel("7878") { println!("✅ SERVER: {}", addr); } else { println!("❌ LAN ONLY (7878)"); }
        logger::init_logger(); run_game(NetworkManager::host("7878".to_string()), "HOST");
    } else {
        print!("Enter IP: "); io::stdout().flush().unwrap(); let mut ip = String::new(); std::io::stdin().read_line(&mut ip).unwrap();
        logger::init_logger(); run_game(NetworkManager::join(ip.trim().to_string()), "CLIENT");
    };
}

fn run_game(network: NetworkManager, title: &str) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title(title).with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)).build(&event_loop).unwrap());
    let _ = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked)); window.set_cursor_visible(false);
    
    let mut renderer = pollster::block_on(Renderer::new(&window));
    let mut world = World::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32);
    renderer.rebuild_all_chunks(&world);
    let mut player = Player::new();
    
    let window_clone = window.clone();
    let mut last_frame = Instant::now();
    let mut is_paused = false;
    let mut cursor_pos = (0.0, 0.0);
    let mut win_size = (1280.0, 720.0);
    let mut breaking_pos: Option<BlockPos> = None;
    let mut break_progress = 0.0;
    let mut left_click = false;
    let mut net_timer = 0.0;

    event_loop.run(move |event, elwt| {
        match event {
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => { if !is_paused && !player.inventory_open { player.process_mouse(delta.0, delta.1); } }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => { renderer.resize(size.width, size.height); win_size = (size.width as f64, size.height as f64); }
                WindowEvent::CursorMoved { position, .. } => cursor_pos = (position.x, position.y),
                WindowEvent::MouseInput { button, state, .. } => {
                    let pressed = state == ElementState::Pressed;
                    if button == MouseButton::Left {
                        if player.inventory_open && pressed {
                            // Inventory Click Logic
                            let (mx, my) = cursor_pos; let (w, h) = (win_size.0 as f32, win_size.1 as f32);
                            let ndc_x = (mx as f32 / w) * 2.0 - 1.0; let ndc_y = -((my as f32 / h) * 2.0 - 1.0);
                            let sw = 0.12; let sh = sw * (w/h); let sx = -(9.0*sw)/2.0; let by = -0.9;
                            let mut click = None; let mut craft = false; let mut c_idx = 0;
                            // Check hotbar
                            for i in 0..9 { if ndc_x >= sx + i as f32 * sw && ndc_x < sx + (i+1) as f32 * sw && ndc_y >= by && ndc_y < by + sh { click = Some(i); break; } }
                            // Check inventory
                            let iby = by + sh * 1.5;
                            for r in 0..3 { for c in 0..9 { let x = sx + c as f32 * sw; let y = iby + r as f32 * sh; if ndc_x >= x && ndc_x < x + sw && ndc_y >= y && ndc_y < y + sh { click = Some(9+r*9+c); } } }
                            // Check Crafting (2x2)
                            let cx = 0.3; let cy = 0.5;
                            for r in 0..2 { for c in 0..2 { let x = cx + c as f32 * sw; let y = cy - r as f32 * sh; if ndc_x >= x && ndc_x < x + sw && ndc_y >= y && ndc_y < y + sh { click = Some(99); craft = true; c_idx = r*2+c; } } }
                            // Check Output
                            let ox = cx + 3.0*sw; let oy = cy - 0.5*sh;
                            if ndc_x >= ox && ndc_x < ox+sw && ndc_y >= oy && ndc_y < oy+sh { if let Some(o) = player.inventory.crafting_output { if player.inventory.cursor_item.is_none() { player.inventory.cursor_item = Some(o); player.inventory.craft(); player.inventory.check_recipes(); } } }

                            if let Some(i) = click {
                                if craft { let s = player.inventory.crafting_grid[c_idx]; player.inventory.crafting_grid[c_idx] = player.inventory.cursor_item; player.inventory.cursor_item = s; player.inventory.check_recipes(); }
                                else { let s = player.inventory.slots[i]; player.inventory.slots[i] = player.inventory.cursor_item; player.inventory.cursor_item = s; }
                            }
                        } else {
                            left_click = pressed; if !pressed { breaking_pos = None; break_progress = 0.0; }
                        }
                    } else if button == MouseButton::Right && pressed && !player.inventory_open {
                        // Place Block
                        let (sin, cos) = player.rotation.x.sin_cos(); let (ysin, ycos) = player.rotation.y.sin_cos();
                        let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                        if let Some((_, place)) = world.raycast(player.position + glam::Vec3::new(0.0, player.height*0.9, 0.0), dir, 5.0) {
                            let p_min = player.position - glam::Vec3::new(player.radius, 0.0, player.radius);
                            let p_max = player.position + glam::Vec3::new(player.radius, player.height, player.radius);
                            let b_min = glam::Vec3::new(place.x as f32, place.y as f32, place.z as f32);
                            let b_max = b_min + glam::Vec3::ONE;
                            if !(p_min.x < b_max.x && p_max.x > b_min.x && p_min.y < b_max.y && p_max.y > b_min.y && p_min.z < b_max.z && p_max.z > b_min.z) {
                                if let Some(blk) = player.inventory.get_selected_item() {
                                    if !blk.is_tool() && !blk.is_item() {
                                        let c = world.place_block(place, blk); network.send_packet(Packet::BlockUpdate { pos: place, block: blk });
                                        player.inventory.remove_one_from_hand();
                                        for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); }
                                    }
                                }
                            }
                        }
                    }
                },
                WindowEvent::KeyboardInput { event, .. } => {
                    if let PhysicalKey::Code(key) = event.physical_key {
                        let pressed = event.state == ElementState::Pressed;
                        if key == winit::keyboard::KeyCode::Escape && pressed {
                            if player.inventory_open { 
                                player.inventory_open = false; 
                                let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false);
                                if let Some(c) = player.inventory.cursor_item { player.inventory.add_item(c.item); player.inventory.cursor_item = None; }
                            } else {
                                is_paused = !is_paused;
                                if is_paused { let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined); window_clone.set_cursor_visible(true); }
                                else { let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false); }
                            }
                        } else if key == winit::keyboard::KeyCode::KeyE && pressed && !is_paused {
                            player.inventory_open = !player.inventory_open;
                            if player.inventory_open { let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined); window_clone.set_cursor_visible(true); }
                            else { let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false); }
                        } else if !is_paused { player.handle_input(key, pressed); }
                    }
                },
                WindowEvent::RedrawRequested => {
                    renderer.break_progress = if breaking_pos.is_some() { break_progress } else { 0.0 };
                    renderer.render(&player, &world, is_paused, cursor_pos);
                },
                _ => {}
            },
            Event::AboutToWait => {
                while let Some(pkt) = network.try_recv() {
                    match pkt {
                        Packet::PlayerMove { id, x, y, z, ry } => { if let Some(p) = world.remote_players.iter_mut().find(|p| p.id == id) { p.position = glam::Vec3::new(x,y,z); p.rotation = ry; } else { world.remote_players.push(world::RemotePlayer{id, position:glam::Vec3::new(x,y,z), rotation:ry}); } },
                        Packet::BlockUpdate { pos, block } => { let c = world.place_block(pos, block); for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); } },
                        _ => {}
                    }
                }
                let now = Instant::now(); let dt = (now - last_frame).as_secs_f32(); last_frame = now;
                net_timer += dt; if net_timer > 0.05 { net_timer = 0.0; network.send_packet(Packet::PlayerMove { id: network.my_id, x: player.position.x, y: player.position.y, z: player.position.z, ry: player.rotation.y }); }
                
                if !is_paused {
                    player.update(dt, &world); world.update_entities(dt, &mut player);
                    if left_click && !player.inventory_open {
                        let (sin, cos) = player.rotation.x.sin_cos(); let (ysin, ycos) = player.rotation.y.sin_cos();
                        let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                        if let Some((hit, _)) = world.raycast(player.position + glam::Vec3::new(0.0, player.height*0.9, 0.0), dir, 5.0) {
                            if Some(hit) != breaking_pos { breaking_pos = Some(hit); break_progress = 0.0; }
                            let blk = world.get_block(hit); let tool = player.inventory.get_selected_item().unwrap_or(BlockType::Air);
                            let speed = if tool.get_tool_class() == blk.get_best_tool_type() || blk.get_best_tool_type() == "none" { tool.get_tool_speed() } else { 1.0 };
                            let h = blk.get_hardness();
                            if h > 0.0 { break_progress += (speed / h) * dt; } else { break_progress = 1.1; }
                            if break_progress >= 1.0 {
                                let c = world.break_block(hit); network.send_packet(Packet::BlockUpdate { pos: hit, block: BlockType::Air });
                                for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); }
                                breaking_pos = None; break_progress = 0.0;
                            }
                        } else { breaking_pos = None; break_progress = 0.0; }
                    } else { breaking_pos = None; break_progress = 0.0; }
                }
                window_clone.request_redraw();
            },
            _ => {}
        }
    }).unwrap();
}