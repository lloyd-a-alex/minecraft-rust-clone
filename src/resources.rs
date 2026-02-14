#[allow(dead_code)]
pub struct TextureAtlas {
    pub data: Vec<u8>,
    pub size: u32,
    pub grid_size: u32,
}

impl TextureAtlas {
pub fn new() -> Self {
        let atlas_width = 512;
        let atlas_height = 512;
        let total_pixels = atlas_width * atlas_height;
        let rgba_bytes = (total_pixels * 4) as usize;
        let mut data = vec![0u8; rgba_bytes];

        let block_size = 16;
        let grid_width_in_blocks = atlas_width / block_size;

// --- 1. BASIC TERRAIN (Mellowed Colors) ---
        Self::generate_noise(&mut data, block_size, atlas_width, 0, [115, 155, 105], 10); // Grass Top
        Self::generate_grass_side(&mut data, block_size, atlas_width, 1);                 // Grass Side
        // --- NEW PLANKS ---
        Self::generate_planks(&mut data, block_size, atlas_width, 14, [180, 130, 80]);    // Oak Planks
        Self::generate_planks(&mut data, block_size, atlas_width, 170, [61, 46, 32]);     // Spruce Planks
        Self::generate_planks(&mut data, block_size, atlas_width, 171, [230, 225, 220]);   // Birch Planks
        Self::generate_dirt(&mut data, block_size, atlas_width, 2);                      // Dirt
        Self::generate_noise(&mut data, block_size, atlas_width, 3, [140, 140, 140], 12); // Stone
        Self::generate_wood_side(&mut data, block_size, atlas_width, 4, [120, 100, 80]); // Wood Side
        Self::generate_leaves_fancy(&mut data, block_size, atlas_width, 5);              // Leaves
        Self::generate_noise(&mut data, block_size, atlas_width, 6, [240, 245, 250], 5);  // Snow
        Self::generate_noise(&mut data, block_size, atlas_width, 7, [215, 205, 175], 8);  // Sand
        Self::generate_bedrock(&mut data, block_size, atlas_width, 8);                   // Bedrock
        Self::generate_liquid(&mut data, block_size, atlas_width, 9, [80, 120, 180]);    // Water

        // --- 2. ORES (Stone base + colored flecks) ---
        Self::generate_ore(&mut data, block_size, atlas_width, 10, [20, 20, 20]);    // Coal
        Self::generate_ore(&mut data, block_size, atlas_width, 11, [210, 180, 140]); // Iron
        Self::generate_ore(&mut data, block_size, atlas_width, 12, [255, 215, 0]);   // Gold
        Self::generate_ore(&mut data, block_size, atlas_width, 13, [0, 255, 255]);   // Diamond
        Self::generate_ore(&mut data, block_size, atlas_width, 22, [255, 0, 0]);     // Redstone
        Self::generate_ore(&mut data, block_size, atlas_width, 23, [20, 40, 180]);   // Lapis

        // --- 3. WOOD PRODUCTS ---
        // Planks (Horizontal Boards)
        Self::generate_planks(&mut data, block_size, atlas_width, 14, [180, 130, 80]);
        // Sticks (Item)
        Self::generate_stick(&mut data, block_size, atlas_width, 15);
        // Cobblestone (Distinct Pattern)
        Self::generate_cobblestone(&mut data, block_size, atlas_width, 16);
        
        // --- 4. UTILITY BLOCKS ---
        // Torch
        Self::generate_torch(&mut data, block_size, atlas_width, 20);
        // Crafting Table
        Self::generate_crafting_top(&mut data, block_size, atlas_width, 21);
        Self::generate_crafting_side(&mut data, block_size, atlas_width, 25);
        // Furnace
        Self::generate_cobblestone(&mut data, block_size, atlas_width, 26); // Top/Side generic
        Self::generate_furnace_front(&mut data, block_size, atlas_width, 27, false); // Front Off
        Self::generate_furnace_front(&mut data, block_size, atlas_width, 201, true); // Front On (Fire)
        // Chest
        Self::generate_planks(&mut data, block_size, atlas_width, 28, [160, 100, 50]); // Top (Darker planks)
        Self::generate_chest_front(&mut data, block_size, atlas_width, 29);

        // --- 5. ENVIRONMENT & DECOR ---
        Self::generate_noise(&mut data, block_size, atlas_width, 30, [128, 128, 128], 40); // Gravel (Rougher stone)
        Self::generate_noise(&mut data, block_size, atlas_width, 31, [158, 164, 176], 10); // Clay (Smooth)
        Self::generate_planks(&mut data, block_size, atlas_width, 32, [210, 200, 160]);    // Sandstone (Smooth layered)
        Self::generate_planks(&mut data, block_size, atlas_width, 33, [210, 200, 160]);    // Sandstone Side
        Self::generate_obsidian(&mut data, block_size, atlas_width, 34);
        
        // Cactus
        Self::generate_cactus(&mut data, block_size, atlas_width, 35, true);  // Top
        Self::generate_cactus(&mut data, block_size, atlas_width, 36, false); // Side
        
        // Flowers / Plants
        Self::generate_flower(&mut data, block_size, atlas_width, 37, [230, 0, 0]); // Rose
        Self::generate_flower(&mut data, block_size, atlas_width, 38, [230, 230, 0]); // Dandelion
        Self::generate_deadbush(&mut data, block_size, atlas_width, 39);
        Self::generate_tallgrass(&mut data, block_size, atlas_width, 45);
        Self::generate_sugarcane(&mut data, block_size, atlas_width, 46);
        Self::generate_sapling(&mut data, block_size, atlas_width, 47);
        
        // Glass
        Self::generate_glass(&mut data, block_size, atlas_width, 48);
        // Bookshelf
        Self::generate_bookshelf(&mut data, block_size, atlas_width, 49);
        
        // TNT & Pumpkin & Melon
        Self::generate_tnt_top(&mut data, block_size, atlas_width, 50);
        Self::generate_tnt_side(&mut data, block_size, atlas_width, 51);
        Self::generate_noise(&mut data, block_size, atlas_width, 52, [200, 110, 0], 10); // Pumpkin Top
        Self::generate_pumpkin_face(&mut data, block_size, atlas_width, 53); // Face
        Self::generate_noise(&mut data, block_size, atlas_width, 54, [50, 180, 50], 10); // Melon Top
        Self::generate_melon_side(&mut data, block_size, atlas_width, 55);
        
        // Bricks
        Self::generate_bricks(&mut data, block_size, atlas_width, 56);
        Self::generate_mossy(&mut data, block_size, atlas_width, 57);

        // --- 6. NEW WOOD TYPES ---
        // Spruce
        Self::generate_wood_side(&mut data, block_size, atlas_width, 202, [61, 46, 32]);
        Self::generate_leaves(&mut data, block_size, atlas_width, 203, [40, 70, 40]);
        // Birch
        Self::generate_birch_side(&mut data, block_size, atlas_width, 204);
        Self::generate_leaves(&mut data, block_size, atlas_width, 205, [100, 140, 100]);

        // --- 7. ITEMS (Simple Pixel Art) ---
        Self::generate_generic(&mut data, block_size, atlas_width, 40, [100, 50, 0]);    // Stick
        Self::generate_generic(&mut data, block_size, atlas_width, 41, [20, 20, 20]);    // Coal
        Self::generate_ingot(&mut data, block_size, atlas_width, 42, [180, 180, 180]); // Iron Ingot
        Self::generate_ingot(&mut data, block_size, atlas_width, 43, [255, 215, 0]);   // Gold Ingot
        Self::generate_gem(&mut data, block_size, atlas_width, 44, [0, 255, 255]);     // Diamond
        
        // Food
        Self::generate_generic(&mut data, block_size, atlas_width, 80, [150, 200, 50]); // Wheat
        Self::generate_generic(&mut data, block_size, atlas_width, 81, [180, 130, 50]); // Bread
        Self::generate_generic(&mut data, block_size, atlas_width, 82, [220, 20, 20]);  // Apple
        Self::generate_meat(&mut data, block_size, atlas_width, 83, [240, 140, 140]);   // Porkchop
        Self::generate_meat(&mut data, block_size, atlas_width, 84, [200, 100, 60]);    // Cooked Porkchop

        // Tools
        Self::generate_tool(&mut data, block_size, atlas_width, 21, [130, 90, 50]);   // Wood Pickaxe
        Self::generate_tool(&mut data, block_size, atlas_width, 22, [100, 100, 100]); // Stone Pickaxe
        Self::generate_tool(&mut data, block_size, atlas_width, 23, [220, 220, 220]); // Iron Pickaxe
        Self::generate_tool(&mut data, block_size, atlas_width, 24, [255, 240, 50]);  // Gold Pickaxe
        Self::generate_tool(&mut data, block_size, atlas_width, 25, [80, 240, 255]);  // Diamond Pickaxe

        for i in 26..=30 { Self::generate_tool(&mut data, block_size, atlas_width, i, [120, 120, 120]); } // Axes
        for i in 31..=35 { Self::generate_tool(&mut data, block_size, atlas_width, i, [200, 200, 200]); } // Shovels
        for i in 36..=40 { Self::generate_tool(&mut data, block_size, atlas_width, i, [255, 215, 0]); }   // Swords

        // --- 8. UI ELEMENTS ---
        const UI_HOTBAR_SLOT: u32 = 240;
        const UI_SELECTION: u32 = 241;
        const UI_HEART: u32 = 242;
        const UI_BUBBLE: u32 = 243;
        const UI_BAR: u32 = 244;

        Self::clear_tile(&mut data, block_size, atlas_width, UI_HOTBAR_SLOT);
        Self::generate_hotbar_slot(&mut data, block_size, atlas_width, UI_HOTBAR_SLOT);
        Self::clear_tile(&mut data, block_size, atlas_width, UI_SELECTION);
        Self::generate_selection(&mut data, block_size, atlas_width, UI_SELECTION);
        Self::clear_tile(&mut data, block_size, atlas_width, UI_HEART);
        Self::generate_heart(&mut data, block_size, atlas_width, UI_HEART);
        Self::clear_tile(&mut data, block_size, atlas_width, UI_BUBBLE);
        Self::generate_bubble_data(&mut data, block_size, atlas_width, UI_BUBBLE);
        Self::clear_tile(&mut data, block_size, atlas_width, UI_BAR);
Self::generate_ui_bar_data(&mut data, block_size, atlas_width, UI_BAR);
        Self::generate_generic(&mut data, block_size, atlas_width, 245, [255, 255, 0]); // Yellow Bar
        Self::generate_generic(&mut data, block_size, atlas_width, 246, [255, 0, 0]);   // Red Bar

        Self::generate_font(&mut data, block_size, atlas_width, 300);

        // --- 9. BREAKING CRACKS (Indices 210-219) ---
        for i in 0..10 { Self::generate_cracks(&mut data, block_size, atlas_width, 210 + i, i as f32 / 9.0); }

        // --- 10. WHEAT CROPS (Indices 220-227) ---
        for i in 0..8 { Self::generate_wheat_stage(&mut data, block_size, atlas_width, 220 + i, i); }

// --- 11. CLOUDS (Index 228) ---
        Self::generate_noise(&mut data, block_size, atlas_width, 228, [255, 255, 255], 10);

        // --- 12. SPECIALTY BLOCKS ---
        Self::generate_generic(&mut data, block_size, atlas_width, 120, [255, 220, 0]); // Gold Block
        Self::generate_generic(&mut data, block_size, atlas_width, 121, [230, 230, 230]); // Iron Block
        Self::generate_generic(&mut data, block_size, atlas_width, 122, [100, 255, 255]); // Diamond Block
        Self::generate_dirt(&mut data, block_size, atlas_width, 123); // Farmland Dry
        Self::generate_noise(&mut data, block_size, atlas_width, 124, [60, 40, 20], 5); // Farmland Wet

        // --- 13. TOOLS (HOES/BUCKETS) ---
        for i in 41..=45 { Self::generate_tool(&mut data, block_size, atlas_width, i, [150, 150, 150]); } // Hoes
        Self::generate_bucket(&mut data, block_size, atlas_width, 46, false); // Bucket Empty
        Self::generate_bucket(&mut data, block_size, atlas_width, 47, true);  // Bucket Water

        // --- 12. UI BUTTONS ---
        Self::generate_button(&mut data, block_size, atlas_width, 250, false); // Normal
        Self::generate_button(&mut data, block_size, atlas_width, 251, true);  // Hovered
TextureAtlas { data, size: block_size, grid_size: grid_width_in_blocks }
    }

