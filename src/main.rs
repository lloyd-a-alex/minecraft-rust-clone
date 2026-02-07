use winit::{
    event::{Event, WindowEvent, ElementState, DeviceEvent, MouseButton, MouseScrollDelta, KeyEvent},
    event_loop::EventLoop,
    window::{WindowBuilder, CursorGrabMode},
    keyboard::{KeyCode, PhysicalKey},
};
use std::sync::Arc;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

mod renderer; mod world; mod texture; mod player; mod logger; mod noise_gen; mod network; mod ngrok_utils;
use renderer::Renderer; use world::{World, BlockPos, BlockType}; use player::Player; use network::{NetworkManager, Packet};

// --- MENU SYSTEM ---
#[derive(PartialEq)] enum GameState { Menu, Playing }
struct Rect { x: f32, y: f32, w: f32, h: f32 }
impl Rect { fn contains(&self, nx: f32, ny: f32) -> bool { nx >= self.x - self.w/2.0 && nx <= self.x + self.w/2.0 && ny >= self.y - self.h/2.0 && ny <= self.y + self.h/2.0 } }
enum MenuAction { Singleplayer, Host, Join, Stress, Quit }
struct MenuButton { rect: Rect, text: String, action: MenuAction, hovered: bool }
pub struct MainMenu { buttons: Vec<MenuButton> }
impl MainMenu {
fn new() -> Self {
        let mut b = Vec::new();
        let w = 0.8; // Wider buttons
        let h = 0.12; 
        let g = 0.05; 
        let sy = 0.2; 

        b.push(MenuButton{rect:Rect{x:0.0,y:sy,w,h}, text:"SINGLEPLAYER".to_string(), action:MenuAction::Singleplayer, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g),w,h}, text:"HOST ONLINE".to_string(), action:MenuAction::Host, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*2.0,w,h}, text:"JOIN GAME".to_string(), action:MenuAction::Join, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*3.0,w,h}, text:"STRESS TEST".to_string(), action:MenuAction::Stress, hovered:false});
        b.push(MenuButton{rect:Rect{x:0.0,y:sy-(h+g)*4.5,w,h}, text:"QUIT".to_string(), action:MenuAction::Quit, hovered:false});
        MainMenu { buttons: b }
    }
}

// --- UI STRUCTURES ---
#[repr(C)] #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIElement { pub pos: [f32; 2], pub size: [f32; 2], pub tex_idx: u32, pub padding: u32 }
pub struct Hotbar { pub slots: [Option<(world::BlockType, u32)>; 9], pub selected_slot: usize }
impl Hotbar { fn new() -> Self { Self { slots: [None; 9], selected_slot: 0 } } }

fn main() {
    logger::init_logger();
    let master_seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;
    let args: Vec<String> = std::env::args().collect();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title("Minecraft Rust Clone").with_maximized(true).build(&event_loop).unwrap());
    
    // Initialize Renderer immediately (No Pollster block needed if we map async correctly, but keeping simple)
    let window_arc = window.clone(); // Clone ARC for renderer
let mut renderer = pollster::block_on(Renderer::new(&window_arc));
    let mut world = World::new(master_seed); // Temp world for menu background
    
    // --- GAME STATE ---
    let mut game_state = GameState::Menu;
    let mut main_menu = MainMenu::new();
    let mut network_mgr: Option<NetworkManager> = None;
    
    // If CLI args provided, jump straight to game
    if args.len() > 1 && args[1] == "--join-localhost" { 
        network_mgr = Some(NetworkManager::join("127.0.0.1:7878".to_string()));
        game_state = GameState::Playing;
    }

    // --- PLAYER & LOGIC STATE (Preserved from your code) ---
let mut player = Player::new();
    
