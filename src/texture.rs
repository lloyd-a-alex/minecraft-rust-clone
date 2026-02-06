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

        // --- BLOCKS (0-9) ---
        // FIX: Generate Index 0 (Grass Top) so it isn't transparent air!
        Self::generate_grass(&mut data, block_size, atlas_width, 0); 
        Self::generate_grass(&mut data, block_size, atlas_width, 1); // Side
        Self::generate_dirt(&mut data, block_size, atlas_width, 2);
        Self::generate_stone(&mut data, block_size, atlas_width, 3);
        Self::generate_wood(&mut data, block_size, atlas_width, 4);
        Self::generate_leaves(&mut data, block_size, atlas_width, 5);
        Self::generate_snow(&mut data, block_size, atlas_width, 6);
        Self::generate_sand(&mut data, block_size, atlas_width, 7);
        Self::generate_bedrock(&mut data, block_size, atlas_width, 8);
        Self::generate_water(&mut data, block_size, atlas_width, 9);

        // --- CRAFTING TABLE (Index 21) ---
        let idx = 21;
        for y in 0..block_size {
            for x in 0..block_size {
                let i = ((idx / grid_width_in_blocks) * block_size + y) * atlas_width + ((idx % grid_width_in_blocks) * block_size + x);
                let pixel_idx = (i * 4) as usize;
                let border = x == 0 || x == block_size-1 || y == 0 || y == block_size-1 || x == block_size/2 || y == block_size/2;
                if border {
                    data[pixel_idx] = 60; data[pixel_idx+1] = 40; data[pixel_idx+2] = 10; data[pixel_idx+3] = 255;
                } else {
                    data[pixel_idx] = 160; data[pixel_idx+1] = 110; data[pixel_idx+2] = 60; data[pixel_idx+3] = 255;
                }
            }
        }