    // --- 9. UI BUTTONS (Helper) ---
    fn generate_button(data: &mut [u8], size: u32, w: u32, idx: u32, hovered: bool) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let base_color = if hovered { [120, 120, 160] } else { [60, 60, 60] };
        let border_light = if hovered { [160, 160, 200] } else { [100, 100, 100] };
        let border_dark = if hovered { [80, 80, 120] } else { [30, 30, 30] };

        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let mut c = base_color;
                if x == 0 || y == 0 { c = border_light; }
                else if x == size - 1 || y == size - 1 { c = border_dark; }

                p[i] = c[0]; p[i+1] = c[1]; p[i+2] = c[2]; p[i+3] = 255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    // --- GENERATION HELPERS ---

    fn clear_tile(data: &mut [u8], blocksize: u32, atlaswidth: u32, grididx: u32) {
        let blocks_per_row = atlaswidth / blocksize;
        let gridx = grididx % blocks_per_row;
        let gridy = grididx / blocks_per_row;
        let basex = gridx * blocksize;
        let basey = gridy * blocksize;
        for y in 0..blocksize {
            for x in 0..blocksize {
                let dstidx = ((basey + y) * atlaswidth + (basex + x)) as usize * 4;
                if dstidx + 3 < data.len() {
                    data[dstidx] = 0; data[dstidx + 1] = 0; data[dstidx + 2] = 0; data[dstidx + 3] = 0;
                }
            }
        }
    }

fn place_texture(data: &mut [u8], block_size: u32, atlas_width: u32, grid_idx: u32, pixels: &[u8]) {
        let blocks_per_row = atlas_width / block_size;
        let grid_x = grid_idx % blocks_per_row;
        let grid_y = grid_idx / blocks_per_row;
        let base_x = grid_x * block_size;
        let base_y = grid_y * block_size;

// DIABOLICAL FIX: Always wipe the destination first
        let blocks_per_row = atlas_width / block_size;
        let grid_x_clear = grid_idx % blocks_per_row;
        let grid_y_clear = grid_idx / blocks_per_row;
        let base_x_clear = grid_x_clear * block_size;
        let base_y_clear = grid_y_clear * block_size;
        for cy in 0..block_size {
            for cx in 0..block_size {
                let d_idx = ((base_y_clear + cy) * atlas_width + (base_x_clear + cx)) as usize * 4;
                data[d_idx] = 0; data[d_idx+1] = 0; data[d_idx+2] = 0; data[d_idx+3] = 0;
            }
        }

        for y in 0..block_size {
            for x in 0..block_size {
                let src_idx = ((y * block_size + x) * 4) as usize;
                let dst_idx = ((base_y + y) * atlas_width + (base_x + x)) as usize * 4;
                if src_idx + 3 < pixels.len() && dst_idx + 3 < data.len() {
                    let alpha = pixels[src_idx + 3];
                    if alpha > 0 {
                        data[dst_idx] = pixels[src_idx];
                        data[dst_idx + 1] = pixels[src_idx + 1];
                        data[dst_idx + 2] = pixels[src_idx + 2];
                        data[dst_idx + 3] = alpha;
                    }
                }
            }
        }
    }

