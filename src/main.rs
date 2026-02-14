use winit::{
    event::{Event, WindowEvent, ElementState, DeviceEvent, MouseButton, MouseScrollDelta, KeyEvent},
    event_loop::EventLoop,
    window::{WindowBuilder, CursorGrabMode},
    keyboard::{KeyCode, PhysicalKey},
};
use std::sync::Arc;
use std::time::Instant;
use std::time::{SystemTime, UNIX_EPOCH};
use minecraft_clone::{Renderer, World, BlockType, BlockPos, Player, NetworkManager, GameState, MainMenu, MenuAction, AudioSystem, Rect, SettingsMenu};
use minecraft_clone::network::Packet;
use glam::Vec3;
use serde_json::json;
use std::fs;

// Audio system is now imported from lib.rs

// --- UI STRUCTURES ---
#[repr(C)] #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct UIElement { pub pos: [f32; 2], pub size: [f32; 2], pub tex_idx: u32, pub padding: u32 }

fn main() {
    minecraft_clone::utils::init_logger();
    let master_seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u32;
    let args: Vec<String> = std::env::args().collect();

    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title("Minecraft Rust Clone").with_maximized(true).build(&event_loop).unwrap());
    
    // Initialize Renderer immediately (No Pollster block needed if we map async correctly, but keeping simple)
    let window_arc = window.clone(); // Clone ARC for renderer
let mut renderer = pollster::block_on(Renderer::new(&window_arc));
// DIABOLICAL LIVE PERSISTENCE: Force reload into the exact previous position and state
    let mut start_pos = Vec3::new(0.0, 80.0, 0.0);
    let mut current_seed = master_seed;
    let mut was_playing = false;

    let save_path = "target/.live_state.json";
    if let Ok(data) = fs::read_to_string(save_path) {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&data) {
            current_seed = val["seed"].as_u64().unwrap_or(master_seed as u64) as u32;
            start_pos.x = val["x"].as_f64().unwrap_or(0.0) as f32;
            start_pos.y = val["y"].as_f64().unwrap_or(80.0) as f32;
            start_pos.z = val["z"].as_f64().unwrap_or(0.0) as f32;
            was_playing = val["was_playing"].as_bool().unwrap_or(false);
        }
    }

    let mut world = World::new(current_seed);
    let mut player = Player::new();
    player.position = start_pos;
    let mut last_persist = Instant::now();
    let mut accumulator = 0.0f32;
    const FIXED_TIME: f32 = 1.0 / 120.0; // 120Hz DIABOLICAL PHYSICS LOCK
    
    // --- GAME STATE ---
    // DIABOLICAL FIX: Start in Menu for instant-access. Atlas bakes on first load.
    let mut game_state = GameState::Menu;
    let mut load_step = 0;
    if was_playing {
        let _ = window.set_cursor_grab(CursorGrabMode::Locked);
        window.set_cursor_visible(false);
    }
    let mut main_menu = MainMenu::new_main();
    let mut pause_menu = MainMenu::new_pause();
    let mut settings_menu = SettingsMenu::new();
    let mut hosting_mgr = minecraft_clone::network::HostingManager::new();
    let mut network_mgr: Option<NetworkManager> = None;
    
    // If CLI args provided, jump straight to game
    if args.len() > 1 && args[1] == "--join-localhost" { 
        network_mgr = Some(NetworkManager::join("127.0.0.1:7878".to_string()));
        game_state = GameState::Playing;
    }

    // --- PLAYER & LOGIC STATE (Preserved from your code) ---

    
// --- SPAWN STATE ---
    let mut spawn_found = false;
    let mut breaking_pos: Option<BlockPos> = None;
    let mut break_progress = 0.0;
    let mut left_click = false;
    let mut net_timer = 0.0;
    let mut death_timer = 0.0;
    let mut is_paused = false;
    let mut cursor_pos = (0.0, 0.0);
    let mut win_size = (window.inner_size().width, window.inner_size().height);
    let window_clone = window.clone();
