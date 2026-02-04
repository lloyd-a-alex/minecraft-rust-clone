use winit::{
    event::{Event, WindowEvent, ElementState, DeviceEvent, MouseButton, MouseScrollDelta},
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
    logger::init_logger();
    // Generate a master seed for hosting
    let master_seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "--join-localhost" { 
        run_game(NetworkManager::join("127.0.0.1:7878".to_string()), "Minecraft Clone (Test Client)"); return; 
    }

    println!("╔══════════════════════════════════════════════╗");
    println!("║       MINECRAFT RUST MULTIPLAYER             ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("1. SINGLEPLAYER (LAN Mode)");
    println!("2. HOST ONLINE (Auto: Ngrok -> SSH -> LAN)");
    println!("3. JOIN GAME");
println!("4. STRESS TEST (Multi-Client)");
    println!(""); 
    println!("> Enter option (1-4) and press ENTER:"); 
    
    let mut input = String::new(); 
    if std::io::stdin().read_line(&mut input).is_err() { input = "1".to_string(); }
    let choice = input.trim();

if choice == "4" {
        print!("How many clients? > "); io::stdout().flush().unwrap();
        let mut n_str = String::new(); std::io::stdin().read_line(&mut n_str).unwrap();
        let n: usize = n_str.trim().parse().unwrap_or(1);
        let exe = std::env::current_exe().unwrap();
        for _ in 0..n { std::process::Command::new(&exe).arg("--join-localhost").spawn().unwrap(); }
        run_game(NetworkManager::host("7878".to_string(), master_seed), "HOST (Stress Test)");
    } else if choice == "2" {
        log::info!("Starting online hosting...");
        if let Some(addr) = ngrok_utils::start_ngrok_tunnel("7878") { log::info!("✅ SERVER LIVE: {}", addr); } 
        else { log::warn!("❌ Tunnels failed. LAN only."); }
        run_game(NetworkManager::host("7878".to_string(), master_seed), "HOST (Online)");
    } else if choice == "3" {
        print!("Enter IP: "); io::stdout().flush().unwrap();
        let mut ip = String::new(); std::io::stdin().read_line(&mut ip).unwrap();
        run_game(NetworkManager::join(ip.trim().to_string()), "CLIENT");
    } else {
        run_game(NetworkManager::host("7878".to_string(), master_seed), "Minecraft Clone (Singleplayer)");
    };
}

fn run_game(network: NetworkManager, title: &str) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title(title).with_inner_size(winit::dpi::LogicalSize::new(1280.0, 720.0)).build(&event_loop).unwrap());
    let _ = window.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window.set_cursor_grab(CursorGrabMode::Locked)); window.set_cursor_visible(false);
    
    let mut renderer = pollster::block_on(Renderer::new(&window));
    // Use network seed if hosting, otherwise temp (0) until Handshake
    let initial_seed = network.seed.unwrap_or(0);
    let mut world = World::new(initial_seed);
    renderer.rebuild_all_chunks(&world);
    
    // SAFE SPAWN LOGIC (No Water)
    let mut player = Player::new();
    let mut spawn_found = false;
    // Spiral search for non-water spawn
    for r in 0..20 {
        if spawn_found { break; }
        for x in -r..=r {
            for z in -r..=r {
                for y in (0..100).rev() {
                    let b = world.get_block(BlockPos{x, y, z});
                    if b.is_solid() && !b.is_water() {
                        player.position = glam::Vec3::new(x as f32 + 0.5, y as f32 + 2.0, z as f32 + 0.5);
                        spawn_found = true;
                        break;
                    }
                }
                if spawn_found { break; }
            }
        }
    }
    
    let window_clone = window.clone();
    let mut last_frame = Instant::now();
    let mut is_paused = false;
    let mut cursor_pos = (0.0, 0.0);
    let mut modifiers = winit::keyboard::ModifiersState::default(); 
    let mut win_size = (1280.0, 720.0);
    