    // --- PROCEDURAL TEXTURES ---

fn generate_noise(data: &mut [u8], size: u32, w: u32, idx: u32, base_col: [u8; 3], variance: i32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            // Pseudo-random based on index
            let r_offset = (i % 7 * 13 % (variance as u32 * 2 + 1)) as i32 - variance;
            let g_offset = (i % 5 * 17 % (variance as u32 * 2 + 1)) as i32 - variance;
            let b_offset = (i % 3 * 19 % (variance as u32 * 2 + 1)) as i32 - variance;
            
            let base = (i as usize) * 4;
            p[base] = (base_col[0] as i32 + r_offset).clamp(0,255) as u8;
            p[base+1] = (base_col[1] as i32 + g_offset).clamp(0,255) as u8;
            p[base+2] = (base_col[2] as i32 + b_offset).clamp(0,255) as u8;
            p[base+3] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_dirt(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                // Base brown
                let mut r = 139; let mut g = 69; let mut b = 19;
                
                // "Clumps" logic - larger noise pattern
                if ((x / 2) + (y / 2)) % 3 == 0 {
                    r = 110; g = 55; b = 15; // Darker clump
                } else if (x + y * 3) % 7 == 0 {
                    r = 160; g = 90; b = 40; // Lighter speck
                }
                
                // Minor noise
                let var = ((x * 13 ^ y * 23) % 15) as i32 - 7;
                p[i] = (r as i32 + var).clamp(0, 255) as u8;
                p[i+1] = (g as i32 + var).clamp(0, 255) as u8;
                p[i+2] = (b as i32 + var).clamp(0, 255) as u8;
                p[i+3] = 255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_leaves_fancy(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                
                // Variegated pattern: Checkerboard-ish clusters
                let cluster = ((x / 2) + (y / 2)) % 2 == 0;
                
                if (x % 4 == 0 && y % 4 != 0) || (y % 4 == 0 && x % 4 != 0) {
                    p[i+3] = 0; // Transparent holes for "fancy" graphics look
                } else {
                    if cluster {
                        p[i] = 40; p[i+1] = 100; p[i+2] = 40; // Dark Green
                    } else {
                        p[i] = 70; p[i+1] = 140; p[i+2] = 70; // Lighter Green
                    }
                    p[i+3] = 255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
fn generate_grass_side(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                // Dirt base
                let d_var = ((x + y) % 20) as i32 - 10;
                p[i] = (139 + d_var).clamp(0,255) as u8;
                p[i+1] = (69 + d_var).clamp(0,255) as u8;
                p[i+2] = (19 + d_var).clamp(0,255) as u8;
                p[i+3] = 255;

                // Grass overlay (Top 3 pixels + random drips)
                let grass_depth = 3 + (x % 3 == 0) as u32 + (x % 7 == 0) as u32;
                if y < grass_depth {
                    let g_var = ((x * y) % 20) as i32 - 10;
                    p[i] = (100 + g_var).clamp(0,255) as u8;
                    p[i+1] = (170 + g_var).clamp(0,255) as u8;
                    p[i+2] = (80 + g_var).clamp(0,255) as u8;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_wood_side(data: &mut [u8], size: u32, w: u32, idx: u32, col: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                // Vertical streaks
                let streak = if x % 4 == 0 || x % 7 == 0 { 20 } else { 0 };
                let noise = (y % 3) * 5;
                let val = (streak + noise) as i32;
                p[i] = (col[0] as i32 - val).clamp(0,255) as u8;
                p[i+1] = (col[1] as i32 - val).clamp(0,255) as u8;
                p[i+2] = (col[2] as i32 - val).clamp(0,255) as u8;
                p[i+3] = 255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
fn generate_birch_side(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let noise = ((x * 17 + y * 31) % 15) as i32;
                p[i] = (230 + noise) as u8; p[i+1] = (225 + noise) as u8; p[i+2] = (220 + noise) as u8; p[i+3] = 255;
                // Dark knots (More organic)
                if (y % 7 == 0 && x > 4 && x < 12) || (y % 13 == 0 && x < 6) {
                    p[i]=40; p[i+1]=40; p[i+2]=40;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

#[allow(dead_code)]
    fn generate_spruce_wood(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let grain = ((y * 11 + x * 3) % 10) as i32;
                p[i] = (60 - grain) as u8; p[i+1] = (45 - grain) as u8; p[i+2] = (35 - grain) as u8; p[i+3] = 255;
                if x % 8 == 0 { p[i]=40; p[i+1]=30; p[i+2]=25; } // Bark texture
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_leaves(data: &mut [u8], size: u32, w: u32, idx: u32, col: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size*size {
            let base = (i * 4) as usize;
            if i % 3 == 0 { // Transparent holes
                p[base+3] = 0;
            } else {
                let var = (i % 7 * 10) as i32 - 20;
                p[base] = (col[0] as i32 + var).clamp(0,255) as u8;
                p[base+1] = (col[1] as i32 + var).clamp(0,255) as u8;
                p[base+2] = (col[2] as i32 + var).clamp(0,255) as u8;
                p[base+3] = 255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_planks(data: &mut [u8], size: u32, w: u32, idx: u32, col: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let plank_h = size / 4;
                let is_gap = y % plank_h == 0;
                if is_gap {
                    p[i] = col[0]/2; p[i+1] = col[1]/2; p[i+2] = col[2]/2; p[i+3] = 255;
                } else {
                    let grain = (x % 7) as i32 * 5;
                    p[i] = (col[0] as i32 + grain).clamp(0,255) as u8;
                    p[i+1] = (col[1] as i32 + grain).clamp(0,255) as u8;
                    p[i+2] = (col[2] as i32 + grain).clamp(0,255) as u8;
                    p[i+3] = 255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_cobblestone(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                // Roughly brick pattern
                let bx = x % 8; let by = y % 8;
                let border = bx == 0 || by == 0;
                let val = if border { 80 } else { 130 + (bx+by) as u8 * 3 };
                p[i] = val; p[i+1] = val; p[i+2] = val; p[i+3] = 255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_bricks(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            let offset = if (y / 4) % 2 == 0 { 0 } else { 4 };
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let border = (x + offset) % 8 == 0 || y % 4 == 0;
                if border {
                    p[i] = 200; p[i+1] = 200; p[i+2] = 200; p[i+3] = 255; // Mortar
                } else {
                    p[i] = 160; p[i+1] = 60; p[i+2] = 50; p[i+3] = 255; // Brick Red
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_bedrock(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size*size {
            let val = if i % 3 == 0 || i % 7 == 0 { 20 } else { 100 };
            let base = (i as usize) * 4;
            p[base] = val; p[base+1] = val; p[base+2] = val; p[base+3] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }
fn generate_ore(data: &mut [u8], size: u32, w: u32, idx: u32, ore_col: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        // 1. Gritty Base Stone
        for i in 0..size*size {
            let x = i % size;
            let y = i / size;
            let noise = ((x * 7 + y * 13) % 16) as i32 - 8;
            let val = (120 + noise) as u8;
            let base = (i as usize) * 4;
            p[base] = val; p[base+1] = val; p[base+2] = val; p[base+3] = 255;
        }
        // 2. Crystalline Clusters with Specular Highlights
        for y in 1..size-1 {
            for x in 1..size-1 {
                // Pseudo-random cluster trigger
                if (x as u32 * 713 + y as u32 * 911) % 100 < 12 {
                    let i = ((y * size + x) * 4) as usize;
                    
                    // The "Shine": Top-left pixels of a cluster are brighter
                    let is_top_left = (x as u32 * 713 + (y-1) as u32 * 911) % 100 >= 12;
                    let shine = if is_top_left { 40 } else { -20 };
                    
                    p[i] = (ore_col[0] as i32 + shine).clamp(0, 255) as u8;
                    p[i+1] = (ore_col[1] as i32 + shine).clamp(0, 255) as u8;
                    p[i+2] = (ore_col[2] as i32 + shine).clamp(0, 255) as u8;

                    // Small neighboring fleck for organic feel
                    let ni = (((y+1) * size + (x+1)) * 4) as usize;
                    if ni + 3 < p.len() {
                        p[ni] = ore_col[0]; p[ni+1] = ore_col[1]; p[ni+2] = ore_col[2];
                    }
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
fn generate_obsidian(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size*size {
            let val = 20 + (i % 5 * 5) as u8;
            let base = (i as usize) * 4;
            p[base] = val; p[base+1] = 0; p[base+2] = val + 20; p[base+3] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_liquid(data: &mut [u8], size: u32, w: u32, idx: u32, col: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size*size {
             let base = (i*4) as usize;
             p[base] = col[0]; p[base+1] = col[1]; p[base+2] = col[2]; p[base+3] = 200; // Semi-transparent
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_glass(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y*size+x)*4) as usize;
                let border = x==0 || y==0 || x==size-1 || y==size-1;
                let streak = (x+y) % 9 == 0 && x > 2 && x < 14;
                if border {
                    p[i]=220; p[i+1]=255; p[i+2]=255; p[i+3]=255;
                } else if streak {
                     p[i]=240; p[i+1]=255; p[i+2]=255; p[i+3]=150;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_crafting_top(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y*size+x)*4) as usize;
                let is_border = x == 0 || y == 0 || x == size-1 || y == size-1;
                let is_grid = (x == 5 || x == 10) || (y == 5 || y == 10);
                
                if is_border {
                    p[i]=80; p[i+1]=50; p[i+2]=20; // Dark frame
                } else if is_grid && x > 2 && x < 13 && y > 2 && y < 13 {
                    p[i]=60; p[i+1]=40; p[i+2]=10; // Grid lines
                } else {
                    p[i]=160; p[i+1]=120; p[i+2]=70; // Fresh wood
                }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_crafting_side(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y*size+x)*4) as usize;
                // Rich Wood background with grain
                let grain = if x % 3 == 0 { -10 } else { 0 };
                p[i] = (140 + grain) as u8; p[i+1] = (100 + grain) as u8; p[i+2] = (50 + grain) as u8; p[i+3] = 255;
                
                // "Hammer & Saw" Silhouette
                let is_hammer_head = y == 5 && x > 4 && x < 10;
                let is_hammer_handle = x == 7 && y > 5 && y < 12;
                let is_saw = x == 3 && y > 4 && y < 12 && y % 2 == 0;
                
                if is_hammer_head || is_saw {
                    p[i]=50; p[i+1]=50; p[i+2]=50; // Iron
                } else if is_hammer_handle {
                    p[i]=100; p[i+1]=70; p[i+2]=40; // Tool wood
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_furnace_front(data: &mut [u8], size: u32, w: u32, idx: u32, active: bool) {
         let mut p = vec![0u8; (size * size * 4) as usize];
         for y in 0..size {
             for x in 0..size {
                 let i = ((y*size+x)*4) as usize;
                 let border = x==0 || y==0 || x==size-1 || y==size-1;
                 if border {
                     p[i]=100; p[i+1]=100; p[i+2]=100; p[i+3]=255;
                 } else if x > 3 && x < 12 && y > 3 && y < 12 {
                     if active { p[i]=255; p[i+1]=100; p[i+2]=0; p[i+3]=255; }
                     else { p[i]=0; p[i+1]=0; p[i+2]=0; p[i+3]=255; }
                 } else {
                     p[i]=130; p[i+1]=130; p[i+2]=130; p[i+3]=255;
                 }
             }
         }
         Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_chest_front(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                 let i = ((y*size+x)*4) as usize;
                 let lock = x >= 7 && x <= 8 && y >= 6 && y <= 8;
                 if lock {
                     p[i]=200; p[i+1]=200; p[i+2]=200; p[i+3]=255;
                 } else {
                     p[i]=160; p[i+1]=100; p[i+2]=50; p[i+3]=255;
                 }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_cactus(data: &mut [u8], size: u32, w: u32, idx: u32, top: bool) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let stripe = x % 4 == 0;
                let col = if top { [50, 150, 50] } else { if stripe { [30, 100, 30] } else { [50, 150, 50] } };
                p[i] = col[0]; p[i+1] = col[1]; p[i+2] = col[2]; p[i+3] = 255;
                if !top && x % 4 == 0 && y % 4 == 2 { // Spikes
                     p[i] = 200; p[i+1] = 200; p[i+2] = 200;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_flower(data: &mut [u8], size: u32, w: u32, idx: u32, col: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let c = size/2;
        for y in 0..size {
            for x in 0..size {
                let i = ((y*size+x)*4) as usize;
                let dx = x as i32 - c as i32; let dy = y as i32 - c as i32;
                if dx*dx + dy*dy < 12 { // Petals
                    p[i]=col[0]; p[i+1]=col[1]; p[i+2]=col[2]; p[i+3]=255;
                } else if x == c && y > c { // Stem
                    p[i]=0; p[i+1]=120; p[i+2]=0; p[i+3]=255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_torch(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                if x >= 7 && x <= 8 && y >= 6 {
                    p[i] = 139; p[i + 1] = 69; p[i + 2] = 19; p[i + 3] = 255; // Stick
                } else if x >= 6 && x <= 9 && y >= 3 && y <= 5 {
                     p[i] = 255; p[i + 1] = 200; p[i + 2] = 0; p[i + 3] = 255; // Flame
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_melon_side(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_stripe = x % 4 == 0;
                if is_stripe { p[i]=40; p[i+1]=150; p[i+2]=40; }
                else { p[i]=60; p[i+1]=180; p[i+2]=60; }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_generic(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = color[0]; p[(i * 4 + 1) as usize] = color[1]; p[(i * 4 + 2) as usize] = color[2]; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    // --- STUBS FOR COMPLETENESS ---
fn generate_deadbush(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                // Jagged, thin brown branches
                let is_branch = (x as i32 - y as i32).abs() < 2 || (x as i32 + y as i32 - 16).abs() < 2;
                if is_branch && y > 4 { p[i]=110; p[i+1]=80; p[i+2]=40; p[i+3]=255; }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_tallgrass(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_stalk = (x == 4 || x == 8 || x == 12) && y > 2;
                if is_stalk { p[i]=60; p[i+1]=140; p[i+2]=40; p[i+3]=255; }
                else if (x + y) % 4 == 0 && y > 6 { p[i]=80; p[i+1]=160; p[i+2]=60; p[i+3]=255; }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_sugarcane(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_ring = y % 5 == 0;
                if x > 5 && x < 10 {
                    if is_ring { p[i]=120; p[i+1]=200; p[i+2]=80; }
                    else { p[i]=100; p[i+1]=255; p[i+2]=100; }
                    p[i+3]=255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_sapling(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_stem = x == 8 && y > 10;
                let is_leaf = (x as i32 - 8).abs() < 4 && (y as i32 - 8).abs() < 4;
                if is_stem { p[i]=100; p[i+1]=60; p[i+2]=20; p[i+3]=255; }
                else if is_leaf { p[i]=40; p[i+1]=180; p[i+2]=40; p[i+3]=255; }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_bookshelf(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_shelf = y % 8 < 2;
                if is_shelf { p[i]=160; p[i+1]=100; p[i+2]=50; }
                else {
                    let book_col = match x % 5 { 0=>[200,50,50], 1=>[50,200,50], 2=>[50,50,200], 3=>[200,200,50], _=>[200,200,200] };
                    p[i]=book_col[0]; p[i+1]=book_col[1]; p[i+2]=book_col[2];
                }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_tnt_side(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_white_band = y > 6 && y < 10;
                if is_white_band { p[i]=255; p[i+1]=255; p[i+2]=255; }
                else { p[i]=200; p[i+1]=40; p[i+2]=20; }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_tnt_top(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_fuse = (x as i32 - 8).abs() < 2 && (y as i32 - 8).abs() < 2;
                if is_fuse { p[i]=50; p[i+1]=50; p[i+2]=50; }
                else { p[i]=200; p[i+1]=40; p[i+2]=20; }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_pumpkin_face(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_eye = (y == 4 || y == 5) && (x == 4 || x == 5 || x == 10 || x == 11);
                let is_mouth = y > 9 && y < 12 && x > 3 && x < 12;
                if is_eye || is_mouth { p[i]=40; p[i+1]=20; p[i+2]=0; }
                else { p[i]=230; p[i+1]=120; p[i+2]=0; }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_mossy(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let is_moss = (x * 7 + y * 3) % 11 < 4;
                if is_moss { p[i]=60; p[i+1]=100; p[i+2]=40; }
                else { p[i]=120; p[i+1]=120; p[i+2]=120; }
                p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_stick(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size {
            let idx = ((i * size + i) * 4) as usize;
            if idx + 3 < p.len() { p[idx]=100; p[idx+1]=60; p[idx+2]=20; p[idx+3]=255; }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_ingot(data: &mut [u8], size: u32, w: u32, idx: u32, c: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 6..10 {
            for x in 4..12 {
                let i = ((y * size + x) * 4) as usize;
                p[i]=c[0]; p[i+1]=c[1]; p[i+2]=c[2]; p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_gem(data: &mut [u8], size: u32, w: u32, idx: u32, c: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 4..12 {
            for x in 4..12 {
                let i = ((y * size + x) * 4) as usize;
                let dist = (x as i32 - 8).abs() + (y as i32 - 8).abs();
                if dist < 6 { p[i]=c[0]; p[i+1]=c[1]; p[i+2]=c[2]; p[i+3]=255; }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_meat(data: &mut [u8], size: u32, w: u32, idx: u32, c: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 5..11 {
            for x in 4..12 {
                let i = ((y * size + x) * 4) as usize;
                p[i]=c[0]; p[i+1]=c[1]; p[i+2]=c[2]; p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_tool(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let handle_col = [101, 67, 33]; // Dark Oak handle
        
        // Determine tool type based on index ranges defined in world.rs
        // 21-25: Pickaxe, 26-30: Axe, 31-35: Shovel, 36-40: Sword, 41-45: Hoe
        let tool_type = if idx >= 21 && idx <= 25 { "pick" }
                   else if idx >= 26 && idx <= 30 { "axe" }
                   else if idx >= 31 && idx <= 35 { "shovel" }
                   else if idx >= 36 && idx <= 40 { "sword" }
                   else { "hoe" };

        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                
                // 1. Handle (Diagonal with structural shading)
                if x == y && x > 2 && x < size - 2 {
                    let shade = if x % 2 == 0 { 0 } else { -15 };
                    p[i] = (handle_col[0] as i32 + shade).clamp(0, 255) as u8;
                    p[i+1] = (handle_col[1] as i32 + shade).clamp(0, 255) as u8;
                    p[i+2] = (handle_col[2] as i32 + shade).clamp(0, 255) as u8;
                    p[i+3] = 255;
                }

                // 2. DIABOLICAL SHAPE DEFINITIONS
                let is_head = match tool_type {
                    "pick" => {
                        let is_arc = y > size - 5 && (x as i32 - (size as i32 - 1 - y as i32)).abs() < 2;
                        let is_tip = (y == size - 2 && x < 6) || (x == size - 2 && y < 6);
                        is_arc || is_tip
                    },
                    "axe" => x < 7 && y > size - 8 && (x as i32 + (size as i32 - y as i32)) < 12,
                    "shovel" => x < 6 && y > size - 6 && (x as i32 - (size as i32 - y as i32)).abs() < 3,
                    "sword" => {
                        let dist_from_diag = (x as i32 + y as i32 - (size as i32 + 2)).abs();
                        dist_from_diag < 3 && x + y > size + 2
                    },
                    "hoe" => (y > size - 4 && x < 8) || (y == size - 2 && x < 10),
                    _ => false,
                };

                if is_head {
                    // Add a directional "Glint" effect for sexiness
                    let glint = if x < 4 || y > size - 4 { 45 } else if x > 8 || y < 8 { -30 } else { 0 };
                    p[i] = (color[0] as i32 + glint).clamp(0, 255) as u8;
                    p[i+1] = (color[1] as i32 + glint).clamp(0, 255) as u8;
                    p[i+2] = (color[2] as i32 + glint).clamp(0, 255) as u8;
                    p[i+3] = 255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    // --- UI HELPERS ---

    fn generate_hotbar_slot(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 50; p[(i * 4 + 1) as usize] = 50; p[(i * 4 + 2) as usize] = 50; p[(i * 4 + 3) as usize] = 150;
        }
        for i in 0..size {
            let top = (i * 4) as usize;
            let btm = (((size - 1) * size + i) * 4) as usize;
            let l = ((i * size) * 4) as usize;
            let r = ((i * size + size - 1) * 4) as usize;
            for j in 0..3 { p[top + j] = 200; p[btm + j] = 200; p[l + j] = 200; p[r + j] = 200; }
            p[top + 3] = 255; p[btm + 3] = 255; p[l + 3] = 255; p[r + 3] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_selection(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size {
            let top = (i * 4) as usize; let btm = (((size - 1) * size + i) * 4) as usize;
            let l = ((i * size) * 4) as usize; let r = ((i * size + size - 1) * 4) as usize;
            for x in [top, btm, l, r] { p[x] = 255; p[x+1] = 255; p[x+2] = 255; p[x+3] = 255; }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_heart(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let pattern = [
            0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        for y in 0..8 {
            for x in 0..8 {
                let hit = pattern[(y * 8 + x) as usize] == 1;
                if hit {
                    // Scale 8x8 to 16x16
                    for dy in 0..2 { for dx in 0..2 {
                        let i = (((y*2+dy) * size + (x*2+dx)) * 4) as usize;
                        p[i] = 220; p[i + 1] = 20; p[i + 2] = 60; p[i + 3] = 255;
                    }}
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_bubble_data(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let c = size as f32 / 2.0;
        for y in 0..size {
            for x in 0..size {
                let dx = x as f32 - c; let dy = y as f32 - c;
                let dist = (dx*dx + dy*dy).sqrt();
                let i = ((y * size + x) * 4) as usize;
                if dist > 3.5 && dist < 6.5 { // Defined Outer Ring
                    p[i] = 40; p[i+1] = 120; p[i+2] = 255; p[i+3] = 255;
                } else if dist <= 3.5 { // Refractive Highlight
                    let shine = if dx < 0.0 && dy < 0.0 { 50 } else { 0 };
                    p[i] = (150 + shine) as u8; p[i+1] = (200 + shine) as u8; p[i+2] = 255; p[i+3] = 180;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_ui_bar_data(data: &mut [u8], size: u32, w: u32, idx: u32) {
        Self::generate_generic(data, size, w, idx, [200, 200, 200]);
    }

    fn generate_cracks(data: &mut [u8], size: u32, w: u32, idx: u32, intensity: f32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                // Procedural jagged cracks based on intensity
                let seed = (x * 713 + y * 911) % 100;
                if seed < (intensity * 40.0) as u32 {
                    p[i]=0; p[i+1]=0; p[i+2]=0; p[i+3]=200; // Black jagged lines
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_wheat_stage(data: &mut [u8], size: u32, w: u32, idx: u32, stage: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let height = (stage + 1) * 2;
        let col = if stage < 7 { [100, 180, 50] } else { [200, 180, 50] }; // Green -> Yellow
        for y in (size - height)..size {
            for x in [4, 8, 12] {
                let i = ((y * size + x) * 4) as usize;
                p[i]=col[0]; p[i+1]=col[1]; p[i+2]=col[2]; p[i+3]=255;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_font(data: &mut [u8], size: u32, w: u32, start_idx: u32) {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789->/";
        let patterns: [[u8; 5]; 39] = [
            [0xE, 0x11, 0x1F, 0x11, 0x11], [0x1E, 0x11, 0x1E, 0x11, 0x1E], [0xE, 0x11, 0x10, 0x11, 0xE], [0x1E, 0x11, 0x11, 0x11, 0x1E],
            [0x1F, 0x10, 0x1E, 0x10, 0x1F], [0x1F, 0x10, 0x1E, 0x10, 0x10], [0xE, 0x11, 0x17, 0x11, 0xE], [0x11, 0x11, 0x1F, 0x11, 0x11],
            [0xE, 0x4, 0x4, 0x4, 0xE], [0x7, 0x2, 0x2, 0x12, 0xC], [0x11, 0x12, 0x1C, 0x12, 0x11], [0x10, 0x10, 0x10, 0x10, 0x1F],
            [0x11, 0x1B, 0x15, 0x11, 0x11], [0x11, 0x19, 0x15, 0x13, 0x11], [0xE, 0x11, 0x11, 0x11, 0xE], [0x1E, 0x11, 0x1E, 0x10, 0x10],
            [0xE, 0x11, 0x11, 0x13, 0xD], [0x1E, 0x11, 0x1E, 0x12, 0x11], [0xF, 0x10, 0xE, 0x1, 0x1E], [0x1F, 0x4, 0x4, 0x4, 0x4],
            [0x11, 0x11, 0x11, 0x11, 0xE], [0x11, 0x11, 0x11, 0xA, 0x4], [0x11, 0x11, 0x15, 0x15, 0xA], [0x11, 0xA, 0x4, 0xA, 0x11],
            [0x11, 0x11, 0xA, 0x4, 0x4], [0x1F, 0x2, 0x4, 0x8, 0x1F], 
            [0xE, 0x11, 0x13, 0x15, 0xE], [0x4, 0xC, 0x4, 0x4, 0xE], [0xE, 0x11, 0x2, 0x4, 0x1F], [0xE, 0x11, 0x6, 0x11, 0xE], 
            [0x9, 0x9, 0xF, 0x1, 0x1], [0x1F, 0x10, 0x1E, 0x1, 0x1E], [0xE, 0x10, 0x1E, 0x11, 0xE], [0x1F, 0x2, 0x4, 0x8, 0x8], [0xE, 0x11, 0xE, 0x11, 0xE], [0xE, 0x11, 0x1E, 0x1, 0xE],
            [0x0, 0x0, 0xF, 0x0, 0x0], [0x0, 0x2, 0x4, 0x8, 0x0],
            [0x1, 0x2, 0x4, 0x8, 0x10], // '/' SLASH PATTERN
        ];
        for (i, _) in chars.iter().enumerate() {
            let idx = start_idx + i as u32;
            Self::clear_tile(data, size, w, idx); // DIABOLICAL FIX: Wipe the tile first so no blocks show through!
            let mut p = vec![0u8; (size * size * 4) as usize];
            let pattern = patterns[i];
            for y in 0..5 {
                let row = pattern[y];
                for x in 0..5 {
                    if (row >> (4 - x)) & 1 == 1 {
                        let sx = x * 3; let sy = y * 3;
                        for dy in 0..3 { for dx in 0..3 {
                            let px_idx = (((sy + dy) * size as usize + (sx + dx)) * 4) as usize;
                            if px_idx + 3 < p.len() { p[px_idx]=255; p[px_idx+1]=255; p[px_idx+2]=255; p[px_idx+3]=255; }
                        }}
                    }
                }
            }
            Self::place_texture(data, size, w, idx, &p);
        }
    }
fn generate_bucket(data: &mut [u8], size: u32, w: u32, idx: u32, water: bool) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 4..12 {
            for x in 4..12 {
                let i = ((y * size + x) * 4) as usize;
                let is_rim = y == 4 || x == 4 || x == 11 || y == 11;
                if is_rim { p[i]=180; p[i+1]=180; p[i+2]=180; p[i+3]=255; }
                else if water { p[i]=40; p[i+1]=60; p[i+2]=220; p[i+3]=255; }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
}
// DIABOLICAL TRADITIONAL TEXTURE SYSTEM - Hand-Crafted Visual Enhancement
// 
// This module provides comprehensive traditional texture generation with:
// - Artist-designed templates for authentic Minecraft aesthetics
// - Layered texture composition for depth and detail
// - Traditional color palettes inspired by classic Minecraft
// - Material-specific rendering properties
// - Biome and time-based texture variations

// Removed unused Vec3 import
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Stone,
    Wood,
    Dirt,
    Grass,
    Sand,
    Water,
    Leaves,
    Metal,
    Glass,
    Fabric,
    Crystal,
}

#[derive(Debug, Clone)]
pub struct TextureTemplate {
    pub base_color: [u8; 3],
    pub detail_colors: Vec<[u8; 3]>,
    pub material_type: MaterialType,
    pub pattern_type: PatternType,
    pub roughness: f32,
    pub metallic: f32,
    pub transparency: f32,
    pub emission: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PatternType {
    Solid,
    Grain,
    Veined,
    Crystalline,
    Fabric,
    Metallic,
    Organic,
    Geometric,
}

#[derive(Debug, Clone)]
pub struct TextureLayer {
    pub pattern: PatternType,
    pub color: [u8; 3],
    pub intensity: f32,
    pub scale: f32,
    pub offset: [f32; 2],
}

#[derive(Debug, Clone)]
pub struct TraditionalPalette {
    pub stone_colors: [[u8; 3]; 8],
    pub wood_colors: [[u8; 3]; 6],
    pub dirt_colors: [[u8; 3]; 4],
    pub grass_colors: [[u8; 3]; 5],
    pub sand_colors: [[u8; 3]; 3],
    pub ore_colors: [[u8; 3]; 10],
    pub plant_colors: [[u8; 3]; 12],
    pub metal_colors: [[u8; 3]; 8],
}

pub struct TraditionalTextureGenerator {
    pub palette: TraditionalPalette,
    pub templates: HashMap<MaterialType, TextureTemplate>,
    pub noise_scale: f32,
    pub detail_level: f32,
}

impl TraditionalTextureGenerator {
    pub fn new() -> Self {
        let palette = Self::create_traditional_palette();
        let templates = Self::create_texture_templates(&palette);
        
        Self {
            palette,
            templates,
            noise_scale: 0.1,
            detail_level: 0.8,
        }
    }

    fn create_traditional_palette() -> TraditionalPalette {
        TraditionalPalette {
            // Warm, traditional stone colors
            stone_colors: [
                [136, 136, 136], // Light stone
                [119, 119, 119], // Medium stone
                [102, 102, 102], // Dark stone
                [85, 85, 85],     // Very dark stone
                [153, 153, 153], // Pale stone
                [170, 170, 170], // Bright stone
                [68, 68, 68],     // Shadow stone
                [187, 187, 187], // Weathered stone
            ],
            // Natural wood colors with grain
            wood_colors: [
                [143, 101, 69],  // Oak wood
                [92, 51, 23],    // Dark oak
                [194, 178, 128], // Birch wood
                [113, 67, 25],   // Spruce wood
                [160, 106, 66],  // Jungle wood
                [247, 233, 163], // Acacia wood
            ],
            // Earth tones
            dirt_colors: [
                [139, 90, 69],   // Light dirt
                [121, 85, 61],   // Medium dirt
                [109, 77, 54],   // Dark dirt
                [155, 118, 87],  // Sandy dirt
            ],
            // Natural grass colors
            grass_colors: [
                [124, 169, 80],  // Healthy grass
                [134, 179, 90],  // Lush grass
                [114, 159, 70],  // Dry grass
                [144, 189, 100], // Tropical grass
                [104, 149, 60],  // Sparse grass
            ],
            // Sandy colors
            sand_colors: [
                [238, 220, 194], // Light sand
                [218, 200, 174], // Medium sand
                [198, 180, 154], // Dark sand
            ],
            // Rich ore colors
            ore_colors: [
                [24, 24, 24],     // Coal
                [210, 180, 140], // Iron
                [255, 215, 0],   // Gold
                [0, 255, 255],    // Diamond
                [255, 0, 0],     // Redstone
                [20, 40, 180],   // Lapis
                [160, 160, 160], // Silver
                [255, 128, 0],   // Copper
                [128, 0, 128],   // Amethyst
                [255, 255, 255], // Quartz
            ],
            // Vibrant plant colors
            plant_colors: [
                [34, 89, 34],    // Green leaves
                [124, 169, 80],  // Grass green
                [255, 255, 0],   // Yellow flowers
                [255, 0, 0],     // Red flowers
                [128, 0, 128],   // Purple flowers
                [255, 192, 203], // Pink flowers
                [255, 165, 0],   // Orange flowers
                [0, 0, 255],     // Blue flowers
                [255, 255, 255], // White flowers
                [165, 42, 42],   // Brown mushrooms
                [255, 0, 255],   // Magenta flowers
                [192, 192, 192], // Gray flowers
            ],
            // Metallic colors
            metal_colors: [
                [192, 192, 192], // Iron
                [255, 215, 0],   // Gold
                [192, 192, 192], // Steel
                [184, 115, 51],  // Copper
                [128, 128, 128], // Lead
                [255, 255, 255], // Silver
                [217, 217, 217], // Aluminum
                [255, 140, 0],   // Bronze
            ],
        }
    }

    fn create_texture_templates(palette: &TraditionalPalette) -> HashMap<MaterialType, TextureTemplate> {
        let mut templates = HashMap::new();

        // Stone template with grain pattern
        templates.insert(MaterialType::Stone, TextureTemplate {
            base_color: palette.stone_colors[1],
            detail_colors: palette.stone_colors[1..].to_vec(),
            material_type: MaterialType::Stone,
            pattern_type: PatternType::Grain,
            roughness: 0.8,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Wood template with visible grain
        templates.insert(MaterialType::Wood, TextureTemplate {
            base_color: palette.wood_colors[0],
            detail_colors: palette.wood_colors[1..].to_vec(),
            material_type: MaterialType::Wood,
            pattern_type: PatternType::Grain,
            roughness: 0.6,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Dirt template with organic pattern
        templates.insert(MaterialType::Dirt, TextureTemplate {
            base_color: palette.dirt_colors[0],
            detail_colors: palette.dirt_colors[1..].to_vec(),
            material_type: MaterialType::Dirt,
            pattern_type: PatternType::Organic,
            roughness: 0.9,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Grass template with organic pattern
        templates.insert(MaterialType::Grass, TextureTemplate {
            base_color: palette.grass_colors[0],
            detail_colors: palette.grass_colors[1..].to_vec(),
            material_type: MaterialType::Grass,
            pattern_type: PatternType::Organic,
            roughness: 0.7,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Sand template with fine grain
        templates.insert(MaterialType::Sand, TextureTemplate {
            base_color: palette.sand_colors[0],
            detail_colors: palette.sand_colors[1..].to_vec(),
            material_type: MaterialType::Sand,
            pattern_type: PatternType::Grain,
            roughness: 0.5,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Water template with transparency
        templates.insert(MaterialType::Water, TextureTemplate {
            base_color: [80, 120, 180],
            detail_colors: vec![[60, 100, 160], [100, 140, 200]],
            material_type: MaterialType::Water,
            pattern_type: PatternType::Solid,
            roughness: 0.1,
            metallic: 0.0,
            transparency: 0.7,
            emission: 0.0,
        });

        // Leaves template with veined pattern
        templates.insert(MaterialType::Leaves, TextureTemplate {
            base_color: palette.plant_colors[0],
            detail_colors: palette.plant_colors[1..4].to_vec(),
            material_type: MaterialType::Leaves,
            pattern_type: PatternType::Veined,
            roughness: 0.4,
            metallic: 0.0,
            transparency: 0.3,
            emission: 0.0,
        });

        // Metal template with metallic properties
        templates.insert(MaterialType::Metal, TextureTemplate {
            base_color: palette.metal_colors[0],
            detail_colors: palette.metal_colors[1..].to_vec(),
            material_type: MaterialType::Metal,
            pattern_type: PatternType::Metallic,
            roughness: 0.2,
            metallic: 0.8,
            transparency: 0.0,
            emission: 0.0,
        });

        // Glass template with transparency
        templates.insert(MaterialType::Glass, TextureTemplate {
            base_color: [200, 200, 200],
            detail_colors: vec![[180, 180, 180], [220, 220, 220]],
            material_type: MaterialType::Glass,
            pattern_type: PatternType::Solid,
            roughness: 0.0,
            metallic: 0.0,
            transparency: 0.8,
            emission: 0.0,
        });

        // Fabric template
        templates.insert(MaterialType::Fabric, TextureTemplate {
            base_color: [200, 150, 100],
            detail_colors: vec![[180, 130, 80], [220, 170, 120]],
            material_type: MaterialType::Fabric,
            pattern_type: PatternType::Fabric,
            roughness: 0.6,
            metallic: 0.0,
            transparency: 0.0,
            emission: 0.0,
        });

        // Crystal template with crystalline pattern
        templates.insert(MaterialType::Crystal, TextureTemplate {
            base_color: [200, 200, 255],
            detail_colors: vec![[180, 180, 235], [220, 220, 255]],
            material_type: MaterialType::Crystal,
            pattern_type: PatternType::Crystalline,
            roughness: 0.1,
            metallic: 0.2,
            transparency: 0.6,
            emission: 0.1,
        });

        templates
    }

    pub fn generate_traditional_texture(
        &self,
        material_type: MaterialType,
        size: usize,
        biome_modifier: Option<BiomeModifier>,
        time_modifier: Option<TimeModifier>,
    ) -> Vec<u8> {
        let template = &self.templates[&material_type];
        let mut texture_data = Vec::with_capacity(size * size * 4);

        for y in 0..size {
            for x in 0..size {
                let uv = [x as f32 / size as f32, y as f32 / size as f32];
                
                // Generate base color with pattern
                let base_color = self.generate_pattern_color(template, uv);
                
                // Apply biome and time modifiers
                let modified_color = self.apply_modifiers(base_color, biome_modifier, time_modifier);
                
                // Add detail layers
                let final_color = self.add_detail_layers(template, modified_color, uv);
                
                texture_data.extend([final_color[0], final_color[1], final_color[2], 255]);
            }
        }

        texture_data
    }

    fn generate_pattern_color(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        match template.pattern_type {
            PatternType::Solid => template.base_color,
            PatternType::Grain => self.generate_grain_pattern(template, uv),
            PatternType::Veined => self.generate_veined_pattern(template, uv),
            PatternType::Crystalline => self.generate_crystalline_pattern(template, uv),
            PatternType::Fabric => self.generate_fabric_pattern(template, uv),
            PatternType::Metallic => self.generate_metallic_pattern(template, uv),
            PatternType::Organic => self.generate_organic_pattern(template, uv),
            PatternType::Geometric => self.generate_geometric_pattern(template, uv),
        }
    }

    fn generate_grain_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let noise = self.perlin_noise(uv[0] * 8.0, uv[1] * 8.0);
        let color_index = (noise * template.detail_colors.len() as f32) as usize % template.detail_colors.len();
        template.detail_colors[color_index]
    }

    fn generate_veined_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let vein_noise = self.perlin_noise(uv[0] * 16.0, uv[1] * 16.0);
        let base_noise = self.perlin_noise(uv[0] * 4.0, uv[1] * 4.0);
        
        if vein_noise > 0.7 {
            template.detail_colors[0] // Vein color
        } else {
            self.blend_colors(template.base_color, template.detail_colors[1], base_noise)
        }
    }

    fn generate_crystalline_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let crystal_noise = self.perlin_noise(uv[0] * 12.0, uv[1] * 12.0);
        let facet_noise = self.perlin_noise(uv[0] * 24.0, uv[1] * 24.0);
        
        let base_color = self.blend_colors(template.base_color, template.detail_colors[0], crystal_noise);
        self.blend_colors(base_color, template.detail_colors[1], facet_noise * 0.3)
    }

    fn generate_fabric_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let weave_x = (uv[0] * 20.0).sin() * 0.5 + 0.5;
        let weave_y = (uv[1] * 20.0).cos() * 0.5 + 0.5;
        let weave_pattern = (weave_x * weave_y).powf(2.0);
        
        self.blend_colors(template.base_color, template.detail_colors[0], weave_pattern)
    }

    fn generate_metallic_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let metal_noise = self.perlin_noise(uv[0] * 32.0, uv[1] * 32.0);
        let polish_pattern = (uv[0] * 100.0).sin() * (uv[1] * 100.0).cos() * 0.1 + 0.9;
        
        let base_color = self.blend_colors(template.base_color, template.detail_colors[0], metal_noise);
        self.blend_colors(base_color, template.detail_colors[1], polish_pattern)
    }

    fn generate_organic_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let organic_noise = self.perlin_noise(uv[0] * 6.0, uv[1] * 6.0);
        let detail_noise = self.perlin_noise(uv[0] * 12.0, uv[1] * 12.0);
        
        self.blend_colors(template.base_color, template.detail_colors[0], organic_noise * 0.7 + detail_noise * 0.3)
    }

    fn generate_geometric_pattern(&self, template: &TextureTemplate, uv: [f32; 2]) -> [u8; 3] {
        let grid_x = (uv[0] * 8.0) as usize % 2;
        let grid_y = (uv[1] * 8.0) as usize % 2;
        let checker_pattern = if (grid_x + grid_y) % 2 == 0 { 1.0 } else { 0.0 };
        
        self.blend_colors(template.base_color, template.detail_colors[0], checker_pattern)
    }

    fn apply_modifiers(
        &self,
        color: [u8; 3],
        biome_modifier: Option<BiomeModifier>,
        time_modifier: Option<TimeModifier>,
    ) -> [u8; 3] {
        let mut modified_color = color;
        
        if let Some(biome) = biome_modifier {
            modified_color = self.apply_biome_modifier(modified_color, biome);
        }
        
        if let Some(time) = time_modifier {
            modified_color = self.apply_time_modifier(modified_color, time);
        }
        
        modified_color
    }

    fn apply_biome_modifier(&self, color: [u8; 3], biome: BiomeModifier) -> [u8; 3] {
        match biome {
            BiomeModifier::Forest => {
                // Greener, more vibrant colors
                [
                    (color[0] as f32 * 0.9).max(0.0) as u8,
                    (color[1] as f32 * 1.2).min(255.0) as u8,
                    (color[2] as f32 * 0.8).max(0.0) as u8,
                ]
            }
            BiomeModifier::Desert => {
                // Warmer, more orange tones
                [
                    (color[0] as f32 * 1.3).min(255.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 0.7).max(0.0) as u8,
                ]
            }
            BiomeModifier::Tundra => {
                // Cooler, more blue tones
                [
                    (color[0] as f32 * 0.8).max(0.0) as u8,
                    (color[1] as f32 * 0.9).max(0.0) as u8,
                    (color[2] as f32 * 1.2).min(255.0) as u8,
                ]
            }
            BiomeModifier::Swamp => {
                // Murkier, more green-brown tones
                [
                    (color[0] as f32 * 0.9).max(0.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 0.8).max(0.0) as u8,
                ]
            }
        }
    }

    fn apply_time_modifier(&self, color: [u8; 3], time: TimeModifier) -> [u8; 3] {
        match time {
            TimeModifier::Dawn => {
                // Warmer, more orange-pink tones
                [
                    (color[0] as f32 * 1.2).min(255.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 0.9).max(0.0) as u8,
                ]
            }
            TimeModifier::Noon => {
                // Brighter, more saturated
                [
                    (color[0] as f32 * 1.1).min(255.0) as u8,
                    (color[1] as f32 * 1.1).min(255.0) as u8,
                    (color[2] as f32 * 1.1).min(255.0) as u8,
                ]
            }
            TimeModifier::Dusk => {
                // Warmer, more orange-red tones
                [
                    (color[0] as f32 * 1.3).min(255.0) as u8,
                    (color[1] as f32 * 0.9).max(0.0) as u8,
                    (color[2] as f32 * 0.7).max(0.0) as u8,
                ]
            }
            TimeModifier::Night => {
                // Cooler, more blue tones
                [
                    (color[0] as f32 * 0.7).max(0.0) as u8,
                    (color[1] as f32 * 0.8).max(0.0) as u8,
                    (color[2] as f32 * 1.2).min(255.0) as u8,
                ]
            }
        }
    }

    fn add_detail_layers(&self, template: &TextureTemplate, base_color: [u8; 3], uv: [f32; 2]) -> [u8; 3] {
        let mut final_color = base_color;
        
        // Add noise-based detail
        let detail_noise = self.perlin_noise(uv[0] * 16.0, uv[1] * 16.0);
        let detail_intensity = detail_noise * self.detail_level;
        
        // Blend with detail color
        if detail_intensity > 0.5 {
            let detail_color = template.detail_colors[1];
            final_color = self.blend_colors(final_color, detail_color, (detail_intensity - 0.5) * 2.0);
        }
        
        // Add edge highlighting
        let edge_noise = self.perlin_noise(uv[0] * 32.0, uv[1] * 32.0);
        if edge_noise > 0.8 {
            let highlight_color = [
                (final_color[0] as f32 * 1.2).min(255.0) as u8,
                (final_color[1] as f32 * 1.2).min(255.0) as u8,
                (final_color[2] as f32 * 1.2).min(255.0) as u8,
            ];
            final_color = self.blend_colors(final_color, highlight_color, (edge_noise - 0.8) * 5.0);
        }
        
        final_color
    }

    fn blend_colors(&self, color1: [u8; 3], color2: [u8; 3], factor: f32) -> [u8; 3] {
        let factor = factor.clamp(0.0, 1.0);
        [
            (color1[0] as f32 * (1.0 - factor) + color2[0] as f32 * factor) as u8,
            (color1[1] as f32 * (1.0 - factor) + color2[1] as f32 * factor) as u8,
            (color1[2] as f32 * (1.0 - factor) + color2[2] as f32 * factor) as u8,
        ]
    }

    fn perlin_noise(&self, x: f32, y: f32) -> f32 {
        // Simple Perlin noise implementation
        let x = x * self.noise_scale;
        let y = y * self.noise_scale;
        
        let xi = x.floor() as i32;
        let yi = y.floor() as i32;
        let xf = x - xi as f32;
        let yf = y - yi as f32;
        
        let u = self.fade(xf);
        let v = self.fade(yf);
        
        let a = self.hash(xi, yi);
        let b = self.hash(xi + 1, yi);
        let c = self.hash(xi, yi + 1);
        let d = self.hash(xi + 1, yi + 1);
        
        let x1 = self.lerp(self.grad(a, xf, yf), self.grad(b, xf - 1.0, yf), u);
        let x2 = self.lerp(self.grad(c, xf, yf - 1.0), self.grad(d, xf - 1.0, yf - 1.0), u);
        
        self.lerp(x1, x2, v)
    }

    fn fade(&self, t: f32) -> f32 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    fn lerp(&self, a: f32, b: f32, t: f32) -> f32 {
        a + t * (b - a)
    }

    fn grad(&self, hash: i32, x: f32, y: f32) -> f32 {
        let h = hash & 3;
        let u = if h < 2 { x } else { y };
        let v = if h < 2 { y } else { x };
        (if h & 1 == 0 { u } else { -u }) + (if h & 2 == 0 { v } else { -v })
    }

    fn hash(&self, x: i32, y: i32) -> i32 {
        let mut h = x;
        h ^= y << 13;
        h ^= h >> 17;
        h.wrapping_mul(0x85ebca6b).wrapping_add(0xc2b2ae35)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BiomeModifier {
    Forest,
    Desert,
    Tundra,
    Swamp,
}

#[derive(Debug, Clone, Copy)]
pub enum TimeModifier {
    Dawn,
    Noon,
    Dusk,
    Night,
}

pub struct TraditionalTextureAtlas {
    pub generator: TraditionalTextureGenerator,
    pub textures: HashMap<String, Vec<u8>>,
    pub size: u32,
    pub grid_size: u32,
}

impl TraditionalTextureAtlas {
    pub fn new() -> Self {
        let generator = TraditionalTextureGenerator::new();
        let textures = HashMap::new();
        
        Self {
            generator,
            textures,
            size: 512,
            grid_size: 32,
        }
    }

    pub fn generate_all_traditional_textures(&mut self) {
        let block_size = 16;
        
        // Natural materials
        self.generate_texture("grass", MaterialType::Grass, block_size);
        self.generate_texture("dirt", MaterialType::Dirt, block_size);
        self.generate_texture("stone", MaterialType::Stone, block_size);
        self.generate_texture("sand", MaterialType::Sand, block_size);
        self.generate_texture("water", MaterialType::Water, block_size);
        self.generate_texture("leaves", MaterialType::Leaves, block_size);
        
        // Building materials
        self.generate_texture("wood", MaterialType::Wood, block_size);
        self.generate_texture("glass", MaterialType::Glass, block_size);
        self.generate_texture("fabric", MaterialType::Fabric, block_size);
        
        // Special materials
        self.generate_texture("metal", MaterialType::Metal, block_size);
        self.generate_texture("crystal", MaterialType::Crystal, block_size);
        
        // Generate biome variants
        self.generate_biome_variants();
        
        // Generate time variants
        self.generate_time_variants();
    }

    fn generate_texture(&mut self, name: &str, material_type: MaterialType, block_size: usize) {
        let texture_data = self.generator.generate_traditional_texture(
            material_type,
            block_size,
            None,
            None,
        );
        self.textures.insert(name.to_string(), texture_data);
    }

    fn generate_biome_variants(&mut self) {
        let block_size = 16;
        let biomes = [BiomeModifier::Forest, BiomeModifier::Desert, BiomeModifier::Tundra, BiomeModifier::Swamp];
        
        for biome in &biomes {
            let suffix = match biome {
                BiomeModifier::Forest => "_forest",
                BiomeModifier::Desert => "_desert",
                BiomeModifier::Tundra => "_tundra",
                BiomeModifier::Swamp => "_swamp",
            };
            
            // Generate grass variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Grass,
                block_size,
                Some(*biome),
                None,
            );
            self.textures.insert(format!("grass{}", suffix), texture_data);
            
            // Generate leaves variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Leaves,
                block_size,
                Some(*biome),
                None,
            );
            self.textures.insert(format!("leaves{}", suffix), texture_data);
        }
    }

    fn generate_time_variants(&mut self) {
        let block_size = 16;
        let times = [TimeModifier::Dawn, TimeModifier::Noon, TimeModifier::Dusk, TimeModifier::Night];
        
        for time in &times {
            let suffix = match time {
                TimeModifier::Dawn => "_dawn",
                TimeModifier::Noon => "_noon",
                TimeModifier::Dusk => "_dusk",
                TimeModifier::Night => "_night",
            };
            
            // Generate grass time variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Grass,
                block_size,
                None,
                Some(*time),
            );
            self.textures.insert(format!("grass{}", suffix), texture_data);
            
            // Generate leaves time variants
            let texture_data = self.generator.generate_traditional_texture(
                MaterialType::Leaves,
                block_size,
                None,
                Some(*time),
            );
            self.textures.insert(format!("leaves{}", suffix), texture_data);
        }
    }

    pub fn get_texture(&self, name: &str) -> Option<&Vec<u8>> {
        self.textures.get(name)
    }

    pub fn create_atlas_data(&self) -> Vec<u8> {
        let mut atlas_data = vec![0u8; (self.size * self.size * 4) as usize];
        
        for (name, texture) in &self.textures {
            // Place texture in atlas
            let index = self.get_texture_index(name);
            if let Some((x, y)) = index {
                self.copy_texture_to_atlas(&mut atlas_data, texture, x, y);
            }
        }
        
        atlas_data
    }

    fn get_texture_index(&self, name: &str) -> Option<(u32, u32)> {
        // Simple grid placement - in a real implementation, this would be more sophisticated
        let texture_names: Vec<&String> = self.textures.keys().collect();
        if let Some(pos) = texture_names.iter().position(|&n| n == name) {
            let x = (pos as u32 % self.grid_size) * 16;
            let y = (pos as u32 / self.grid_size) * 16;
            Some((x, y))
        } else {
            None
        }
    }

    fn copy_texture_to_atlas(&self, atlas_data: &mut [u8], texture: &[u8], x: u32, y: u32) {
        let texture_size = 16;
        let atlas_width = self.size;
        
        for ty in 0..texture_size {
            for tx in 0..texture_size {
                let atlas_x = x + tx;
                let atlas_y = y + ty;
                
                if atlas_x < self.size && atlas_y < self.size {
                    let texture_index = ((ty * texture_size + tx) * 4) as usize;
                    let atlas_index = ((atlas_y * atlas_width + atlas_x) * 4) as usize;
                    
                    if atlas_index + 3 < atlas_data.len() && texture_index + 3 < texture.len() {
                        atlas_data[atlas_index..atlas_index + 4].copy_from_slice(&texture[texture_index..texture_index + 4]);
                    }
                }
            }
        }
    }
}
pub struct NoiseGenerator {
    permutation: [u8; 512],
}

impl NoiseGenerator {
    pub fn new(seed: u32) -> Self {
        let mut p = [0u8; 512];
        let mut permutation = [0u8; 256];
        
        let mut state = seed as u64;
        for i in 0..256 { permutation[i] = i as u8; }
        
        for i in 0..256 {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = ((state >> 33) % 256) as usize;
            permutation.swap(i, j);
        }
        
        for i in 0..512 { p[i] = permutation[i % 256]; }
        NoiseGenerator { permutation: p }
    }

    fn fade(t: f64) -> f64 { t * t * t * (t * (t * 6.0 - 15.0) + 10.0) }
    fn lerp(t: f64, a: f64, b: f64) -> f64 { a + t * (b - a) }
    fn grad(hash: u8, x: f64, y: f64, z: f64) -> f64 {
        let h = hash & 15;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 { y } else { if h == 12 || h == 14 { x } else { z } };
        (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
    }

    pub fn get_noise3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let x_idx = (x.floor() as i32 & 255) as usize;
        let y_idx = (y.floor() as i32 & 255) as usize;
        let z_idx = (z.floor() as i32 & 255) as usize;
        let x = x - x.floor(); let y = y - y.floor(); let z = z - z.floor();
        let u = Self::fade(x); let v = Self::fade(y); let w = Self::fade(z);
        
        let a = self.permutation[x_idx] as usize + y_idx;
        let aa = self.permutation[a] as usize + z_idx;
        let ab = self.permutation[a + 1] as usize + z_idx;
        let b = self.permutation[x_idx + 1] as usize + y_idx;
        let ba = self.permutation[b] as usize + z_idx;
        let bb = self.permutation[b + 1] as usize + z_idx;

        Self::lerp(w, Self::lerp(v, Self::lerp(u, Self::grad(self.permutation[aa], x, y, z),
                                     Self::grad(self.permutation[ba], x - 1.0, y, z)),
                             Self::lerp(u, Self::grad(self.permutation[ab], x, y - 1.0, z),
                                     Self::grad(self.permutation[bb], x - 1.0, y - 1.0, z))),
                     Self::lerp(v, Self::lerp(u, Self::grad(self.permutation[aa + 1], x, y, z - 1.0),
                                     Self::grad(self.permutation[ba + 1], x - 1.0, y, z - 1.0)),
                             Self::lerp(u, Self::grad(self.permutation[ab + 1], x, y - 1.0, z - 1.0),
                                     Self::grad(self.permutation[bb + 1], x - 1.0, y - 1.0, z - 1.0))))
    }

pub fn get_noise_octaves(&self, x: f64, y: f64, z: f64, octaves: u32) -> f64 {
        match octaves {
            1 => self.get_noise3d(x, y, z),
            2 => (self.get_noise3d(x, y, z) + self.get_noise3d(x * 2.0, y * 2.0, z * 2.0) * 0.5) / 1.5,
            3 => (self.get_noise3d(x, y, z) + self.get_noise3d(x * 2.0, y * 2.0, z * 2.0) * 0.5 + self.get_noise3d(x * 4.0, y * 4.0, z * 4.0) * 0.25) / 1.75,
            4 => (self.get_noise3d(x, y, z) + self.get_noise3d(x * 2.0, y * 2.0, z * 2.0) * 0.5 + self.get_noise3d(x * 4.0, y * 4.0, z * 4.0) * 0.25 + self.get_noise3d(x * 8.0, y * 8.0, z * 8.0) * 0.125) / 1.875,
            _ => {
                let mut total = 0.0; let mut frequency = 1.0; let mut amplitude = 1.0; let mut max_val = 0.0;
                for _ in 0..octaves { total += self.get_noise3d(x * frequency, y * frequency, z * frequency) * amplitude; max_val += amplitude; amplitude *= 0.5; frequency *= 2.0; }
                total / max_val
            }
        }
    }

pub fn get_height_params(&self, x: i32, z: i32) -> (f32, f32, f32, f32) {
        // DIABOLICAL FIX: 0.015 frequency makes biomes tight and varied
        let xf = x as f64 * 0.015;
        let zf = z as f64 * 0.015;
        let continentalness = self.get_noise_octaves(xf, 0.0, zf, 4) as f32;
        let erosion = self.get_noise_octaves(xf + 500.0, 11.0, zf + 500.0, 4) as f32;
        let weirdness = self.get_noise_octaves(xf + 1000.0, 22.0, zf + 1000.0, 4) as f32;
        let temperature = self.get_noise_octaves(xf - 500.0, 33.0, zf - 500.0, 3) as f32;
        (continentalness, erosion, weirdness, temperature)
    }
pub fn get_density(&self, x: i32, y: i32, z: i32, cont: f32, eros: f32, weird: f32) -> f32 {
        let xf = x as f64;
        let yf = y as f64;
        let zf = z as f64;

        // 1. Continental Foundation: The "Macro-Shape"
        // Continentalness determines the baseline ground level.
        let ground_bias = 64.0 + (cont * 32.0);

        // 2. The Monolith Warp Field: Low-frequency "hotspots"
        // This determines WHERE the laws of gravity/falloff are suspended.
        let warp_weight = self.get_noise_octaves(xf * 0.012, 13.37, zf * 0.012, 2) as f32;
        
        // 3. Volumetric Detail Noise: The "Micro-Shape"
        // Using multiple octaves at medium frequency to create crags and alcoves.
        let noise_3d = self.get_noise_octaves(xf * 0.035, yf * 0.035, zf * 0.035, 4) as f32;

        // 4. Diabolical Falloff Spline
        // Standard falloff increases as we go up.
        let mut falloff_scale = 0.15;
        
        // The Monolith Trigger: In high-warp zones, we radically flatten the falloff.
        // This forces the density to remain high even at Y=120.
        if warp_weight > 0.35 {
            let intensity = (warp_weight - 0.35) * 2.0;
            falloff_scale *= (1.0 - intensity).max(0.02);
        }

        // 5. Erosion/Weirdness Modification
        // We inject weirdness into the noise_3d directly to create jagged "spikes" in peak biomes.
        let spiked_noise = if eros < -0.4 { noise_3d + weird.abs() * 0.5 } else { noise_3d };

        // 6. River/Fjord Carving: Subtract density in river channels
        let river = self.get_river_noise(x, z) as f32;
        let river_cut = if river < 0.08 && cont > 0.0 { (0.08 - river) * 2.5 } else { 0.0 };

        (spiked_noise - ((yf as f32 - ground_bias) * falloff_scale)) - river_cut
    }

pub fn get_river_noise(&self, x: i32, z: i32) -> f64 {
        let xf = x as f64 * 0.005;
        let zf = z as f64 * 0.005;
        // Ridged noise spline for Fjords and Rivers
        let n = self.get_noise_octaves(xf, 0.5, zf, 3);
        (1.0 - (n.abs() * 2.0 - 1.0).abs()) * 0.5
    }

    #[allow(dead_code)]
    pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let (cont, eros, weird, _) = self.get_height_params(x, z);
        let mut h = 64.0 + (cont * 40.0);
        if eros < -0.3 { h += weird.abs() * 50.0; }
        h as i32
    }

#[allow(dead_code)]
    pub fn get_biome_at(&self, x: i32, z: i32, y: i32) -> &'static str {
        let (cont, eros, _weird, temp) = self.get_height_params(x, z);
        let humid = self.get_noise_octaves(x as f64 * 0.01, 44.0, z as f64 * 0.01, 3) as f32;
        self.get_biome(cont, eros, temp, humid, y)
    }

pub fn get_biome(&self, cont: f32, eros: f32, temp: f32, humid: f32, y: i32) -> &'static str {
        if y > 102 { return "peaks"; }
        if cont < -0.25 { return if temp < -0.4 { "ice_ocean" } else { "ocean" }; }
        
        // EROSION BASED BIOMES
        if eros < -0.5 { return "badlands"; }
        if eros > 0.4 { return "plains"; }
        
        // TEMPERATURE / HUMIDITY GRID
        if temp < -0.2 {
            return if humid > 0.0 { "taiga" } else { "ice_plains" };
        }
        if temp > 0.3 {
            return if humid < 0.0 { "desert" } else { "jungle" };
        }
        if humid > 0.45 { return "swamp"; }
        
        "forest"
    }
}
// Resource management and cleanup utilities
// 
// This module provides centralized resource management to prevent memory exhaustion
// and ensure proper cleanup of game resources.

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global resource limits configuration
pub struct ResourceLimits {
    pub max_chunks: usize,
    pub max_entities: usize,
    pub max_particles: usize,
    pub max_pending_tasks: usize,
    pub mesh_memory_limit_mb: usize,
    pub texture_memory_limit_mb: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_chunks: 2000,           // Maximum number of loaded chunks
            max_entities: 1000,          // Maximum number of entities
            max_particles: 5000,        // Maximum number of particles
            max_pending_tasks: 100,     // Maximum pending mesh tasks
            mesh_memory_limit_mb: 512,  // 512MB for mesh data
            texture_memory_limit_mb: 256, // 256MB for textures
        }
    }
}

/// Resource usage tracking
pub struct ResourceTracker {
    pub chunks_loaded: AtomicUsize,
    pub entities_active: AtomicUsize,
    pub particles_active: AtomicUsize,
    pub pending_tasks: AtomicUsize,
    pub mesh_memory_mb: AtomicUsize,
    pub texture_memory_mb: AtomicUsize,
    last_cleanup: Instant,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            chunks_loaded: AtomicUsize::new(0),
            entities_active: AtomicUsize::new(0),
            particles_active: AtomicUsize::new(0),
            pending_tasks: AtomicUsize::new(0),
            mesh_memory_mb: AtomicUsize::new(0),
            texture_memory_mb: AtomicUsize::new(0),
            last_cleanup: Instant::now(),
        }
    }

    pub fn check_limits(&self, limits: &ResourceLimits) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.chunks_loaded.load(Ordering::Relaxed) > limits.max_chunks {
            warnings.push(format!("Chunk limit exceeded: {}/{}", 
                self.chunks_loaded.load(Ordering::Relaxed), limits.max_chunks));
        }

        if self.entities_active.load(Ordering::Relaxed) > limits.max_entities {
            warnings.push(format!("Entity limit exceeded: {}/{}", 
                self.entities_active.load(Ordering::Relaxed), limits.max_entities));
        }

        if self.particles_active.load(Ordering::Relaxed) > limits.max_particles {
            warnings.push(format!("Particle limit exceeded: {}/{}", 
                self.particles_active.load(Ordering::Relaxed), limits.max_particles));
        }

        if self.pending_tasks.load(Ordering::Relaxed) > limits.max_pending_tasks {
            warnings.push(format!("Pending task limit exceeded: {}/{}", 
                self.pending_tasks.load(Ordering::Relaxed), limits.max_pending_tasks));
        }

        if self.mesh_memory_mb.load(Ordering::Relaxed) > limits.mesh_memory_limit_mb {
            warnings.push(format!("Mesh memory limit exceeded: {}MB/{}MB", 
                self.mesh_memory_mb.load(Ordering::Relaxed), limits.mesh_memory_limit_mb));
        }

        if self.texture_memory_mb.load(Ordering::Relaxed) > limits.texture_memory_limit_mb {
            warnings.push(format!("Texture memory limit exceeded: {}MB/{}MB", 
                self.texture_memory_mb.load(Ordering::Relaxed), limits.texture_memory_limit_mb));
        }

        warnings
    }

    pub fn should_cleanup(&self) -> bool {
        self.last_cleanup.elapsed() > Duration::from_secs(30)
    }

    pub fn mark_cleanup(&mut self) {
        self.last_cleanup = Instant::now();
    }
}

/// Cleanup strategies for different resource types
pub enum CleanupStrategy {
    /// Remove oldest resources
    OldestFirst,
    /// Remove farthest from player
    FarthestFromPlayer,
    /// Remove least recently used
    LeastRecentlyUsed,
    /// Random removal (for particles)
    Random,
}

/// Resource cleanup manager
pub struct ResourceCleanupManager {
    tracker: ResourceTracker,
    limits: ResourceLimits,
}

impl ResourceCleanupManager {
    pub fn new() -> Self {
        Self {
            tracker: ResourceTracker::new(),
            limits: ResourceLimits::default(),
        }
    }

    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            tracker: ResourceTracker::new(),
            limits,
        }
    }

    pub fn tracker(&self) -> &ResourceTracker {
        &self.tracker
    }

    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Perform cleanup if needed and return cleanup statistics
    pub fn cleanup_if_needed(&mut self) -> CleanupStats {
        if !self.tracker.should_cleanup() {
            return CleanupStats::default();
        }

        let warnings = self.tracker.check_limits(&self.limits);
        if warnings.is_empty() {
            self.tracker.mark_cleanup();
            return CleanupStats::default();
        }

        log::info!("Starting resource cleanup: {:?}", warnings);
        let stats = self.perform_cleanup();
        self.tracker.mark_cleanup();
        
        log::info!("Cleanup completed: {:?}", stats);
        stats
    }

    fn perform_cleanup(&mut self) -> CleanupStats {
        let mut stats = CleanupStats::default();

        // This would be called from the main game loop with actual resource references
        // For now, we'll just log what would be cleaned up
        
        if self.tracker.chunks_loaded.load(Ordering::Relaxed) > self.limits.max_chunks {
            let excess = self.tracker.chunks_loaded.load(Ordering::Relaxed) - self.limits.max_chunks;
            stats.chunks_cleaned = excess;
            log::debug!("Would clean up {} excess chunks", excess);
        }

        if self.tracker.entities_active.load(Ordering::Relaxed) > self.limits.max_entities {
            let excess = self.tracker.entities_active.load(Ordering::Relaxed) - self.limits.max_entities;
            stats.entities_cleaned = excess;
            log::debug!("Would clean up {} excess entities", excess);
        }

        if self.tracker.particles_active.load(Ordering::Relaxed) > self.limits.max_particles {
            let excess = self.tracker.particles_active.load(Ordering::Relaxed) - self.limits.max_particles;
            stats.particles_cleaned = excess;
            log::debug!("Would clean up {} excess particles", excess);
        }

        stats
    }
}

/// Cleanup statistics
#[derive(Debug, Default)]
pub struct CleanupStats {
    pub chunks_cleaned: usize,
    pub entities_cleaned: usize,
    pub particles_cleaned: usize,
    pub tasks_cancelled: usize,
    pub memory_freed_mb: usize,
}

impl CleanupStats {
    pub fn total_cleaned(&self) -> usize {
        self.chunks_cleaned + self.entities_cleaned + self.particles_cleaned + self.tasks_cancelled
    }

    pub fn has_cleaned_anything(&self) -> bool {
        self.total_cleaned() > 0 || self.memory_freed_mb > 0
    }
}

/// Global resource manager instance
static mut RESOURCE_MANAGER: Option<ResourceCleanupManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Get the global resource manager
#[allow(static_mut_refs)]
pub fn get_resource_manager() -> &'static mut ResourceCleanupManager {
    unsafe {
        INIT.call_once(|| {
            RESOURCE_MANAGER = Some(ResourceCleanupManager::new());
        });
        RESOURCE_MANAGER.as_mut().unwrap()
    }
}

/// Initialize the resource manager with custom limits
pub fn init_resource_manager(limits: ResourceLimits) {
    unsafe {
        INIT.call_once(|| {
            RESOURCE_MANAGER = Some(ResourceCleanupManager::with_limits(limits));
        });
    }
}

/// Convenience functions for tracking resource usage
pub fn track_chunk_usage(count: usize) {
    get_resource_manager().tracker.chunks_loaded.store(count, Ordering::Relaxed);
}

pub fn track_entity_usage(count: usize) {
    get_resource_manager().tracker.entities_active.store(count, Ordering::Relaxed);
}

pub fn track_particle_usage(count: usize) {
    get_resource_manager().tracker.particles_active.store(count, Ordering::Relaxed);
}

pub fn track_pending_tasks(count: usize) {
    get_resource_manager().tracker.pending_tasks.store(count, Ordering::Relaxed);
}

pub fn track_mesh_memory_mb(mb: usize) {
    get_resource_manager().tracker.mesh_memory_mb.store(mb, Ordering::Relaxed);
}

pub fn track_texture_memory_mb(mb: usize) {
    get_resource_manager().tracker.texture_memory_mb.store(mb, Ordering::Relaxed);
}

/// Perform cleanup if needed
pub fn cleanup_if_needed() -> CleanupStats {
    get_resource_manager().cleanup_if_needed()
}

/// Check if any resource limits are exceeded
pub fn check_resource_limits() -> Vec<String> {
    let manager = get_resource_manager();
    manager.tracker.check_limits(&manager.limits)
}