let audio = AudioSystem::new();
    let mut last_frame = Instant::now();
    let mut first_build_done = false;



    // Initial Cursor Logic
    let _ = window.set_cursor_grab(CursorGrabMode::None);
    window.set_cursor_visible(true);

    let _ = event_loop.run(move |event, elwt| {
        match event {
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => elwt.exit(),
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                renderer.resize(size.width, size.height);
                win_size = (size.width, size.height);
            },
Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } => {
                cursor_pos = (position.x, position.y);
                if game_state == GameState::Menu || is_paused || player.inventory_open || game_state == GameState::Settings {
                    let ndc_x = (position.x as f32 / win_size.0 as f32) * 2.0 - 1.0;
                    let ndc_y = 1.0 - (position.y as f32 / win_size.1 as f32) * 2.0;
                    for btn in &mut main_menu.buttons { btn.hovered = btn.rect.contains(ndc_x, ndc_y); }
                    // Update pause menu hover state
                    if is_paused {
                        for btn in &mut pause_menu.buttons { 
                            // Calculate button position based on new professional layout
                            let button_width = 0.4;
                            let button_height = 0.06;
                            let button_spacing = 0.08;
                            let panel_x = 0.0;
                            let panel_y = 0.0;
                            let start_y = panel_y - button_spacing/2.0;
                            let button_y = start_y - (btn.rect.y / button_spacing) * button_spacing;
                            
                            let btn_rect_x = panel_x - button_width/2.0;
                            let btn_rect_y = button_y - button_height/2.0;
                            let btn_rect_w = button_width;
                            let btn_rect_h = button_height;
                            
                            // Create a temporary rect for hover detection
                            let temp_rect = Rect { x: btn_rect_x, y: btn_rect_y, w: btn_rect_w, h: btn_rect_h };
                            btn.hovered = temp_rect.contains(ndc_x, ndc_y);
                        }
                    }
                    // Update settings menu hover state
                    if game_state == GameState::Settings {
                        for btn in &mut settings_menu.buttons { 
                            btn.hovered = btn.rect.contains(ndc_x, ndc_y);
                        }
                    }
                }
            },
            Event::DeviceEvent { event: DeviceEvent::MouseMotion { delta }, .. } => { 
                if game_state == GameState::Playing && !is_paused && !player.inventory_open { player.process_mouse(delta.0, delta.1); } 
            },
// --- MOUSE INPUT ---
Event::WindowEvent { event: WindowEvent::MouseInput { button, state, .. }, .. } => {
                let pressed = state == ElementState::Pressed;
                let ndc_x = (cursor_pos.0 as f32 / win_size.0 as f32) * 2.0 - 1.0;
                let ndc_y = 1.0 - (cursor_pos.1 as f32 / win_size.1 as f32) * 2.0;

                if (game_state == GameState::Menu || game_state == GameState::Multiplayer) && pressed && button == MouseButton::Left {
                    let mut action = None;
                    for btn in &main_menu.buttons { if btn.rect.contains(ndc_x, ndc_y) { action = Some(btn.action.clone()); break; } }
                    
                    if let Some(act) = action {
                        match act {
                            MenuAction::Singleplayer => {
                                world = World::new(master_seed);
                                renderer.rebuild_all_chunks(&world);
                                game_state = GameState::Loading; // Transition to loading bar
                                load_step = 0;
                                spawn_found = false;
                            },
                            MenuAction::Settings => {
                                game_state = GameState::Settings;
                            },
                            MenuAction::JoinMenu => {
                                game_state = GameState::Multiplayer;
                            },
                            MenuAction::JoinAddr(addr) => {
                                network_mgr = Some(NetworkManager::join(addr));
                                game_state = GameState::Playing;
                                spawn_found = false;
                            },
                            MenuAction::Host => {
                                hosting_mgr.init();
                                network_mgr = Some(NetworkManager::host("25565".to_string(), master_seed));
                                world = World::new(master_seed);
                                renderer.rebuild_all_chunks(&world);
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
                            MenuAction::Quit => {
                                if game_state == GameState::Multiplayer { 
                                    game_state = GameState::Menu; 
                                    main_menu = MainMenu::new_main(); // DIABOLICAL RECOVERY: Rebuild the main menu buttons
                                }
                                else { elwt.exit(); }
                            },
                            _ => {}
                        }

                        if game_state == GameState::Playing || game_state == GameState::Loading {
                            // Reset state for the transition
                            spawn_found = false;
                            breaking_pos = None;
                            break_progress = 0.0;
                            // Cursor will be locked after Loading finishes
                        }
                    }
                } else if is_paused && pressed && button == MouseButton::Left {
                    let mut action = None;
                    for (i, btn) in pause_menu.buttons.iter().enumerate() {
                        // Calculate button position based on new professional layout
                        let button_width = 0.4;
                        let button_height = 0.06;
                        let button_spacing = 0.08;
                        let panel_x = 0.0;
                        let panel_y = 0.0;
                        let start_y = panel_y - button_spacing/2.0;
                        let button_y = start_y - (i as f32 * button_spacing);
                        
                        let btn_rect_x = panel_x - button_width/2.0;
                        let btn_rect_y = button_y - button_height/2.0;
                        let btn_rect_w = button_width;
                        let btn_rect_h = button_height;
                        
                        // Create a temporary rect for click detection
                        let temp_rect = Rect { x: btn_rect_x, y: btn_rect_y, w: btn_rect_w, h: btn_rect_h };
                        if temp_rect.contains(ndc_x, ndc_y) { 
                            action = Some(&btn.action); 
                            break; 
                        }
                    }
                    if let Some(act) = action {
                        match act {
                            MenuAction::Resume => { is_paused = false; let _ = window_clone.set_cursor_grab(CursorGrabMode::Locked); window_clone.set_cursor_visible(false); },
                            MenuAction::Settings => { game_state = GameState::Settings; },
                            MenuAction::Quit => { game_state = GameState::Menu; is_paused = false; window_clone.set_cursor_visible(true); let _ = window_clone.set_cursor_grab(CursorGrabMode::None); },
                            _ => {}
                        }
                    }
                } else if game_state == GameState::Settings && pressed && button == MouseButton::Left {
                    let mut action = None;
                    for btn in &settings_menu.buttons {
                        if btn.rect.contains(ndc_x, ndc_y) { 
                            action = Some(&btn.action); 
                            break; 
                        }
                    }
                    if let Some(act) = action {
                        match act {
                            MenuAction::Quit => { game_state = GameState::Menu; },
                            _ => {}
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
                                            player.inventory.cursor_item = Some(minecraft_clone::engine::ItemStack::new(s.item, half)); 
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
                                        *slot = Some(minecraft_clone::engine::ItemStack::new(cursor.item, 1)); 
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
                                        player.inventory.cursor_item = Some(minecraft_clone::engine::ItemStack::new(curr.item, curr.count + o.count)); 
                                    } else { 
                                        player.inventory.cursor_item = Some(o); 
                                    }
player.inventory.craft(); 
                                    player.inventory.check_recipes(); 
                                    audio.play("click", false);
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
                                player.inventory.slots[player.inventory.selected_hotbar_slot] = Some(minecraft_clone::engine::ItemStack::new(BlockType::BucketWater, 1));
                                renderer.update_chunk(hit.x.div_euclid(16), hit.y.div_euclid(16), hit.z.div_euclid(16), &world);
                            } else if held_item == BlockType::BucketWater {
                                world.place_block(place, BlockType::Water);
                                player.inventory.slots[player.inventory.selected_hotbar_slot] = Some(minecraft_clone::engine::ItemStack::new(BlockType::BucketEmpty, 1));
                                renderer.update_chunk(place.x.div_euclid(16), place.y.div_euclid(16), place.z.div_euclid(16), &world);
                            } else if held_item.get_tool_class() == "hoe" && (targeted_block == BlockType::Grass || targeted_block == BlockType::Dirt) {
                                world.place_block(hit, BlockType::FarmlandDry);
                                renderer.update_chunk(place.x.div_euclid(16), place.y.div_euclid(16), place.z.div_euclid(16), &world);
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
                                                        renderer.update_chunk(neighbor_pos.x.div_euclid(16), neighbor_pos.y.div_euclid(16), neighbor_pos.z.div_euclid(16), &world);
                                                        break;
                                                    }
                                                }
                                            }
                                            let _c = world.place_block(place, actual_blk);
                                            let head_p = BlockPos { x: player.position.x as i32, y: (player.position.y + 1.5) as i32, z: player.position.z as i32 };
                                            let is_submerged = world.get_block(head_p).is_water();
                                            audio.play("place", is_submerged);
                                            player.inventory.remove_one_from_hand();
                                            if let Some(net) = &network_mgr { net.send_packet(Packet::BlockUpdate { pos: place, block: actual_blk }); }
                                            // ROOT FIX: Removed renderer.update_chunk loops to eliminate lag spikes. 
                                            // The Renderer will now detect 'mesh_dirty' and handle it off-thread.
                                        }
                                    }
                                }
                            }
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
                let _dt_frame = (now - last_frame).as_secs_f32().min(0.1);
                last_frame = now;

                if game_state == GameState::Loading {
                    // DIABOLICAL ASYNC BOOTLOADER: Max speed with zero main-thread stalls
                    match load_step {
                        0 => {
                            renderer.loading_message = "INITIALIZING CORE KERNEL...".to_string();
                            renderer.loading_progress = 0.05;
                            // Atlas is already baked in Renderer::new, skip to Stage 2
                            load_step = 2;
                        }
                        1 => {
                            load_step = 2;
                        }
                        2 => {
                            // Stage 2: Parallel Column Generation
                            // DIABOLICAL OPTIMIZATION: Generate only 4 columns per frame to keep OS responsive.
                            let columns_per_frame = 4; 
                            let current_col = world.chunks.len() / (minecraft_clone::engine::WORLD_HEIGHT as usize / 16);
                            
                            for i in 0..columns_per_frame {
                                let step = (current_col + i) as i32;
                                if step < 169 {
                                    world.bootstrap_terrain_step(step);
                                }
                            }
                            
                            renderer.loading_progress = 0.1 + (current_col as f32 / 169.0) * 0.4;
                            renderer.loading_message = format!("GENERATING TOPOLOGY... [STEP {}/169]", current_col);
                            
                            // Exit early logic: We only NEED a 3x3 radius to play immediately.
                            // This brings the "Playable" time down from 24s to ~3s.
                            if current_col >= 49 || current_col >= 168 { load_step = 3; }
                        }
                        3 => {
                            // Stage 3: Async Background Mesh Dispatch
                            renderer.loading_message = "DISPATCHING ASYNC MESHERS...".to_string();
                            let world_arc = Arc::new(world.clone());
                            let mut keys: Vec<_> = world.chunks.keys().cloned().collect();
                            // Priority Sort: Mesh spawn area first
                            keys.sort_by_key(|k| k.0 * k.0 + k.2 * k.2);

                            for key in keys {
                                if !renderer.chunk_meshes.contains_key(&key) && !renderer.pending_chunks.contains(&key) {
                                    renderer.pending_chunks.insert(key);
                                    let _ = renderer.mesh_tx.send((key.0, key.1, key.2, 0, world_arc.clone()));
                                }
                            }
                            load_step = 4;
                        }
                        4 => {
                            // Stage 4: RADICAL HABITABLE SCOUT
                            // This forces chunk generation for the spawn column to ensure we don't spawn in stone.
                            renderer.loading_message = "MAPPING HABITABLE BIOMES...".to_string();
                            if !spawn_found {
                                let mut scout_x = 0;
                                let mut scout_z = 0;
                                let mut found = false;
                                
                                // DIABOLICAL SPIRAL SEARCH
                                'outer: for r in 0..20i32 {
                                    for dx in -r..=r {
                                        for dz in -r..=r {
                                            if dx.abs() != r && dz.abs() != r { continue; }
                                            let wx = dx * 16;
                                            let wz = dz * 16;
                                            
                                            // Force column generation immediately
                                            let _noise = minecraft_clone::resources::NoiseGenerator::new(world.seed);
                                            for y_chunk in 0..8 {
                                                if !world.chunks.contains_key(&(dx, y_chunk, dz)) {
                                                    world.bootstrap_terrain_step(dx * 1337 + dz); // Dummy step for force gen
                                                }
                                            }

                                            let h = world.get_height_at(wx, wz);
                                            let blk = world.get_block(BlockPos { x: wx, y: h, z: wz });
                                            
                                            // Check for Dry Land (not water, not deep ocean)
                                            if h > minecraft_clone::engine::WATER_LEVEL + 2 && !blk.is_water() {
                                                scout_x = wx;
                                                scout_z = wz;
                                                found = true;
                                                break 'outer;
                                            }
                                        }
                                    }
                                }

                                if found {
                                    let final_h = world.get_height_at(scout_x, scout_z) as f32;
                                    player.position = glam::Vec3::new(scout_x as f32 + 0.5, final_h + 2.5, scout_z as f32 + 0.5);
                                    player.prev_position = player.position;
                                    player.velocity = glam::Vec3::ZERO; // Kill any falling momentum
                                    player.health = 10.0; // Set health to 10 hearts
                                    player.stasis = false; // RELEASE PLAYER FROM STASIS
                                    spawn_found = true;
                                    log::info!("✅ DIABOLICAL SPAWN SECURED: ({}, {}, {})", scout_x, final_h, scout_z);
                                } else {
                                    // Fallback to surface spawn
                                    let surface_y = world.get_height_at(0, 0) as f32;
                                    player.position = glam::Vec3::new(0.5, surface_y + 2.5, 0.5);
                                    for x in -2..=2 { for z in -2..=2 { world.place_block(BlockPos { x, y: surface_y as i32 - 2, z }, BlockType::Stone); } }
                                    player.health = 10.0; // Set health to 10 hearts
                                    spawn_found = true;
                                }
                            }
                            load_step = 5;
                        }
                        5 => {
                            // Stage 5: Radical Boot - Exit as soon as spawn is meshed
                            renderer.process_mesh_queue(); 

                            let total = world.chunks.len() as f32;
                            let remaining = renderer.pending_chunks.len() as f32;
                            let progress = (total - remaining) / (total + 0.001);
                            
                            renderer.loading_message = format!("OPTIMIZING GEOMETRY... {}%", (progress * 100.0) as u32);
                            renderer.loading_progress = 0.5 + progress * 0.45;
                            
                            // DIABOLICAL 7-SECOND BYPASS: Launch if local chunks (radius 2) are meshed.
                            // This gets the player in-game while distant chunks mesh in the background.
                            let mut local_done = true;
                            for dx in -2..=2 {
                                for dz in -2..=2 {
                                    for dy in 0..8 {
                                        if renderer.pending_chunks.contains(&(dx, dy, dz)) { local_done = false; }
                                    }
                                }
                            }

                            if local_done || renderer.pending_chunks.is_empty() {
                                // Transition to final fade step
                                load_step = 6;
                            }
                        }
                        _ => {
                            renderer.transition_alpha = (renderer.transition_alpha - _dt_frame * 1.5).max(0.0);
                            renderer.loading_progress = 1.0;
                            renderer.loading_message = "SYSTEMS NOMINAL. READY.".to_string();

                            if renderer.transition_alpha <= 0.0 {
                                let total_load_time = renderer.init_time.elapsed().as_secs_f32();
                                log::info!("╔════════════════════════════════════════════════════════════╗");
                                log::info!("║ 🚀 LOAD COMPLETED IN {:<5.2} SECONDS                      ║", total_load_time);
                                log::info!("╚════════════════════════════════════════════════════════════╝");
                                
                                // DIABOLICAL TRANSITION: Enter Play mode and capture cursor
                                game_state = GameState::Playing;
                                window.set_cursor_visible(false);
                                let _ = window.set_cursor_grab(winit::window::CursorGrabMode::Locked);
                                
                                // FORCE POSITION RE-SYNC
                                player.prev_position = player.position;
                                
                                renderer.transition_alpha = 1.0; // Reset for next load
                                first_build_done = true;
                            }
                        }
                    }
                    
                    if let Err(e) = renderer.render_loading_screen() { log::error!("Loading error: {:?}", e); }
                    window.request_redraw();
                    return; 
                }
                
                // ROOT CAUSE FIX: Decouple Physics from Framerate via Accumulator
                accumulator += _dt_frame.min(0.1); // Cap to 100ms to prevent spiral of death

                if game_state == GameState::Playing {
                    // 1. INFINITE GENERATION CALL (OPTIMIZED)
                    let p_cx = (player.position.x / 16.0).floor() as i32;
                    let p_cy = (player.position.y / 16.0).floor() as i32;
                    let p_cz = (player.position.z / 16.0).floor() as i32;
                    world.generate_one_chunk_around(p_cx, p_cy, p_cz, 4); // Reduced from 8

                    player.capture_state(); 

                    while accumulator >= FIXED_TIME {
                        let head_pos = BlockPos { 
                            x: player.position.x as i32, 
                            y: (player.position.y + 1.0) as i32, 
                            z: player.position.z as i32 
                        };
                        let is_cave = world.get_light_world(head_pos) < 6;
                        
                        player.update(&world, FIXED_TIME, &audio, is_cave);
                        world.update_entities(FIXED_TIME, &mut player);
                        accumulator -= FIXED_TIME;
                    }
                    
                    let alpha = accumulator / FIXED_TIME;
                    renderer.update_camera(&player, win_size.0 as f32 / win_size.1 as f32, alpha);

                    // DIABOLICAL FIRST-FRAME STABILITY
                    // DIABOLICAL FIRST-FRAME STABILITY
                    if !first_build_done {
                        renderer.rebuild_all_chunks(&world);
                        first_build_done = true;
                    }

                    // --- INFINITE GENERATION (OPTIMIZED) ---
                    let p_cx = (player.position.x / 16.0).floor() as i32;
                    let p_cz = (player.position.z / 16.0).floor() as i32;
                    let r_dist = 4; // Reduced from 6 for better performance
                    for dx in -r_dist..=r_dist {
                        for dz in -r_dist..=r_dist {
                            let target = (p_cx + dx, 0, p_cz + dz);
                            if !world.chunks.contains_key(&target) {
                                world.generate_one_chunk_around(p_cx + dx, 0, p_cz + dz, 1);
                            }
                        }
                    }

                    // --- DAY/NIGHT CYCLE ---
                    let _day_time = (renderer.start_time.elapsed().as_secs_f32() % 600.0) / 600.0;

// DIABOLICAL AUTO-SAVE: Save every 10 seconds to stop cargo-watch restart loops
                    if last_persist.elapsed().as_millis() >= 10000 {
                        let save_data = json!({
                            "seed": world.seed,
                            "x": player.position.x,
                            "y": player.position.y,
                            "z": player.position.z,
                            "was_playing": game_state == GameState::Playing,
                        });
if let Ok(contents) = serde_json::to_string(&save_data) {
                            let _ = fs::write("target/.live_state.json", contents);
                        }
                        last_persist = Instant::now();
                    }

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
                                                    player.respawn(); 
                                                    let surface_y = world.get_height_at(x, z) as f32;
                                                    player.position = glam::Vec3::new(x as f32 + 0.5, surface_y + 2.5, z as f32 + 0.5);
                                                    player.health = 10.0; // Set health to 10 hearts
                                                    spawn_found = true; break 'net_spawn;
                                                }
                                            }
                                        }}
                                    }
