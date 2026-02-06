#[allow(dead_code)]
pub struct TextureAtlas {
    pub data: Vec<u8>,
    pub size: u32,
    pub grid_size: u32,
}

impl TextureAtlas {
    pub fn new() -> Self {
        let atlas_width = 256;
        let atlas_height = 256;
        let total_pixels = atlas_width * atlas_height;
        let rgba_bytes = (total_pixels * 4) as usize;
        let mut data = vec![0u8; rgba_bytes];

        let block_size = 16;
        let grid_width_in_blocks = atlas_width / block_size;

        // --- 1. BASIC TERRAIN ---
        // Grass Top (Green Noise)
        Self::generate_noise(&mut data, block_size, atlas_width, 0, [100, 170, 80], 20); 
        // Grass Side (Dirt + Green Overlay)
        Self::generate_grass_side(&mut data, block_size, atlas_width, 1);
        // Dirt (Brown Noise)
        Self::generate_noise(&mut data, block_size, atlas_width, 2, [139, 69, 19], 30);
        // Stone (Grey Noise)
        Self::generate_noise(&mut data, block_size, atlas_width, 3, [125, 125, 125], 20);
        // Wood Side (Bark - Vertical Streaks)
        Self::generate_wood_side(&mut data, block_size, atlas_width, 4, [110, 80, 50]);
        // Leaves (Green pattern)
        Self::generate_leaves(&mut data, block_size, atlas_width, 5, [50, 120, 50]);
        // Snow
        Self::generate_noise(&mut data, block_size, atlas_width, 6, [245, 250, 255], 10);
        // Sand
        Self::generate_noise(&mut data, block_size, atlas_width, 7, [238, 228, 170], 15);
        // Bedrock (High Contrast)
        Self::generate_bedrock(&mut data, block_size, atlas_width, 8);
        // Water (Animated Blue)
        Self::generate_liquid(&mut data, block_size, atlas_width, 9, [40, 60, 220]);

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
        for i in 21..=25 { Self::generate_tool(&mut data, block_size, atlas_width, i, [150, 100, 50]); } // Wood
        for i in 26..=30 { Self::generate_tool(&mut data, block_size, atlas_width, i, [120, 120, 120]); } // Stone
        for i in 31..=35 { Self::generate_tool(&mut data, block_size, atlas_width, i, [200, 200, 200]); } // Iron
        for i in 36..=40 { Self::generate_tool(&mut data, block_size, atlas_width, i, [255, 215, 0]); }   // Gold (Indices reused for example)

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

        Self::generate_font(&mut data, block_size, atlas_width, 200);

        TextureAtlas { data, size: block_size, grid_size: grid_width_in_blocks }
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
                // White base
                p[i] = 220; p[i+1] = 220; p[i+2] = 220; p[i+3] = 255;
                // Black spots
                if (x % 5 == 0 && y % 7 == 0) || (x % 8 == 0 && y % 4 == 0) {
                     p[i] = 50; p[i+1] = 50; p[i+2] = 50;
                }
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
        // Base Stone
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size*size {
            let val = (125 + (i % 5 * 10) as i32 - 20) as u8;
            let base = (i as usize) * 4;
            p[base] = val; p[base+1] = val; p[base+2] = val; p[base+3] = 255;
        }
        // Clusters
        for i in 0..size*size {
            if (i % 7 == 0 && i % 3 != 0) || i % 19 == 0 {
                 let base = (i as usize) * 4;
                 p[base] = ore_col[0]; p[base+1] = ore_col[1]; p[base+2] = ore_col[2];
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
                let grid = x > 3 && x < 13 && y > 3 && y < 13;
                if grid {
                    p[i] = 100; p[i+1] = 60; p[i+2] = 20; p[i+3] = 255;
                } else {
                    p[i] = 180; p[i+1] = 130; p[i+2] = 80; p[i+3] = 255; // Wood
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_crafting_side(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size*size {
            let base = (i*4) as usize;
            p[base]=160; p[base+1]=100; p[base+2]=50; p[base+3]=255;
        }
        // Tools on side
        let mid = (size/2) * size + (size/2);
        let m = (mid*4) as usize;
        p[m]=50; p[m+1]=50; p[m+2]=50;
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

    fn generate_generic(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = color[0]; p[(i * 4 + 1) as usize] = color[1]; p[(i * 4 + 2) as usize] = color[2]; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    // --- STUBS FOR COMPLETENESS ---
    fn generate_deadbush(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 60, 20]); }
    fn generate_tallgrass(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [50, 200, 50]); }
    fn generate_sugarcane(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 255, 100]); }
    fn generate_sapling(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [34, 139, 34]); }
    fn generate_bookshelf(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 50, 20]); }
    fn generate_tnt_side(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [200, 50, 50]); }
    fn generate_tnt_top(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [200, 200, 200]); }
    fn generate_pumpkin_face(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [255, 140, 0]); }
    fn generate_melon_side(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 200, 50]); }
    fn generate_mossy(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 120, 100]); }
    fn generate_stick(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 50, 0]); }
    fn generate_ingot(data: &mut [u8], size: u32, w: u32, idx: u32, c: [u8; 3]) { Self::generate_generic(data, size, w, idx, c); }
    fn generate_gem(data: &mut [u8], size: u32, w: u32, idx: u32, c: [u8; 3]) { Self::generate_generic(data, size, w, idx, c); }
    fn generate_meat(data: &mut [u8], size: u32, w: u32, idx: u32, c: [u8; 3]) { Self::generate_generic(data, size, w, idx, c); }

    fn generate_tool(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                if x == y { // Handle
                    p[i] = 100; p[i+1] = 50; p[i+2] = 0; p[i+3] = 255;
                } else if y > size-6 && x < 6 { // Head
                    p[i] = color[0]; p[i+1] = color[1]; p[i+2] = color[2]; p[i+3] = 255;
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
        for y in 4..12 { for x in 4..12 {
            let i = ((y*size+x)*4) as usize;
            p[i]=200; p[i+1]=200; p[i+2]=255; p[i+3]=200;
        }}
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_ui_bar_data(data: &mut [u8], size: u32, w: u32, idx: u32) {
        Self::generate_generic(data, size, w, idx, [200, 200, 200]);
    }

    fn generate_font(data: &mut [u8], size: u32, w: u32, start_idx: u32) {
        let chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789->";
        let patterns: [[u8; 5]; 38] = [
            [0xE, 0x11, 0x1F, 0x11, 0x11], [0x1E, 0x11, 0x1E, 0x11, 0x1E], [0xE, 0x11, 0x10, 0x11, 0xE], [0x1E, 0x11, 0x11, 0x11, 0x1E],
            [0x1F, 0x10, 0x1E, 0x10, 0x1F], [0x1F, 0x10, 0x1E, 0x10, 0x10], [0xE, 0x11, 0x17, 0x11, 0xE], [0x11, 0x11, 0x1F, 0x11, 0x11],
            [0xE, 0x4, 0x4, 0x4, 0xE], [0x7, 0x2, 0x2, 0x12, 0xC], [0x11, 0x12, 0x1C, 0x12, 0x11], [0x10, 0x10, 0x10, 0x10, 0x1F],
            [0x11, 0x1B, 0x15, 0x11, 0x11], [0x11, 0x19, 0x15, 0x13, 0x11], [0xE, 0x11, 0x11, 0x11, 0xE], [0x1E, 0x11, 0x1E, 0x10, 0x10],
            [0xE, 0x11, 0x11, 0x13, 0xD], [0x1E, 0x11, 0x1E, 0x12, 0x11], [0xF, 0x10, 0xE, 0x1, 0x1E], [0x1F, 0x4, 0x4, 0x4, 0x4],
            [0x11, 0x11, 0x11, 0x11, 0xE], [0x11, 0x11, 0x11, 0xA, 0x4], [0x11, 0x11, 0x15, 0x15, 0xA], [0x11, 0xA, 0x4, 0xA, 0x11],
            [0x11, 0x11, 0xA, 0x4, 0x4], [0x1F, 0x2, 0x4, 0x8, 0x1F], 
            // 0-3
            [0xE, 0x11, 0x13, 0x15, 0xE], [0x4, 0xC, 0x4, 0x4, 0xE], [0xE, 0x11, 0x2, 0x4, 0x1F], [0xE, 0x11, 0x6, 0x11, 0xE], 
            // 4-9
            [0x9, 0x9, 0xF, 0x1, 0x1], [0x1F, 0x10, 0x1E, 0x1, 0x1E], [0xE, 0x10, 0x1E, 0x11, 0xE], [0x1F, 0x2, 0x4, 0x8, 0x8], [0xE, 0x11, 0xE, 0x11, 0xE], [0xE, 0x11, 0x1E, 0x1, 0xE],
            // ->
            [0x0, 0x0, 0xF, 0x0, 0x0], [0x0, 0x2, 0x4, 0x8, 0x0],
        ];
        for (i, _) in chars.iter().enumerate() {
            let mut p = vec![0u8; (size * size * 4) as usize];
            let pattern = patterns[i];
            for y in 0..5 {
                let row = pattern[y];
                for x in 0..5 {
                    if (row >> (4 - x)) & 1 == 1 {
                        let sx = x * 3; let sy = y * 3;
                        for dy in 0..3 { for dx in 0..3 {
                            let idx = (((sy + dy) * size as usize + (sx + dx)) * 4) as usize;
                            if idx + 3 < p.len() { p[idx]=255; p[idx+1]=255; p[idx+2]=255; p[idx+3]=255; }
                        }}
                    }
                }
            }
            Self::place_texture(data, size, w, start_idx + i as u32, &p);
        }
    }
}