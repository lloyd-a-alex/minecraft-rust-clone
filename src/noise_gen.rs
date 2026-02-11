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