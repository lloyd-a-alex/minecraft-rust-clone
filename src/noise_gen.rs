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

    // --- BETA TERRAIN LOGIC ---
pub fn get_height(&self, x: i32, z: i32) -> i32 {
        let xf = x as f64;
        let zf = z as f64;

        // DIABOLICAL MULTI-OCTAVE NOISE
        let n1 = self.get_noise3d(xf * 0.003, 0.0, zf * 0.003) * 40.0; // Macro
        let n2 = self.get_noise3d(xf * 0.015, 11.1, zf * 0.015) * 15.0; // Hills
        let n3 = self.get_noise3d(xf * 0.05, 22.2, zf * 0.05) * 4.0;   // Bumps
        
        // Use n1 as a mask for mountains
        let mut h = 64.0 + n1 + n2 + n3;
        
        // If the macro noise is high, sharpen the peaks for "Natural Mountains"
        if n1 > 15.0 {
            h += (n1 - 15.0).powf(1.8) * 0.8;
        }

        h.clamp(10.0, 120.0) as i32
    }

pub fn get_river_noise(&self, x: i32, z: i32) -> f64 {
        // Wider, smoother rivers that don't cut to bedrock
        let val = self.get_noise3d(x as f64 * 0.004, 500.0, z as f64 * 0.004);
        val.abs() 
    }

pub fn get_biome(&self, x: i32, z: i32, height: i32) -> &'static str {
        // Temperature: Low = Cold (Ice/Taiga), High = Hot (Desert)
        let temp = self.get_noise3d(x as f64 * 0.001, 0.0, z as f64 * 0.001);
        // Humidity: Low = Dry (Desert/Plains), High = Wet (Swamp/Jungle)
        let humidity = self.get_noise3d(x as f64 * 0.001, 100.0, z as f64 * 0.001);

        if height > 85 { return "mountain_peak"; } // Snow caps
        
        if temp < -0.3 {
            return "ice_plains";
        } else if temp < 0.0 {
            if humidity > 0.0 { return "taiga"; } else { return "plains"; } // Cold plains
        } else if temp > 0.3 {
            if humidity < -0.2 { return "desert"; } else { return "jungle"; } // Hot
        }
        
        // Moderate temp
        if humidity > 0.3 { return "swamp"; }
        if humidity < -0.2 { return "plains"; }
        "forest"
    }
}