if !spawn_found { player.position = glam::Vec3::new(0.0, 80.0, 0.0); player.velocity = glam::Vec3::ZERO; }
                                },
                                Packet::PlayerMove { id, x, y, z, ry } => {

                                    if let Some(p) = world.remote_players.iter_mut().find(|p| p.id == id) { p.position = glam::Vec3::new(x,y,z); p.rotation = ry; } 
                                    else { world.remote_players.push(minecraft_clone::engine::RemotePlayer{id, position:glam::Vec3::new(x,y,z), rotation:ry}); }
                                },
                                Packet::BlockUpdate { pos, block } => { 
                                    let _c = world.place_block(pos, block); 
                                    // Renderer automatically picks up world.mesh_dirty flag
                                },
                                _ => {}
                            }
                        }
                        net_timer += _dt_frame; 
                        if net_timer > 0.05 { net_timer = 0.0; network.send_packet(Packet::PlayerMove { id: network.my_id, x: player.position.x, y: player.position.y, z: player.position.z, ry: player.rotation.y }); }
                    }

if !is_paused {
                        if player.spawn_timer > 0.0 { player.spawn_timer -= _dt_frame; player.velocity = glam::Vec3::ZERO; }
                        // DEATH
                        if player.is_dead {
                            death_timer += _dt_frame;
                            if death_timer > 3.0 {
                                spawn_found = false;
                                'respawn: for r in 0..300i32 {
                                    for x in -r..=r { for z in -r..=r {
                                        if x.abs() != r && z.abs() != r { continue; }
                                        for y in (0..150).rev() {
                                            let b = world.get_block(BlockPos{x, y, z});
                                            if b.is_solid() && !b.is_water() {
                                                player.respawn(); 
                                                let surface_y = world.get_height_at(x, z) as f32;
                                                player.position = glam::Vec3::new(x as f32 + 0.5, surface_y + 2.5, z as f32 + 0.5);
                                                player.health = 10.0; // Set health to 10 hearts
                                                spawn_found = true; death_timer = 0.0; break 'respawn;
                                            }
                                        }
                                    }}
                                }
                                if !spawn_found { 
                                    player.respawn(); 
                                    let surface_y = world.get_height_at(0, 0) as f32;
                                    player.position = glam::Vec3::new(0.5, surface_y + 2.5, 0.5);
                                    player.health = 10.0; // Set health to 10 hearts
                                    death_timer = 0.0; 
                                }
                            }
                        } else {
                            // Mining logic still runs per-frame for responsiveness
                            let (sin, cos) = player.rotation.x.sin_cos(); 
                            let (ysin, ycos) = player.rotation.y.sin_cos();
                            let dir = glam::Vec3::new(ycos * cos, sin, ysin * cos).normalize();
                            let ray_res = world.raycast(player.position + glam::Vec3::new(0.0, player.height*0.4, 0.0), dir, 5.0);
                            let current_target = ray_res.map(|(h, _)| h);
                            
                            if left_click && !player.inventory_open {
                                if let Some(hit) = current_target {
                                    if Some(hit) != breaking_pos {
                                        breaking_pos = Some(hit); 
                                        break_progress = 0.0; 
                                    }
                                    
                                    if Some(hit) == breaking_pos {
                                        let blk = world.get_block(hit); 
                                        let tool = player.inventory.get_selected_item().unwrap_or(BlockType::Air);
                                        let is_correct_tool = tool.get_tool_class() == blk.get_best_tool_type();
                                        let speed = if is_correct_tool || blk.get_best_tool_type() == "none" { tool.get_tool_speed() } else { 1.0 };
                                        let h = blk.get_hardness();
                                        if h > 0.0 { break_progress += (speed / h) * _dt_frame; } else { break_progress = 1.1; }
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
                                                renderer.particles.push(minecraft_clone::graphics::Particle {
                                                    pos: glam::Vec3::new(hit.x as f32 + 0.5, hit.y as f32 + 0.5, hit.z as f32 + 0.5),
                                                    vel: glam::Vec3::new((rand::random::<f32>() - 0.5) * 4.0, rand::random::<f32>() * 5.0, (rand::random::<f32>() - 0.5) * 4.0),
                                                    life: 1.0, color_idx: tex,
                                                });
                                            }
                                            let s_type = match b_type {
                                                BlockType::Grass | BlockType::Leaves | BlockType::TallGrass => "grass",
                                                BlockType::Stone | BlockType::Cobblestone | BlockType::CoalOre => "stone",
                                                BlockType::Sand | BlockType::Gravel => "sand",
                                                _ => "break",
                                            };
                                            let head_p = BlockPos { x: player.position.x as i32, y: (player.position.y + 1.5) as i32, z: player.position.z as i32 };
                                            let is_submerged = world.get_block(head_p).is_water();
                                            audio.play(s_type, is_submerged);
                                            let _c = world.break_block(hit);
                                            if let Some(net) = &network_mgr { net.send_packet(Packet::BlockUpdate { pos: hit, block: BlockType::Air }); }
                                            breaking_pos = None; break_progress = 0.0;
                                        }
                                    }
                                } else { breaking_pos = None; break_progress = 0.0; }
                            } else { breaking_pos = None; break_progress = 0.0; }
                    }
                    }
                    renderer.break_progress = if breaking_pos.is_some() { break_progress } else { 0.0 };
                    
                    let result = if game_state == GameState::Settings {
                        renderer.render_settings_menu(&settings_menu, win_size.0, win_size.1)
                    } else if is_paused {
                        renderer.render_pause_menu(&pause_menu, &world, &player, cursor_pos, win_size.0, win_size.1)
                    } else {
                        renderer.render_game(&world, &player, is_paused, cursor_pos, win_size.0, win_size.1)
                    };

                    // RADICAL SYNC: Clear priority flags ONLY after the render call consumes them.
                    world.mesh_dirty = false;
                    world.dirty_chunks.clear();

                    match result {
                        Ok(_) => {}
                        Err(wgpu::SurfaceError::Lost) => renderer.resize(win_size.0, win_size.1),
                        Err(wgpu::SurfaceError::OutOfMemory) => elwt.exit(),
                        Err(_) => {},
                    }
                } else if game_state == GameState::Multiplayer {
                    if let Err(_) = renderer.render_multiplayer_menu(&mut main_menu, &hosting_mgr, win_size.0, win_size.1) { renderer.resize(win_size.0, win_size.1); }
                } else {
                    if let Err(_) = renderer.render_main_menu(&main_menu, win_size.0, win_size.1) { renderer.resize(win_size.0, win_size.1); }
                }
            }
            Event::AboutToWait => {
                // DIABOLICAL THREADING: Only process world-gen and cursor logic if we are actually in the game.
                if game_state == GameState::Playing && !is_paused && !player.inventory_open {
                    // DIABOLICAL SPAWN SAFETY: If player falls into void/water on load (Y < 20), teleport to surface
                    if player.position.y < 20.0 {
                        let surface_y = world.get_height_at(player.position.x as i32, player.position.z as i32) as f32;
                        player.position.y = surface_y + 2.5; // Spawn on surface
                        player.velocity = glam::Vec3::ZERO; // Kill momentum
                        player.health = 10.0; // Set health to 10 hearts
                    }

                    let p_cx = (player.position.x / 16.0).floor() as i32;
                    let p_cy = (player.position.y / 16.0).floor() as i32;
                    let p_cz = (player.position.z / 16.0).floor() as i32;
                    world.generate_one_chunk_around(p_cx, p_cy, p_cz, 8);

                    // Ensure cursor state is always correct
                    let _noise = minecraft_clone::resources::NoiseGenerator::new(world.seed); // DIABOLICAL FIX: Prefix unused var
                    let _ = window.set_cursor_grab(CursorGrabMode::Locked);
                    window.set_cursor_visible(false);
                }
                window.request_redraw();
            },
            _ => {}
}
    });
}