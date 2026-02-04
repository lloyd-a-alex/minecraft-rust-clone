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
        Self::generate_grass(&mut data, block_size, atlas_width, 1);
        Self::generate_dirt(&mut data, block_size, atlas_width, 2);
        Self::generate_stone(&mut data, block_size, atlas_width, 3);
        Self::generate_wood(&mut data, block_size, atlas_width, 4);
        Self::generate_leaves(&mut data, block_size, atlas_width, 5);
        Self::generate_snow(&mut data, block_size, atlas_width, 6);
        Self::generate_sand(&mut data, block_size, atlas_width, 7);
        Self::generate_bedrock(&mut data, block_size, atlas_width, 8);

// --- TOOLS & ITEMS (Existing code...) ---
        
        // 100. CRAFTING TABLE (Index 21)
        let idx = 21;
        for y in 0..block_size {
            for x in 0..block_size {
                let i = ((idx / grid_width_in_blocks) * block_size + y) * atlas_width + ((idx % grid_width_in_blocks) * block_size + x);
                let pixel_idx = (i * 4) as usize;
                // Grid pattern for crafting table
                let border = x == 0 || x == block_size-1 || y == 0 || y == block_size-1 || x == block_size/2 || y == block_size/2;
                if border {
                    data[pixel_idx] = 60; data[pixel_idx+1] = 40; data[pixel_idx+2] = 10; data[pixel_idx+3] = 255;
                } else {
                    data[pixel_idx] = 160; data[pixel_idx+1] = 110; data[pixel_idx+2] = 60; data[pixel_idx+3] = 255;
                }
            }
        }
        Self::generate_water(&mut data, block_size, atlas_width, 9);

        // --- NEW BLOCKS ---
        Self::generate_torch(&mut data, block_size, atlas_width, 24);
        Self::generate_generic(&mut data, block_size, atlas_width, 14, [100, 100, 100]); // Cobble
        Self::generate_generic(&mut data, block_size, atlas_width, 15, [200, 150, 100]); // Planks
        
        // Ores (Indices 17-20)
        Self::generate_ore(&mut data, block_size, atlas_width, 17, [20, 20, 20]); // Coal
        Self::generate_ore(&mut data, block_size, atlas_width, 18, [200, 150, 100]); // Iron
        Self::generate_ore(&mut data, block_size, atlas_width, 19, [255, 215, 0]); // Gold
        Self::generate_ore(&mut data, block_size, atlas_width, 20, [0, 255, 255]); // Diamond

        // Items/Tools (Simplified Placeholders)
        Self::generate_generic(&mut data, block_size, atlas_width, 40, [100, 50, 0]); // Stick
        Self::generate_generic(&mut data, block_size, atlas_width, 42, [180, 180, 180]); // Iron Ingot
        Self::generate_generic(&mut data, block_size, atlas_width, 44, [0, 255, 255]); // Diamond Item
        
        // Tools (Wood)
        for i in 50..54 { Self::generate_tool(&mut data, block_size, atlas_width, i, [150, 100, 50]); }
        // Stone
        for i in 60..64 { Self::generate_tool(&mut data, block_size, atlas_width, i, [100, 100, 100]); }
        // Iron
        for i in 70..74 { Self::generate_tool(&mut data, block_size, atlas_width, i, [200, 200, 200]); }

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
            [0x11, 0x11, 0xA, 0x4, 0x4], [0x1F, 0x2, 0x4, 0x8, 0x1F], [0xE, 0x11, 0x13, 0x15, 0xE], [0x4, 0xC, 0x4, 0x4, 0xE],
            [0xE, 0x11, 0x2, 0x4, 0x1F], [0xE, 0x11, 0x6, 0x11, 0xE], [0x11, 0x11, 0x1F, 0x4, 0x4], [0x1F, 0x10, 0x1E, 0x1, 0x1E],
            [0xE, 0x10, 0x1E, 0x11, 0xE], [0x1F, 0x2, 0x4, 0x8, 0x8], [0xE, 0x11, 0xE, 0x11, 0xE], [0xE, 0x11, 0x1E, 0x1, 0xE],
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