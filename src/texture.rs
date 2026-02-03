pub struct TextureAtlas {
    pub data: Vec<u8>,
    pub size: u32,
}

impl TextureAtlas {
    pub fn new() -> Self {
        let atlas_width = 256;
        let block_size = 16;
        let total_pixels = atlas_width * atlas_width;
        
        let mut data = vec![0u8; (total_pixels * 4) as usize];
        
        // Initialize transparent
        for i in 0..total_pixels {
            let idx = (i * 4) as usize;
            data[idx] = 0; data[idx+1] = 0; data[idx+2] = 0; data[idx+3] = 0;
        }

        // --- BLOCKS ---
        Self::generate_generic(&mut data, block_size, atlas_width, 0, [0, 0, 0]); // Air
        Self::generate_grass(&mut data, block_size, atlas_width, 1);
        Self::generate_dirt(&mut data, block_size, atlas_width, 2);
        Self::generate_stone(&mut data, block_size, atlas_width, 3);
        Self::generate_wood(&mut data, block_size, atlas_width, 4);
        Self::generate_leaves(&mut data, block_size, atlas_width, 5);
        Self::generate_snow(&mut data, block_size, atlas_width, 6);
        Self::generate_sand(&mut data, block_size, atlas_width, 7);
        Self::generate_bedrock(&mut data, block_size, atlas_width, 8);
        Self::generate_water(&mut data, block_size, atlas_width, 9);

        // --- ORES ---
        Self::generate_ore(&mut data, block_size, atlas_width, 10, [20, 20, 20]);    // Coal
        Self::generate_ore(&mut data, block_size, atlas_width, 11, [200, 150, 100]); // Iron
        Self::generate_ore(&mut data, block_size, atlas_width, 12, [255, 215, 0]);   // Gold
        Self::generate_ore(&mut data, block_size, atlas_width, 13, [0, 255, 255]);   // Diamond

        // --- CRAFTED ---
        Self::generate_generic(&mut data, block_size, atlas_width, 14, [100, 100, 100]); // Planks
        Self::generate_generic(&mut data, block_size, atlas_width, 15, [139, 69, 19]);   // Stick
        Self::generate_generic(&mut data, block_size, atlas_width, 16, [120, 120, 120]); // Cobble

        // --- UI ---
        Self::generate_crosshair(&mut data, block_size, atlas_width, 254);
        Self::generate_hotbar_slot(&mut data, block_size, atlas_width, 250); 
        Self::generate_hotbar_selection(&mut data, block_size, atlas_width, 251);
        Self::generate_heart(&mut data, block_size, atlas_width, 252);
        Self::generate_menu_bg(&mut data, block_size, atlas_width, 253);
        
        // --- FONT ---
        Self::generate_font(&mut data, block_size, atlas_width, 200);

        TextureAtlas { data, size: atlas_width }
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

    // --- BETA STYLE NOISY GENERATORS ---
    fn generate_grass(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            // High contrast noise for that "Beta" feel
            let noise = ((i * 1327) % 55) as u8; 
            p[(i * 4) as usize] = 40 + noise; 
            p[(i * 4 + 1) as usize] = 160 + noise; 
            p[(i * 4 + 2) as usize] = 40 + noise; 
            p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_sand(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            // Gritty sand
            let noise = ((i * 9123) % 40) as u8; 
            p[(i * 4) as usize] = 210 + noise; 
            p[(i * 4 + 1) as usize] = 200 + noise; 
            p[(i * 4 + 2) as usize] = 160 + noise / 2; 
            p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_dirt(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let noise = ((i * 543) % 40) as u8;
            p[(i * 4) as usize] = 100 + noise; p[(i * 4 + 1) as usize] = 60 + noise; p[(i * 4 + 2) as usize] = 20 + noise / 2; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_stone(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let n = ((i * 123) % 30) as u8;
            let c = 110 + n;
            p[(i * 4) as usize] = c; p[(i * 4 + 1) as usize] = c; p[(i * 4 + 2) as usize] = c; p[(i * 4 + 3) as usize] = 255;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_heart(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for y in 0..size {
            for x in 0..size {
                let i = ((y * size + x) * 4) as usize;
                let dx = (x as f32 - 7.5) / 7.5;
                let dy = (y as f32 - 7.5) / -7.5;
                let a = dx*dx + dy*dy - 1.0;
                if a*a*a - dx*dx*dy*dy*dy <= 0.0 {
                    p[i] = 255; p[i+1] = 20; p[i+2] = 20; p[i+3] = 255;
                } else {
                    p[i] = 0; p[i+1] = 0; p[i+2] = 0; p[i+3] = 0;
                }
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_menu_bg(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            let n = i * 4; // FIXED warning
            p[n as usize] = 0; p[n as usize+1] = 0; p[n as usize+2] = 0; p[n as usize+3] = 180;
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    // Keep others simple
    fn generate_wood(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [160, 82, 45]); }
    fn generate_leaves(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [34, 139, 34]); }
    fn generate_snow(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [240, 240, 240]); }
    fn generate_bedrock(data: &mut [u8], size: u32, w: u32, idx: u32) { Self::generate_generic(data, size, w, idx, [20, 20, 20]); }

    fn generate_water(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        for i in 0..size * size {
            p[(i * 4) as usize] = 30; p[(i * 4 + 1) as usize] = 80; p[(i * 4 + 2) as usize] = 200; p[(i * 4 + 3) as usize] = 180;
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

    fn generate_hotbar_slot(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let s_usize = size as usize;
        let mut p = vec![0u8; s_usize * s_usize * 4];
        for i in 0..(s_usize * s_usize) {
            let x = i % s_usize;
            let y = i / s_usize;
            let idx_base = i * 4;
            if x == 0 || x == s_usize - 1 || y == 0 || y == s_usize - 1 {
                p[idx_base] = 80; p[idx_base+1] = 80; p[idx_base+2] = 80; p[idx_base+3] = 255;
            } else {
                p[idx_base] = 0; p[idx_base+1] = 0; p[idx_base+2] = 0; p[idx_base+3] = 120;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }

    fn generate_hotbar_selection(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let s_usize = size as usize;
        let mut p = vec![0u8; s_usize * s_usize * 4];
        for i in 0..(s_usize * s_usize) {
            let x = i % s_usize;
            let y = i / s_usize;
            let idx_base = i * 4;
            if x < 2 || x >= s_usize - 2 || y < 2 || y >= s_usize - 2 {
                p[idx_base] = 255; p[idx_base+1] = 255; p[idx_base+2] = 255; p[idx_base+3] = 255;
            } else {
                p[idx_base] = 0; p[idx_base+1] = 0; p[idx_base+2] = 0; p[idx_base+3] = 0;
            }
        }
        Self::place_texture(data, size, w, idx, &p);
    }
    
    fn generate_crosshair(data: &mut [u8], size: u32, w: u32, idx: u32) {
        let mut p = vec![0u8; (size * size * 4) as usize];
        let c = size as i32 / 2;
        for i in 0..size*size {
            let x = (i % size) as i32;
            let y = (i / size) as i32;
            if (x == c && y > c-4 && y < c+4) || (y == c && x > c-4 && x < c+4) {
                 let b = (i * 4) as usize;
                 p[b] = 255; p[b+1] = 255; p[b+2] = 255; p[b+3] = 255;
            }
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
            [0xE, 0x10, 0x1E, 0x11, 0xE], [0x1F, 0x2, 0x4, 0x8, 0x8], [0xE, 0x11, 0xE, 0x11, 0xE], [0x1E, 0x11, 0x1E, 0x1, 0xE],
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