// --- SPAWN STATE ---
    let mut spawn_found = false;
    let mut breaking_pos: Option<BlockPos> = None;
    let mut break_progress = 0.0;
    let mut left_click = false;
    let mut break_grace_timer = 0.0;
    let mut net_timer = 0.0;
    let mut death_timer = 0.0;
    let mut is_paused = false;
    let mut cursor_pos = (0.0, 0.0);
    let mut modifiers = winit::keyboard::ModifiersState::default(); 
    let mut win_size = (window.inner_size().width, window.inner_size().height);
    let window_clone = window.clone();
    let mut last_frame = Instant::now();

    // Initial Cursor Logic
    window.set_cursor_grab(CursorGrabMode::None).unwrap();
    window.set_cursor_visible(true);

    event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => elwt.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                renderer.resize(size.width, size.height);
                win_size = (size.width, size.height);
            },
            Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                cursor_pos = (position.x, position.y);
                if game_state == GameState::Menu {
                    let ndc_x = (position.x as f32 / win_size.0 as f32) * 2.0 - 1.0;
                    let ndc_y = 1.0 - (position.y as f32 / win_size.1 as f32) * 2.0;
                    for btn in &mut main_menu.buttons { btn.hovered = btn.rect.contains(ndc_x, ndc_y); }
                } else if !is_paused && !player.inventory_open {
                    player.process_mouse(0.0, 0.0); // Handled by DeviceEvent for raw input
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => { 
                if game_state == GameState::Playing && !is_paused && !player.inventory_open { player.process_mouse(delta.0, delta.1); } 
            },
            Event::WindowEvent { event: WindowEvent::ModifiersChanged(m), .. } => modifiers = m.state(),
            
// --- MOUSE INPUT ---
            Event::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } => {
                let pressed = state == ElementState::Pressed;
                
                if game_state == GameState::Menu && pressed && button == MouseButton::Left {
                    let ndc_x = (cursor_pos.0 as f32 / win_size.0 as f32) * 2.0 - 1.0;
                    let ndc_y = 1.0 - (cursor_pos.1 as f32 / win_size.1 as f32) * 2.0;
                    
                    let mut action = None;
                    for btn in &main_menu.buttons { if btn.rect.contains(ndc_x, ndc_y) { action = Some(&btn.action); break; } }
                    
                    if let Some(act) = action {
                        match act {
                            MenuAction::Singleplayer => {
                                world = World::new(master_seed);
                                renderer.rebuild_all_chunks(&world);
                                game_state = GameState::Playing;
                                spawn_found = false;
                            },
                            MenuAction::Host => {
                                log::info!("Starting online hosting...");
                                if let Some(addr) = ngrok_utils::start_ngrok_tunnel("7878") { log::info!("✅ SERVER LIVE: {}", addr); } 
                                network_mgr = Some(NetworkManager::host("7878".to_string(), master_seed));
                                world = World::new(master_seed);
                                renderer.rebuild_all_chunks(&world);
                                game_state = GameState::Playing;
                                spawn_found = false;
                            },
                            MenuAction::Join => {
                                network_mgr = Some(NetworkManager::join("127.0.0.1:7878".to_string()));
                                game_state = GameState::Playing;
                                spawn_found = false;
                            },
                            MenuAction::Stress => {
                                let exe = std::env::current_exe().unwrap();
                                for _ in 0..5 { std::process::Command::new(&exe).arg("--join-localhost").spawn().unwrap(); }
                                network_mgr = Some(NetworkManager::host("7878".to_string(), master_seed));
                                world = World::new(master_seed);
                                renderer.rebuild_all_chunks(&world);
                                game_state = GameState::Playing;
                                spawn_found = false;
                            },
                            MenuAction::Quit => elwt.exit(),
                        }

                        if game_state == GameState::Playing {
                            let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined).or_else(|_| window_clone.set_cursor_grab(CursorGrabMode::Locked));
                            window_clone.set_cursor_visible(false);
                            
                            log::info!("🔍 Searching for dry land...");
                            'spawn_search: for r in 0..300i32 {
                                for x in -r..=r {
                                    for z in -r..=r {
                                        if x.abs() != r && z.abs() != r { continue; }
                                        for y in (0..world::CHUNK_SIZE_Y as i32 - 1).rev() {
                                            let b = world.get_block(BlockPos { x, y, z });
                                            if b.is_solid() {
                                                if !b.is_water() {
                                                    player.position = glam::Vec3::new(x as f32 + 0.5, y as f32 + 2.0, z as f32 + 0.5);
                                                    spawn_found = true;
                                                    log::info!("✅ Spawn found at: {}, {}, {}", x, y, z);
                                                    break 'spawn_search;
                                                } else { break; }
                                            }
                                        }
                                    }
                                }
                            }
                            if !spawn_found && network_mgr.as_ref().map(|n| n.is_server).unwrap_or(true) {
                                log::warn!("⚠️ No dry land found! Constructing emergency platform.");
                                player.position = glam::Vec3::new(0.0, 80.0, 0.0);
                                for x in -1..=1 { for z in -1..=1 { world.place_block(BlockPos { x, y: 78, z }, BlockType::Bedrock); } }
                            }
                        }
                    }
                } else if game_state == GameState::Playing {
                    if player.inventory_open && pressed {
                        let (mx, my) = cursor_pos; 
                        let (w, h) = (win_size.0 as f32, win_size.1 as f32);
                        let ndc_x = (mx as f32 / w) * 2.0 - 1.0; 
                        let ndc_y = -((my as f32 / h) * 2.0 - 1.0);
                        let sw = 0.12; 
                        let sh = sw * (w / h); 
                        let sx = -(9.0 * sw) / 2.0; 
                        let by = -0.9;
                        let mut click = None; 
                        let mut craft = false; 
                        let mut c_idx = 0;
                        let is_right_click = button == MouseButton::Right;

                        for i in 0..9 { 
                            if ndc_x >= sx + i as f32 * sw && ndc_x < sx + (i + 1) as f32 * sw && ndc_y >= by && ndc_y < by + sh { 
                                click = Some(i); 
                                break; 
                            } 
                        }
                        
                        let iby = by + sh * 1.5;
                        for r in 0..3 { 
                            for c in 0..9 { 
                                let x = sx + c as f32 * sw; 
                                let y = iby + r as f32 * sh; 
                                if ndc_x >= x && ndc_x < x + sw && ndc_y >= y && ndc_y < y + sh { 
                                    click = Some(9 + r * 9 + c); 
                                } 
                            } 
                        }
                        
                        let cx = 0.3; 
                        let cy = 0.5;
                        let grid_size = if player.crafting_open { 3 } else { 2 };
                        for r in 0..grid_size { 
                            for c in 0..grid_size { 
                                let x = cx + c as f32 * sw; 
                                let y = cy - r as f32 * sh;
                                if ndc_x >= x + 0.01 && ndc_x < x + sw - 0.01 && ndc_y >= y + 0.01 && ndc_y < y + sh - 0.01 { 
                                    click = Some(99); 
                                    craft = true; 
                                    c_idx = if player.crafting_open { r * 3 + c } else { match r * 2 + c { 0 => 0, 1 => 1, 2 => 3, 3 => 4, _ => 0 } }; 
                                } 
                            } 
                        }
                        
                        if let Some(i) = click {
                            let slot = if craft { &mut player.inventory.crafting_grid[c_idx] } else { &mut player.inventory.slots[i] };
                            if is_right_click {
                                if player.inventory.cursor_item.is_none() {
                                    if let Some(s) = slot { 
                                        let half = s.count / 2; 
                                        if half > 0 { 
                                            player.inventory.cursor_item = Some(player::ItemStack::new(s.item, half)); 
                                            s.count -= half; 
                                            if s.count == 0 { *slot = None; } 
                                        } 
                                    }
                                } else {
                                    let cursor = player.inventory.cursor_item.as_mut().unwrap();
                                    if let Some(s) = slot { 
                                        if s.item == cursor.item && s.count < 64 { 
                                            s.count += 1; 
                                            cursor.count -= 1; 
                                            if cursor.count == 0 { player.inventory.cursor_item = None; } 
                                        }
                                    } else { 
                                        *slot = Some(player::ItemStack::new(cursor.item, 1)); 
                                        cursor.count -= 1; 
                                        if cursor.count == 0 { player.inventory.cursor_item = None; } 
                                    }
                                }
                            } else {
                                if let Some(cursor) = &mut player.inventory.cursor_item {
                                    if let Some(s) = slot {
                                        if s.item == cursor.item {
                                            let space = 64 - s.count; 
                                            let transfer = space.min(cursor.count);
                                            s.count += transfer; 
                                            cursor.count -= transfer;
                                            if cursor.count == 0 { player.inventory.cursor_item = None; }
                                        } else { 
                                            let temp = *s; 
                                            *s = *cursor; 
                                            *cursor = temp; 
                                        }
                                    } else { 
                                        *slot = Some(*cursor); 
                                        player.inventory.cursor_item = None; 
                                    }
                                } else { 
                                    player.inventory.cursor_item = *slot; 
                                    *slot = None; 
                                }
                            }
                            if craft { player.inventory.check_recipes(); }
                        }
                        
                        let ox = cx + 3.0 * sw; 
                        let oy = cy - 0.5 * sh;
                        if ndc_x >= ox && ndc_x < ox + sw && ndc_y >= oy && ndc_y < oy + sh { 
                            if let Some(o) = player.inventory.crafting_output { 
                                if player.inventory.cursor_item.is_none() || (player.inventory.cursor_item.unwrap().item == o.item && player.inventory.cursor_item.unwrap().count + o.count <= 64) {
                                    if let Some(curr) = player.inventory.cursor_item { 
                                        player.inventory.cursor_item = Some(player::ItemStack::new(curr.item, curr.count + o.count)); 
                                    } else { 
                                        player.inventory.cursor_item = Some(o); 
                                    }
                                    player.inventory.craft(); 
                                    player.inventory.check_recipes(); 
                                } 
                            } 
                        }
                    } else if button == MouseButton::Left {
                        left_click = pressed;
                    } else if button == MouseButton::Right && pressed && !player.inventory_open {
                        let (sin, cos) = player.rotation.x.sin_cos(); 
                        let (ysin, ycos) = player.rotation.y.sin_cos();
                        let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                        
                        if let Some((hit, place)) = world.raycast(player.position + glam::Vec3::new(0.0, player.height * 0.4, 0.0), dir, 5.0) {
                            let targeted_block = world.get_block(hit);
                            let held_item = player.inventory.get_selected_item().unwrap_or(BlockType::Air);

                            if held_item == BlockType::BucketEmpty && targeted_block == BlockType::Water {
                                world.place_block(hit, BlockType::Air);
                                player.inventory.slots[player.inventory.selected_hotbar_slot] = Some(player::ItemStack::new(BlockType::BucketWater, 1));
                                renderer.update_chunk(hit.x / 16, hit.z / 16, &world);
                            } else if held_item == BlockType::BucketWater {
                                world.place_block(place, BlockType::Water);
                                player.inventory.slots[player.inventory.selected_hotbar_slot] = Some(player::ItemStack::new(BlockType::BucketEmpty, 1));
                                renderer.update_chunk(place.x / 16, place.z / 16, &world);
                            } else if held_item.get_tool_class() == "hoe" && (targeted_block == BlockType::Grass || targeted_block == BlockType::Dirt) {
                                world.place_block(hit, BlockType::FarmlandDry);
                                renderer.update_chunk(hit.x / 16, hit.z / 16, &world);
                            } else if targeted_block == BlockType::CraftingTable {
                                player.inventory_open = true; 
                                player.crafting_open = true;
                                let _ = window_clone.set_cursor_grab(CursorGrabMode::None); 
                                window_clone.set_cursor_visible(true);
                            } else {
                                let p_min = player.position - glam::Vec3::new(player.radius, player.height * 0.5, player.radius);
                                let p_max = player.position + glam::Vec3::new(player.radius, player.height * 0.5, player.radius);
                                let b_min = glam::Vec3::new(place.x as f32, place.y as f32, place.z as f32);
                                let b_max = b_min + glam::Vec3::ONE;

                                let intersect = p_min.x < b_max.x - 0.1 && p_max.x > b_min.x + 0.1 && 
                                                p_min.y < b_max.y && p_max.y > b_min.y && 
                                                p_min.z < b_max.z - 0.1 && p_max.z > b_min.z + 0.1;

                                if !intersect {
                                    if let Some(blk) = player.inventory.get_selected_item() {
                                        if !blk.is_tool() && !blk.is_item() {
                                            let mut actual_blk = blk;
                                            if blk == BlockType::Chest {
                                                for (dx, dz) in &[(1,0), (-1,0), (0,1), (0,-1)] {
                                                    let neighbor_pos = BlockPos { x: place.x + dx, y: place.y, z: place.z + dz };
                                                    if world.get_block(neighbor_pos) == BlockType::Chest {
                                                        world.place_block(neighbor_pos, BlockType::ChestLeft);
                                                        actual_blk = BlockType::ChestRight;
                                                        renderer.update_chunk(neighbor_pos.x / 16, neighbor_pos.z / 16, &world);
                                                        break;
                                                    }
                                                }
                                            }
                                            let c = world.place_block(place, actual_blk);
                                            player.inventory.remove_one_from_hand(); 
                                            if let Some(net) = &network_mgr { net.send_packet(Packet::BlockUpdate { pos: place, block: actual_blk }); }
                                            for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            
            // --- KEYBOARD INPUT ---
            Event::WindowEvent { event: WindowEvent::KeyboardInput { event: KeyEvent { physical_key: PhysicalKey::Code(key), state, .. }, .. }, .. } => {
                let pressed = state == ElementState::Pressed;
                if game_state == GameState::Playing {
                    if key == KeyCode::Escape && pressed {
                        if player.inventory_open { 
                            player.inventory_open = false; 
                            let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false);
                            if let Some(c) = player.inventory.cursor_item { player.inventory.add_item(c.item); player.inventory.cursor_item = None; }
                        } else {
                            is_paused = !is_paused;
                            if is_paused { let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined); window_clone.set_cursor_visible(true); }
                            else { let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false); }
                        }
                    } else if key == KeyCode::KeyE && pressed && !is_paused {
                        player.inventory_open = !player.inventory_open;
                        player.crafting_open = false; 
                        if player.inventory_open { 
                            player.keys.forward = false; player.keys.backward = false; player.keys.left = false; player.keys.right = false;
                            let _ = window_clone.set_cursor_grab(CursorGrabMode::Confined); window_clone.set_cursor_visible(true); 
                        } else { let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false); }
                    } else if key == KeyCode::KeyQ && pressed && !is_paused && !player.inventory_open {
                        let drop_all = modifiers.shift_key(); 
                        if let Some(stack) = player.inventory.drop_item(drop_all) {
                            let base_dir = glam::Vec3::new(player.rotation.y.cos() * player.rotation.x.cos(), player.rotation.x.sin(), player.rotation.y.sin() * player.rotation.x.cos()).normalize();
                            let loop_count = if drop_all { stack.count } else { 1 };
                            for i in 0..loop_count {
                                let i_u32 = i as u32;
                                let px_u32 = (player.position.x * 100.0) as u32;
                                let py_u32 = (player.position.y * 100.0) as u32;
                                let pz_u32 = (player.position.z * 100.0) as u32;
                                let r_x = (i_u32.wrapping_mul(13).wrapping_add(px_u32) % 20) as f32 / 40.0 - 0.25;
                                let r_y = (i_u32.wrapping_mul(7).wrapping_add(py_u32) % 20) as f32 / 40.0 - 0.25;
                                let r_z = (i_u32.wrapping_mul(19).wrapping_add(pz_u32) % 20) as f32 / 40.0 - 0.25;
                                let jitter = glam::Vec3::new(r_x, r_y, r_z);
                                let ent = world::ItemEntity { position: player.position + glam::Vec3::new(0.0, 1.5, 0.0), velocity: (base_dir + jitter).normalize() * 10.0, item_type: stack.item, count: 1, pickup_delay: 1.5, lifetime: 300.0, rotation: 0.0, bob_offset: i as f32 * 0.5 };
                                world.entities.push(ent);
                            }
                        }
                    } else if !is_paused && !player.inventory_open { 
                        player.handle_input(key, pressed);
                        if pressed && key == KeyCode::Space && !player.is_flying && player.on_ground { player.velocity.y = 8.0; }
                        if pressed && key == KeyCode::KeyF { player.is_flying = !player.is_flying; if player.is_flying { player.velocity = glam::Vec3::ZERO; } }
                        if key == KeyCode::ControlLeft { player.is_sprinting = pressed; }
                        if pressed {
                            let slot = match key { KeyCode::Digit1=>Some(0), KeyCode::Digit2=>Some(1), KeyCode::Digit3=>Some(2), KeyCode::Digit4=>Some(3), KeyCode::Digit5=>Some(4), KeyCode::Digit6=>Some(5), KeyCode::Digit7=>Some(6), KeyCode::Digit8=>Some(7), KeyCode::Digit9=>Some(8), _=>None };
                            if let Some(s) = slot { player.inventory.selected_hotbar_slot = s; }
                        }
                    }
                }
            },
            
            // --- TOUCHPAD SCROLL ---
            Event::WindowEvent { event: WindowEvent::MouseWheel { delta, .. }, .. } => {
                if game_state == GameState::Playing {
                    let y = match delta { MouseScrollDelta::LineDelta(_, y) => y, MouseScrollDelta::PixelDelta(p) => (p.y / 10.0) as f32 };
                    if y > 0.0 { player.inventory.selected_hotbar_slot = (player.inventory.selected_hotbar_slot + 8) % 9; } 
                    else if y < 0.0 { player.inventory.selected_hotbar_slot = (player.inventory.selected_hotbar_slot + 1) % 9; }
                }
            },

            // --- GAME UPDATE & DRAW ---
            Event::WindowEvent { event: WindowEvent::RedrawRequested, .. } => {
                let now = Instant::now();
                let dt = (now - last_frame).as_secs_f32().min(0.1);
                last_frame = now;

if game_state == GameState::Playing {
                    // --- INFINITE GENERATION ---
                    let p_cx = (player.position.x / 16.0).floor() as i32;
                    let p_cz = (player.position.z / 16.0).floor() as i32;
                    let r_dist = 6;
                    for dx in -r_dist..=r_dist {
                        for dz in -r_dist..=r_dist {
                            let target = (p_cx + dx, p_cz + dz);
                            if !world.chunks.contains_key(&target) {
                                // Add world generation logic here if desired
                            }
                        }
                    }

                    // --- DAY/NIGHT CYCLE ---
                    let day_time = (renderer.start_time.elapsed().as_secs_f32() % 600.0) / 600.0;
                    let _sky_brightness = (day_time * std::f32::consts::PI * 2.0).sin().max(0.1);

                    if let Some(network) = &mut network_mgr {
                        while let Some(pkt) = network.try_recv() {
                            match pkt {
                                Packet::Handshake { seed, .. } => {
                                    log::info!("🌍 RECEIVED SEED: {}. REBUILDING WORLD...", seed);
                                    world = World::new(seed); renderer.rebuild_all_chunks(&world);
                                    // RE-RUN SPAWN LOGIC FOR CLIENT
                                    let mut spawn_found = false;
                                    'net_spawn: for r in 0..300i32 {
                                        for x in -r..=r { for z in -r..=r {
                                            if x.abs() != r && z.abs() != r { continue; }
                                            for y in (0..150).rev() {
                                                let b = world.get_block(BlockPos{x, y, z});
                                                if b.is_solid() && !b.is_water() {
                                                    player.position = glam::Vec3::new(x as f32 + 0.5, y as f32 + 2.0, z as f32 + 0.5);
                                                    spawn_found = true; break 'net_spawn;
                                                }
                                            }
                                        }}
                                    }
if !spawn_found { player.position = glam::Vec3::new(0.0, 80.0, 0.0); player.velocity = glam::Vec3::ZERO; }
                                },
                                Packet::PlayerMove { id, x, y, z, ry } => {

                                    if let Some(p) = world.remote_players.iter_mut().find(|p| p.id == id) { p.position = glam::Vec3::new(x,y,z); p.rotation = ry; } 
                                    else { world.remote_players.push(world::RemotePlayer{id, position:glam::Vec3::new(x,y,z), rotation:ry}); }
                                },
                                Packet::BlockUpdate { pos, block } => { let c = world.place_block(pos, block); for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); } },
                                _ => {}
                            }
                        }
                        net_timer += dt; 
                        if net_timer > 0.05 { net_timer = 0.0; network.send_packet(Packet::PlayerMove { id: network.my_id, x: player.position.x, y: player.position.y, z: player.position.z, ry: player.rotation.y }); }
                    }

                    if !is_paused {
                        // DEATH
                        if player.is_dead {
                            death_timer += dt;
                            if death_timer > 3.0 {
                                // RESPAWN LOGIC (Copied)
                                spawn_found = false;
                                'respawn: for r in 0..300i32 {
                                    for x in -r..=r { for z in -r..=r {
                                        if x.abs() != r && z.abs() != r { continue; }
                                        for y in (0..150).rev() {
                                            let b = world.get_block(BlockPos{x, y, z});
                                            if b.is_solid() && !b.is_water() {
                                                player.respawn(); player.position = glam::Vec3::new(x as f32 + 0.5, y as f32 + 2.0, z as f32 + 0.5);
                                                spawn_found = true; death_timer = 0.0; break 'respawn;
                                            }
                                        }
}}
                                }
if !spawn_found { player.respawn(); player.position = glam::Vec3::new(0.0, 80.0, 0.0); death_timer = 0.0; }
                            }
                        } else {
                            player.update(dt, &world);
                            let (sin, cos) = player.rotation.x.sin_cos(); 
                            let (ysin, ycos) = player.rotation.y.sin_cos();
                            let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                            let ray_res = world.raycast(player.position + glam::Vec3::new(0.0, player.height*0.4, 0.0), dir, 5.0);
                            let current_target = ray_res.map(|(h, _)| h);
                            
                            if left_click && !player.inventory_open {
                                if let Some(hit) = current_target {
                                    if Some(hit) != breaking_pos {
                                        if breaking_pos.is_some() && break_grace_timer > 0.0 { break_grace_timer -= dt; } 
                                        else { breaking_pos = Some(hit); break_progress = 0.0; break_grace_timer = 0.5; }
                                    } else { break_grace_timer = 0.5; }
                                    
                                    if Some(hit) == breaking_pos {
                                        let blk = world.get_block(hit); 
                                        let tool = player.inventory.get_selected_item().unwrap_or(BlockType::Air);
                                        let is_correct_tool = tool.get_tool_class() == blk.get_best_tool_type();
                                        let speed = if is_correct_tool || blk.get_best_tool_type() == "none" { tool.get_tool_speed() } else { 1.0 };
                                        let h = blk.get_hardness();
                                        if h > 0.0 { break_progress += (speed / h) * dt; } else { break_progress = 1.1; }
                                        if break_progress >= 1.0 {
                                            if let Some(stack) = &mut player.inventory.slots[player.inventory.selected_hotbar_slot] {
                                                if stack.item.is_tool() {
                                                    let damage = if is_correct_tool { 1 } else { 2 };
                                                    if stack.durability > damage { stack.durability -= damage; }
                                                    else { player.inventory.slots[player.inventory.selected_hotbar_slot] = None; }
                                                }
                                            }
                                            let b_type = world.get_block(hit);
                                            let (tex, _, _) = b_type.get_texture_indices();
                                            for _ in 0..8 {
                                                renderer.particles.push(renderer::Particle {
                                                    pos: glam::Vec3::new(hit.x as f32 + 0.5, hit.y as f32 + 0.5, hit.z as f32 + 0.5),
                                                    vel: glam::Vec3::new((rand::random::<f32>() - 0.5) * 4.0, rand::random::<f32>() * 5.0, (rand::random::<f32>() - 0.5) * 4.0),
                                                    life: 1.0,
                                                    color_idx: tex,
                                                });
                                            }
                                            let c = world.break_block(hit);
                                            if let Some(net) = &network_mgr { net.send_packet(Packet::BlockUpdate { pos: hit, block: BlockType::Air }); }
                                            for (cx, cz) in c { renderer.update_chunk(cx, cz, &world); }
                                            breaking_pos = None; break_progress = 0.0;
                                        }
                                    }
                                } else { if breaking_pos.is_some() && break_grace_timer > 0.0 { break_grace_timer -= dt; } else { breaking_pos = None; break_progress = 0.0; } }
                            } else { if breaking_pos.is_some() && break_grace_timer > 0.0 { break_grace_timer -= dt; } else { breaking_pos = None; break_progress = 0.0; } }
                        }
                        world.update_entities(dt, &mut player);
                    }
                    renderer.break_progress = if breaking_pos.is_some() { break_progress } else { 0.0 };
                    renderer.update_camera(&player, win_size.0 as f32 / win_size.1 as f32);
                    match renderer.render(&world, &player, is_paused, cursor_pos, win_size.0, win_size.1) {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(win_size.0, win_size.1),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(_) => {},
                    }
                } else {
                    if let Err(_) = renderer.render_main_menu(&main_menu, win_size.0, win_size.1) { renderer.resize(win_size.0, win_size.1); }
                }
            }
            Event::AboutToWait => window.request_redraw(),
            _ => {}
        }
    }).unwrap();
}