// BREAKING LOGIC with GRACE PERIOD
    let mut breaking_pos: Option<BlockPos> = None;
    let mut break_progress = 0.0;
    let mut left_click = false;
    let mut break_grace_timer = 0.0; // 0.5s grace

    let mut net_timer = 0.0;
    let mut death_timer = 0.0; // New timer

    event_loop.run(move |event, elwt| {
        match event {
            // TOUCHPAD SCROLL -> HOTBAR
            Event::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } => {
                let y = match delta { MouseScrollDelta::LineDelta(_, y) => y, MouseScrollDelta::PixelDelta(p) => (p.y / 10.0) as f32 };
                if y > 0.0 { 
                    player.inventory.selected_hotbar_slot = (player.inventory.selected_hotbar_slot + 8) % 9; 
                } else if y < 0.0 { 
                    player.inventory.selected_hotbar_slot = (player.inventory.selected_hotbar_slot + 1) % 9; 
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => { if !is_paused && !player.inventory_open { player.process_mouse(delta.0, delta.1); } }
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => elwt.exit(),
                WindowEvent::Resized(size) => { renderer.resize(size.width, size.height); win_size = (size.width as f64, size.height as f64); }
                WindowEvent::CursorMoved { position, .. } => cursor_pos = (position.x, position.y),
                WindowEvent::ModifiersChanged(m) => modifiers = m.state(), 
                WindowEvent::MouseInput { button, state, .. } => {
                    let pressed = state == ElementState::Pressed;
                    if player.inventory_open && pressed {
                        // INVENTORY INTERACTION (Stacking Fix)
                        let (mx, my) = cursor_pos; let (w, h) = (win_size.0 as f32, win_size.1 as f32);
                        let ndc_x = (mx as f32 / w) * 2.0 - 1.0; let ndc_y = -((my as f32 / h) * 2.0 - 1.0);
                        let sw = 0.12; let sh = sw * (w/h); let sx = -(9.0*sw)/2.0; let by = -0.9;
                        let mut click = None; let mut craft = false; let mut c_idx = 0;
                        let is_right_click = button == MouseButton::Right;

                        for i in 0..9 { if ndc_x >= sx + i as f32 * sw && ndc_x < sx + (i+1) as f32 * sw && ndc_y >= by && ndc_y < by + sh { click = Some(i); break; } }
                        let iby = by + sh * 1.5;
                        for r in 0..3 { for c in 0..9 { let x = sx + c as f32 * sw; let y = iby + r as f32 * sh; if ndc_x >= x && ndc_x < x + sw && ndc_y >= y && ndc_y < y + sh { click = Some(9+r*9+c); } } }
                        
                        let cx = 0.3; let cy = 0.5;
                        let grid_size = if player.crafting_open { 3 } else { 2 };
                        for r in 0..grid_size { for c in 0..grid_size { 
                            let x = cx + c as f32 * sw; let y = cy - r as f32 * sh;
                            if ndc_x >= x+0.01 && ndc_x < x+sw-0.01 && ndc_y >= y+0.01 && ndc_y < y+sh-0.01 { click = Some(99); craft = true; c_idx = if player.crafting_open { r*3+c } else { match r*2+c { 0=>0, 1=>1, 2=>3, 3=>4, _=>0 } }; } 
                        } }
                        
                        if let Some(i) = click {
                            let slot = if craft { &mut player.inventory.crafting_grid[c_idx] } else { &mut player.inventory.slots[i] };
                            if is_right_click {
                                // Right click logic (unchanged for brevity)
                                if player.inventory.cursor_item.is_none() {
                                    if let Some(s) = slot { let half = s.count/2; if half>0 { player.inventory.cursor_item = Some(player::ItemStack::new(s.item, half)); s.count -= half; if s.count == 0 { *slot = None; } } }
                                } else {
                                    let cursor = player.inventory.cursor_item.as_mut().unwrap();
                                    if let Some(s) = slot { 
                                        if s.item == cursor.item && s.count < 64 { s.count += 1; cursor.count -= 1; if cursor.count == 0 { player.inventory.cursor_item = None; } }
                                    } else { *slot = Some(player::ItemStack::new(cursor.item, 1)); cursor.count -= 1; if cursor.count == 0 { player.inventory.cursor_item = None; } }
                                }
                            } else {
                                // LEFT CLICK STACKING LOGIC
                                if let Some(cursor) = &mut player.inventory.cursor_item {
                                    if let Some(s) = slot {
                                        if s.item == cursor.item {
                                            // Stack!
                                            let space = 64 - s.count;
                                            let transfer = space.min(cursor.count);
                                            s.count += transfer;
cursor.count -= transfer;
                                            if cursor.count == 0 { player.inventory.cursor_item = None; }
                                        } else {
// Swap
                                            let temp = *s; 
                                            *s = *cursor; 
                                            *cursor = temp;
                                        }
                                    } else {
                                        // Place
                                        *slot = Some(*cursor); player.inventory.cursor_item = None;
                                    }
                                } else {
                                    // Pick up
                                    player.inventory.cursor_item = *slot; *slot = None;
                                }
                            }
                            if craft { player.inventory.check_recipes(); }
                        }
                        
                        // Output logic (unchanged)
                        let ox = cx + 3.0*sw; let oy = cy - 0.5*sh;
                        if ndc_x >= ox && ndc_x < ox+sw && ndc_y >= oy && ndc_y < oy+sh { 
                            if let Some(o) = player.inventory.crafting_output { 
                                if player.inventory.cursor_item.is_none() || (player.inventory.cursor_item.unwrap().item == o.item && player.inventory.cursor_item.unwrap().count + o.count <= 64) {
                                    if let Some(curr) = player.inventory.cursor_item { player.inventory.cursor_item = Some(player::ItemStack::new(curr.item, curr.count + o.count)); } 
                                    else { player.inventory.cursor_item = Some(o); }
                                    player.inventory.craft(); player.inventory.check_recipes(); 
                                } 
                            } 
                        }
                    } else if button == MouseButton::Left {
                        left_click = pressed; 
                    } else if button == MouseButton::Right && pressed && !player.inventory_open {
                        // Place Block
                        let (sin, cos) = player.rotation.x.sin_cos(); let (ysin, ycos) = player.rotation.y.sin_cos();
                        let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                        if let Some((_, place)) = world.raycast(player.position + glam::Vec3::new(0.0, player.height*0.9, 0.0), dir, 5.0) {
                            let p_min = player.position - glam::Vec3::new(player.radius, 0.0, player.radius);
                            let p_max = player.position + glam::Vec3::new(player.radius, player.height, player.radius);
                            let b_min = glam::Vec3::new(place.x as f32, place.y as f32, place.z as f32);
                            let b_max = b_min + glam::Vec3::ONE;
                            
                            let intersect_x = p_min.x < b_max.x && p_max.x > b_min.x;
                            let intersect_y = p_min.y < b_max.y && p_max.y > b_min.y;
                            let intersect_z = p_min.z < b_max.z && p_max.z > b_min.z;
                            
                            if !(intersect_x && intersect_y && intersect_z) {
                                if let Some(blk) = player.inventory.get_selected_item() {
                                    if !blk.is_tool() && !blk.is_item() {
                                        let c = world.place_block(place, blk); 
                                        player.inventory.remove_one_from_hand(); 
                                        network.send_packet(Packet::BlockUpdate { pos: place, block: blk });
                                        for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); }
                                    } else if blk == BlockType::CraftingTable {
                                        if world.get_block(place) == BlockType::CraftingTable {
                                            player.inventory_open = true; player.crafting_open = true;
                                            let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined); window_clone.set_cursor_visible(true);
                                        }
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
                            player.crafting_open = false; 
                            if player.inventory_open { let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined); window_clone.set_cursor_visible(true); }
                            else { let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false); }
                        } else if key == winit::keyboard::KeyCode::KeyQ && pressed && !is_paused && !player.inventory_open {
                             let drop_all = modifiers.shift_key(); 
                             if let Some(stack) = player.inventory.drop_item(drop_all) {
                                 let dir = glam::Vec3::new(player.rotation.y.cos() * player.rotation.x.cos(), player.rotation.x.sin(), player.rotation.y.sin() * player.rotation.x.cos()).normalize();
                                 let ent = world::ItemEntity { position: player.position + glam::Vec3::new(0.0, 1.5, 0.0), velocity: dir * 10.0, item_type: stack.item, count: stack.count, pickup_delay: 1.5, lifetime: 300.0, rotation: 0.0, bob_offset: 0.0 };
                                 world.entities.push(ent);
                             }
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
                let now = Instant::now(); let dt = (now - last_frame).as_secs_f32(); last_frame = now;

                while let Some(pkt) = network.try_recv() {
                    match pkt {
                        Packet::Handshake { seed, .. } => {
                            log::info!("🌍 RECEIVED SEED: {}. REBUILDING WORLD...", seed);
                            world = World::new(seed);
                            renderer.rebuild_all_chunks(&world);
                            // SAFE SPAWN (Spiral Search)
                            let mut spawn_found = false;
                            for r in 0..20 {
                                if spawn_found { break; }
                                for x in -r..=r {
                                    for z in -r..=r {
                                        for y in (0..100).rev() {
                                            let b = world.get_block(BlockPos{x, y, z});
                                            if b.is_solid() && !b.is_water() {
                                                player.position = glam::Vec3::new(x as f32 + 0.5, y as f32 + 2.0, z as f32 + 0.5);
                                                spawn_found = true;
                                                break;
                                            }
                                        }
                                        if spawn_found { break; }
                                    }
                                }
                            }
                            if !spawn_found { player.position = glam::Vec3::new(0.0, 80.0, 0.0); }
                        },
                        Packet::PlayerMove { id, x, y, z, ry } => { if let Some(p) = world.remote_players.iter_mut().find(|p| p.id == id) { p.position = glam::Vec3::new(x,y,z); p.rotation = ry; } else { world.remote_players.push(world::RemotePlayer{id, position:glam::Vec3::new(x,y,z), rotation:ry}); } },
                        Packet::BlockUpdate { pos, block } => { let c = world.place_block(pos, block); for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); } },
                        _ => {}
                    }
                }
                
                net_timer += dt; if net_timer > 0.05 { net_timer = 0.0; network.send_packet(Packet::PlayerMove { id: network.my_id, x: player.position.x, y: player.position.y, z: player.position.z, ry: player.rotation.y }); }
                
                if !is_paused {
                    // DEATH & RESPAWN LOGIC
                    if player.is_dead {
                        death_timer += dt;
                        if death_timer > 3.0 {
                            // Find safe spawn
                            let mut spawn_found = false;
                            for r in 0..20 {
                                if spawn_found { break; }
                                for x in -r..=r {
                                    for z in -r..=r {
                                        for y in (0..100).rev() {
                                            let b = world.get_block(BlockPos{x, y, z});
                                            if b.is_solid() && !b.is_water() {
                                                player.respawn();
                                                player.position = glam::Vec3::new(x as f32 + 0.5, y as f32 + 2.0, z as f32 + 0.5);
                                                spawn_found = true;
                                                death_timer = 0.0;
                                                break;
                                            }
                                        }
                                        if spawn_found { break; }
                                    }
                                }
                            }
                            if !spawn_found { player.respawn(); }
                        }
                    } else {
                        player.update(dt, &world);
                        
                        // TARGETING & BREAKING (Grace Period Logic)
                        let (sin, cos) = player.rotation.x.sin_cos(); let (ysin, ycos) = player.rotation.y.sin_cos();
                        let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                        let ray_res = world.raycast(player.position + glam::Vec3::new(0.0, player.height*0.9, 0.0), dir, 5.0);
                        
                        let current_target = ray_res.map(|(h, _)| h);
                        
                        if left_click && !player.inventory_open {
                            if let Some(hit) = current_target {
                                if Some(hit) != breaking_pos {
                                    if breaking_pos.is_some() && break_grace_timer > 0.0 {
                                        break_grace_timer -= dt;
                                    } else {
                                        breaking_pos = Some(hit); break_progress = 0.0; break_grace_timer = 0.5; 
                                    }
                                } else {
                                    break_grace_timer = 0.5;
                                }
                                
                                if Some(hit) == breaking_pos {
                                    let blk = world.get_block(hit); let tool = player.inventory.get_selected_item().unwrap_or(BlockType::Air);
                                    let speed = if tool.get_tool_class() == blk.get_best_tool_type() || blk.get_best_tool_type() == "none" { tool.get_tool_speed() } else { 1.0 };
                                    let h = blk.get_hardness();
                                    if h > 0.0 { break_progress += (speed / h) * dt; } else { break_progress = 1.1; }
                                    if break_progress >= 1.0 {
                                        let c = world.break_block(hit); 
                                        network.send_packet(Packet::BlockUpdate { pos: hit, block: BlockType::Air });
                                        for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); }
                                        breaking_pos = None; break_progress = 0.0;
                                    }
                                }
                            } else {
                                if breaking_pos.is_some() && break_grace_timer > 0.0 { break_grace_timer -= dt; } 
                                else { breaking_pos = None; break_progress = 0.0; }
                            }
                        } else { 
                            if breaking_pos.is_some() && break_grace_timer > 0.0 { break_grace_timer -= dt; } 
                            else { breaking_pos = None; break_progress = 0.0; }
                        }
                    }
                    world.update_entities(dt, &mut player);
                }
                window_clone.request_redraw();
            },
            _ => {}
        }
    }).unwrap();
}