// --- NEW BLOCKS & FIXES ---
        // Lava & Fire
        Self::generate_generic(&mut data, block_size, atlas_width, 200, [255, 69, 0]); // Lava
        Self::generate_generic(&mut data, block_size, atlas_width, 201, [255, 140, 0]); // Fire
        
        // Spruce
        Self::generate_generic(&mut data, block_size, atlas_width, 202, [61, 46, 32]); // Spruce Log
        Self::generate_generic(&mut data, block_size, atlas_width, 203, [50, 80, 50]); // Spruce Leaves
        
        // Birch
        Self::generate_generic(&mut data, block_size, atlas_width, 204, [220, 220, 220]); // Birch Log
        Self::generate_generic(&mut data, block_size, atlas_width, 205, [100, 160, 100]); // Birch Leaves

        // Food Items
        Self::generate_generic(&mut data, block_size, atlas_width, 80, [150, 200, 50]); // Wheat
        Self::generate_generic(&mut data, block_size, atlas_width, 81, [180, 130, 50]); // Bread
        Self::generate_generic(&mut data, block_size, atlas_width, 82, [255, 0, 0]);    // Apple
        Self::generate_generic(&mut data, block_size, atlas_width, 83, [255, 182, 193]); // Porkchop
        Self::generate_generic(&mut data, block_size, atlas_width, 84, [200, 100, 100]); // Cooked Porkchop

        // Decorators
        Self::generate_generic(&mut data, block_size, atlas_width, 60, [100, 100, 255]); // Ice
        Self::generate_generic(&mut data, block_size, atlas_width, 61, [20, 100, 20]);   // Lilypad
        Self::generate_generic(&mut data, block_size, atlas_width, 62, [100, 80, 100]);  // Mycelium
        Self::generate_generic(&mut data, block_size, atlas_width, 63, [100, 70, 50]);   // Mycelium Side
        Self::generate_generic(&mut data, block_size, atlas_width, 64, [30, 100, 30]);   // Vine
        
        Self::generate_torch(&mut data, block_size, atlas_width, 20); 
        Self::generate_generic(&mut data, block_size, atlas_width, 14, [200, 150, 100]); // Planks (Brown)
        Self::generate_generic(&mut data, block_size, atlas_width, 15, [139, 69, 19]);   // Stick (Dark Brown)
        Self::generate_noise(&mut data, block_size, atlas_width, 16, [100, 100, 100], 30); // Cobble (Grey Noise)
        
        // Ores
        Self::generate_ore(&mut data, block_size, atlas_width, 10, [20, 20, 20]);    // Coal Ore
        Self::generate_ore(&mut data, block_size, atlas_width, 11, [200, 150, 100]); // Iron Ore
        Self::generate_ore(&mut data, block_size, atlas_width, 12, [255, 215, 0]);   // Gold Ore
        Self::generate_ore(&mut data, block_size, atlas_width, 13, [0, 255, 255]);   // Diamond Ore
        Self::generate_ore(&mut data, block_size, atlas_width, 22, [255, 0, 0]);     // Redstone Ore
        Self::generate_ore(&mut data, block_size, atlas_width, 23, [0, 0, 139]);     // Lapis Ore

        // Crafting Table
        Self::generate_crafting_side(&mut data, block_size, atlas_width, 25);

        // Furnace
        Self::generate_generic(&mut data, block_size, atlas_width, 26, [60, 60, 60]); // Top
        Self::generate_furnace_front(&mut data, block_size, atlas_width, 27, false);
        
        // Chest
        Self::generate_generic(&mut data, block_size, atlas_width, 28, [160, 82, 45]); // Top
        Self::generate_chest_front(&mut data, block_size, atlas_width, 29);

        // Environment
        Self::generate_noise(&mut data, block_size, atlas_width, 30, [128, 128, 128], 40); // Gravel
        Self::generate_generic(&mut data, block_size, atlas_width, 31, [158, 164, 176]); // Clay
        Self::generate_generic(&mut data, block_size, atlas_width, 32, [218, 204, 165]); // Sandstone Top/Bot
        Self::generate_sandstone_side(&mut data, block_size, atlas_width, 33); // Side
        Self::generate_generic(&mut data, block_size, atlas_width, 34, [20, 10, 30]); // Obsidian
        
        Self::generate_cactus(&mut data, block_size, atlas_width, 35, true); // Cactus Top
        Self::generate_cactus(&mut data, block_size, atlas_width, 36, false); // Cactus Side
        
        // Plants (Cross Models)
        Self::generate_flower(&mut data, block_size, atlas_width, 37, [255, 0, 0]); // Rose
        Self::generate_flower(&mut data, block_size, atlas_width, 38, [255, 255, 0]); // Dandelion
        Self::generate_deadbush(&mut data, block_size, atlas_width, 39);
        Self::generate_tallgrass(&mut data, block_size, atlas_width, 45);
        Self::generate_sugarcane(&mut data, block_size, atlas_width, 46);
        Self::generate_sapling(&mut data, block_size, atlas_width, 47);
        
        // Misc
        Self::generate_glass(&mut data, block_size, atlas_width, 48);
        Self::generate_bookshelf(&mut data, block_size, atlas_width, 49);
        Self::generate_tnt_side(&mut data, block_size, atlas_width, 51); Self::generate_tnt_top(&mut data, block_size, atlas_width, 50);
        Self::generate_pumpkin(&mut data, block_size, atlas_width, 53); Self::generate_generic(&mut data, block_size, atlas_width, 52, [200, 100, 0]);
        Self::generate_melon(&mut data, block_size, atlas_width, 55); Self::generate_generic(&mut data, block_size, atlas_width, 54, [100, 200, 0]);
        Self::generate_brick(&mut data, block_size, atlas_width, 56);
        Self::generate_mossy(&mut data, block_size, atlas_width, 57);

        // Items/Tools (Simplified Placeholders)
        Self::generate_generic(&mut data, block_size, atlas_width, 40, [100, 50, 0]);    // Stick Item
        Self::generate_generic(&mut data, block_size, atlas_width, 41, [20, 20, 20]);    // Coal Item
        Self::generate_generic(&mut data, block_size, atlas_width, 42, [180, 180, 180]); // Iron Ingot
        Self::generate_generic(&mut data, block_size, atlas_width, 43, [255, 215, 0]);   // Gold Ingot
        Self::generate_generic(&mut data, block_size, atlas_width, 44, [0, 255, 255]);   // Diamond Item
        
        // Tools (Wood=50s, Stone=60s, Iron=70s)
        for i in 50..55 { Self::generate_tool(&mut data, block_size, atlas_width, i, [150, 100, 50]); }
        for i in 60..65 { Self::generate_tool(&mut data, block_size, atlas_width, i, [100, 100, 100]); }
        for i in 70..75 { Self::generate_tool(&mut data, block_size, atlas_width, i, [200, 200, 200]); }

        // --- UI ---
        Self::generate_hotbar_slot(&mut data, block_size, atlas_width, 10);
        Self::generate_selection(&mut data, block_size, atlas_width, 11);
        Self::generate_heart(&mut data, block_size, atlas_width, 12);
        Self::generate_skin(&mut data, block_size, atlas_width, 13);

        // --- FONT ---
        Self::generate_font(&mut data, block_size, atlas_width, 200);

        TextureAtlas { data, size: block_size, grid_size: grid_width_in_blocks }
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
                let dst_x = base_x + x;
                let dst_y = base_y + y;
                let dst_idx = ((dst_y * atlas_width + dst_x) * 4) as usize;

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

    fn generate_grass(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let n = (i % 5) as u8;
            p[(i * 4) as usize] = 50u8.saturating_sub(n);
            p[(i * 4 + 1) as usize] = 205u8.saturating_sub(n);
            p[(i * 4 + 2) as usize] = 50u8.saturating_sub(n);
            p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_dirt(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let n = (i % 7) as u8;
            p[(i * 4) as usize] = 139u8.saturating_add(n);
            p[(i * 4 + 1) as usize] = 69;
            p[(i * 4 + 2) as usize] = 19;
            p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_stone(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 120; p[(i * 4 + 1) as usize] = 120; p[(i * 4 + 2) as usize] = 120; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_wood(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 160; p[(i * 4 + 1) as usize] = 82; p[(i * 4 + 2) as usize] = 45; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_leaves(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 34; p[(i * 4 + 1) as usize] = 139; p[(i * 4 + 2) as usize] = 34; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_snow(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 240; p[(i * 4 + 1) as usize] = 240; p[(i * 4 + 2) as usize] = 240; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_sand(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 238; p[(i * 4 + 1) as usize] = 214; p[(i * 4 + 2) as usize] = 175; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_bedrock(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 64; p[(i * 4 + 1) as usize] = 64; p[(i * 4 + 2) as usize] = 64; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_water(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 30; p[(i * 4 + 1) as usize] = 80; p[(i * 4 + 2) as usize] = 200; p[(i * 4 + 3) as usize] = 150;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_torch(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                if x >= 6 && x <= 9 && y >= 8 && y <= 11 {
                    p[i] = 255; p[i + 1] = 200; p[i + 2] = 0; p[i + 3] = 255; // Flame
                } else if x >= 6 && x <= 9 && y < 8 {
                    p[i] = 139; p[i + 1] = 69; p[i + 2] = 19; p[i + 3] = 255; // Stick
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
    
fn generate_ore(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 120; p[(i * 4 + 1) as usize] = 120; p[(i * 4 + 2) as usize] = 120; p[(i * 4 + 3) as usize] = 255;
            if i % 7 == 0 || i % 13 == 0 {
                p[(i * 4) as usize] = color[0]; p[(i * 4 + 1) as usize] = color[1]; p[(i * 4 + 2) as usize] = color[2];
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_noise(data: &mut [u8], size: u32, w: u32, idx: u32, base: [u8; 3], var: i32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let n = (i % 7) as i32 * var / 7;
            p[(i * 4) as usize] = (base[0] as i32 + n).clamp(0,255) as u8;
            p[(i * 4 + 1) as usize] = (base[1] as i32 + n).clamp(0,255) as u8;
            p[(i * 4 + 2) as usize] = (base[2] as i32 + n).clamp(0,255) as u8;
            p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_cactus(data: &mut [u8], size: u32, w: u32, idx: u32, top: bool) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let col = if top { [50, 150, 50] } else { [20, 120, 20] };
            let base = (i * 4) as usize;
            p[base] = col[0]; p[base+1] = col[1]; p[base+2] = col[2]; p[base+3] = 255;
            if !top && i % 4 == 0 { // Spikes
                p[base] = 0; p[base+1] = 0; p[base+2] = 0;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_flower(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
         let mut p = vec![0u8; (size * size * 4) as usize];
         let center = size / 2;
         for y in 0..size {
             for x in 0..size {
                 let i = ((y * size + x) * 4) as usize;
                 let dx = x as i32 - center as i32; let dy = y as i32 - center as i32;
                 if dx*dx + dy*dy < 10 {
                     p[i] = color[0]; p[i+1] = color[1]; p[i+2] = color[2]; p[i+3] = 255;
                 } else if dx == 0 && dy > 0 { // Stem
                     p[i] = 0; p[i+1] = 128; p[i+2] = 0; p[i+3] = 255;
                 }
             }
         }
         Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_glass(data: &mut [u8], size: u32, w: u32, idx: u32) {
         let mut p = vec![0u8; (size * size * 4) as usize];
         // Frame
         for i in 0..size {
             let top = (i * 4) as usize; let btm = (((size-1)*size+i)*4) as usize;
             let l = ((i*size)*4) as usize; let r = ((i*size+size-1)*4) as usize;
             for x in [top, btm, l, r] { p[x]=200; p[x+1]=255; p[x+2]=255; p[x+3]=255; }
         }
         // Streaks
         for i in 5..10 { let idx = ((i*size+i)*4) as usize; p[idx]=220; p[idx+1]=255; p[idx+2]=255; p[idx+3]=200; }
         Self::place_texture(data, size, w, idx, &p);
    }
    
    // Quick Stubs for others
    fn generate_crafting_side(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [160, 110, 60]); }
fn generate_furnace_front(data: &mut [u8], size: u32, w: u32, idx: u32, _active: bool) { 
        Self::generate_generic(data, size, w, idx, [60, 60, 60]); 
        // Add black hole + orange if active (Placeholder logic removed to stop warning)
    }
    fn generate_chest_front(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [160, 82, 45]); }
    fn generate_sandstone_side(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [218, 204, 165]); }
    fn generate_deadbush(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 60, 20]); }
    fn generate_tallgrass(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [50, 200, 50]); }
    fn generate_sugarcane(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 255, 100]); }
    fn generate_sapling(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [34, 139, 34]); }
    fn generate_bookshelf(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 50, 20]); }
    fn generate_tnt_side(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [200, 50, 50]); }
    fn generate_tnt_top(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [200, 200, 200]); }
    fn generate_pumpkin(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [255, 140, 0]); }
    fn generate_melon(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 200, 50]); }
    fn generate_brick(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [150, 50, 50]); }
    fn generate_mossy(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [100, 120, 100]); }
    
    fn generate_tool(data: &mut [u8], size: u32, w: u32, idx: u32, color: [u8; 3]) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                if x == y { // Handle
                    p[i] = 100; p[i+1] = 50; p[i+2] = 0; p[i+3] = 255;
                } else if y > 10 && x < 6 { // Head
                    p[i] = color[0]; p[i+1] = color[1]; p[i+2] = color[2]; p[i+3] = 255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_hotbar_slot(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 50; p[(i * 4 + 1) as usize] = 50; p[(i * 4 + 2) as usize] = 50; p[(i * 4 + 3) as usize] = 150;
        }
        for i in 0..size { // Border
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
            let top = (i * 4) as usize;
            let btm = (((size - 1) * size + i) * 4) as usize;
            let l = ((i * size) * 4) as usize;
            let r = ((i * size + size - 1) * 4) as usize;
            p[top] = 255; p[top + 1] = 255; p[top + 2] = 255; p[top + 3] = 255;
            p[btm] = 255; p[btm + 1] = 255; p[btm + 2] = 255; p[btm + 3] = 255;
            p[l] = 255; p[l + 1] = 255; p[l + 2] = 255; p[l + 3] = 255;
            p[r] = 255; p[r + 1] = 255; p[r + 2] = 255; p[r + 3] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_heart(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let pattern = [
            0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let py = (y * 8) / size;
                let px = (x * 8) / size;
                let hit = pattern[(py * 8 + px) as usize] == 1;
                if hit {
                    p[i] = 220; p[i + 1] = 20; p[i + 2] = 60; p[i + 3] = 255;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

fn generate_skin(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 180; p[(i * 4 + 1) as usize] = 130; p[(i * 4 + 2) as usize] = 100; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
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