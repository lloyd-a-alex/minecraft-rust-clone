//! DIABOLICAL WEATHER SYSTEM - Advanced Atmospheric Simulation
//! 
//! This module provides comprehensive weather simulation including:
//! - Dynamic weather patterns (rain, snow, thunderstorms, clear skies)
//! - Realistic cloud generation and movement
//! - Temperature and humidity simulation
//! - Weather effects on gameplay (visibility, movement speed, block interactions)
//! - Day/night cycle integration
//! - Seasonal variations

use glam::Vec3;
use crate::world::World;
use crate::minecraft_rendering::{MinecraftRenderer, FogType, ShadingMode};
use crate::time_system::TimeSystem;

/// DIABOLICAL Weather Types with realistic behaviors
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WeatherType {
    Clear,
    Cloudy,
    Rain,
    HeavyRain,
    Thunderstorm,
    Snow,
    Blizzard,
    Fog,
    Sandstorm,
    MagicalStorm,
}

/// DIABOLICAL Wind System with realistic patterns
#[derive(Debug, Clone)]
pub struct WindSystem {
    pub direction: Vec3,
    pub speed: f32,
    pub gust_frequency: f32,
    pub gust_strength: f32,
    pub variation: f32,
    pub target_direction: Vec3,
}

impl WindSystem {
    pub fn new() -> Self {
        Self {
            direction: Vec3::new(1.0, 0.0, 0.0),
            speed: 5.0,
            gust_frequency: 0.1,
            gust_strength: 2.0,
            variation: 0.3,
            target_direction: Vec3::new(1.0, 0.0, 0.0),
        }
    }

    pub fn update(&mut self, dt: f32, weather_type: WeatherType) {
        // Update wind based on weather type
        match weather_type {
            WeatherType::Thunderstorm => {
                self.target_direction = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 2.0,
                    0.2,
                    (rand::random::<f32>() - 0.5) * 2.0
                ).normalize();
                self.speed = 15.0 + rand::random::<f32>() * 10.0;
                self.gust_frequency = 0.05;
                self.gust_strength = 5.0;
            }
            WeatherType::Blizzard => {
                self.target_direction = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 2.0,
                    0.1,
                    (rand::random::<f32>() - 0.5) * 2.0
                ).normalize();
                self.speed = 20.0 + rand::random::<f32>() * 10.0;
                self.gust_frequency = 0.02;
                self.gust_strength = 8.0;
            }
            WeatherType::Sandstorm => {
                self.target_direction = Vec3::new(1.0, 0.1, 0.0);
                self.speed = 25.0 + rand::random::<f32>() * 15.0;
                self.gust_frequency = 0.03;
                self.gust_strength = 10.0;
            }
            _ => {
                self.target_direction = Vec3::new(
                    (rand::random::<f32>() - 0.5) * 2.0,
                    0.0,
                    (rand::random::<f32>() - 0.5) * 2.0
                ).normalize();
                self.speed = 3.0 + rand::random::<f32>() * 5.0;
                self.gust_frequency = 0.2;
                self.gust_strength = 1.0;
            }
        }

        // Smoothly interpolate wind direction
        let lerp_factor = dt * 0.1;
        self.direction = self.direction.lerp(self.target_direction, lerp_factor);

        // Add gusts
        if rand::random::<f32>() < self.gust_frequency {
            let gust_multiplier = 1.0 + (rand::random::<f32>() - 0.5) * self.gust_strength;
            self.speed *= gust_multiplier;
        }

        // Add variation
        self.speed *= 1.0 + (rand::random::<f32>() - 0.5) * self.variation;
    }
}

/// DIABOLICAL Cloud System with realistic formation and movement
#[derive(Debug, Clone)]
pub struct Cloud {
    pub position: Vec3,
    pub size: Vec3,
    pub density: f32,
    pub height: f32,
    pub movement_speed: Vec3,
    pub precipitation: Option<PrecipitationType>,
    pub color: [f32; 4],
    pub particles: Vec<CloudParticle>,
}

#[derive(Debug, Clone)]
pub struct CloudParticle {
    pub position: Vec3,
    pub velocity: Vec3,
    pub lifetime: f32,
    pub size: f32,
    pub type_: CloudParticleType,
}

#[derive(Debug, Clone)]
pub enum CloudParticleType {
    Water,
    Snow,
    Dust,
    Magical,
}

#[derive(Debug, Clone)]
pub enum PrecipitationType {
    Rain { intensity: f32 },
    Snow { intensity: f32 },
    Hail { size: f32 },
    Magical { element: String },
    Dust,
}

impl Cloud {
    pub fn new(position: Vec3, weather_type: WeatherType) -> Self {
        let size = Vec3::new(
            50.0 + rand::random::<f32>() * 100.0,
            10.0 + rand::random::<f32>() * 20.0,
            50.0 + rand::random::<f32>() * 100.0
        );

        let (color, density, height, precipitation) = match weather_type {
            WeatherType::Clear => ([0.8, 0.8, 0.9, 0.3], 0.2, 100.0, None),
            WeatherType::Cloudy => ([0.6, 0.6, 0.7, 0.5], 0.4, 120.0, None),
            WeatherType::Rain | WeatherType::HeavyRain => {
                ([0.4, 0.4, 0.5, 0.7], 0.8, 80.0, Some(PrecipitationType::Rain { 
                    intensity: if weather_type == WeatherType::HeavyRain { 0.8 } else { 0.3 } 
                }))
            },
            WeatherType::Thunderstorm => {
                ([0.3, 0.3, 0.4, 0.9], 0.9, 60.0, Some(PrecipitationType::Rain { intensity: 0.9 }))
            },
            WeatherType::Snow | WeatherType::Blizzard => {
                ([0.9, 0.9, 1.0, 0.6], 0.7, 90.0, Some(PrecipitationType::Snow { 
                    intensity: if weather_type == WeatherType::Blizzard { 0.8 } else { 0.3 } 
                }))
            },
            WeatherType::Fog => ([0.7, 0.7, 0.8, 0.4], 0.3, 30.0, None),
            WeatherType::Sandstorm => {
                ([0.8, 0.7, 0.5, 0.5], 0.6, 40.0, Some(PrecipitationType::Dust))
            },
            WeatherType::MagicalStorm => {
                ([0.6, 0.2, 0.8, 0.8], 0.7, 70.0, Some(PrecipitationType::Magical { 
                    element: "arcane".to_string() 
                }))
            },
        };

        Self {
            position,
            size,
            density,
            height,
            movement_speed: Vec3::new(
                (rand::random::<f32>() - 0.5) * 2.0,
                0.0,
                (rand::random::<f32>() - 0.5) * 2.0
            ),
            precipitation,
            color,
            particles: Vec::new(),
        }
    }

    pub fn update(&mut self, dt: f32, wind: &WindSystem) {
        // Move cloud with wind
        self.position += wind.direction * wind.speed * dt * 0.1;
        
        // Update precipitation particles
        if let Some(precipitation) = &self.precipitation {
            match precipitation {
                PrecipitationType::Rain { intensity } => {
                    // Generate rain particles
                    let particle_count = (*intensity * 100.0 * dt) as usize;
                    for _ in 0..particle_count.min(10) {
                        let particle_pos = self.position + Vec3::new(
                            (rand::random::<f32>() - 0.5) * self.size.x,
                            self.height,
                            (rand::random::<f32>() - 0.5) * self.size.z
                        );
                        
                        self.particles.push(CloudParticle {
                            position: particle_pos,
                            velocity: Vec3::new(0.0, -9.8, 0.0) + wind.direction * wind.speed * 0.1,
                            lifetime: 3.0,
                            size: 0.1,
                            type_: CloudParticleType::Water,
                        });
                    }
                }
                PrecipitationType::Snow { intensity } => {
                    // Generate snow particles
                    let particle_count = (*intensity * 50.0 * dt) as usize;
                    for _ in 0..particle_count.min(5) {
                        let particle_pos = self.position + Vec3::new(
                            (rand::random::<f32>() - 0.5) * self.size.x,
                            self.height,
                            (rand::random::<f32>() - 0.5) * self.size.z
                        );
                        
                        self.particles.push(CloudParticle {
                            position: particle_pos,
                            velocity: Vec3::new(
                                wind.direction.x * 2.0,
                                -1.0 + wind.direction.y * 0.5,
                                wind.direction.z * 2.0
                            ),
                            lifetime: 5.0,
                            size: 0.05,
                            type_: CloudParticleType::Snow,
                        });
                    }
                }
                PrecipitationType::Hail { size } => {
                    // Generate hail particles
                    let particle_count = (10.0 * dt) as usize;
                    for _ in 0..particle_count.min(2) {
                        let particle_pos = self.position + Vec3::new(
                            (rand::random::<f32>() - 0.5) * self.size.x,
                            self.height,
                            (rand::random::<f32>() - 0.5) * self.size.z
                        );
                        
                        self.particles.push(CloudParticle {
                            position: particle_pos,
                            velocity: Vec3::new(0.0, -20.0, 0.0),
                            lifetime: 2.0,
                            size: *size,
                            type_: CloudParticleType::Water,
                        });
                    }
                }
                PrecipitationType::Dust => {
                    // Generate dust particles
                    let particle_count = (20.0 * dt) as usize;
                    for _ in 0..particle_count.min(3) {
                        let particle_pos = self.position + Vec3::new(
                            (rand::random::<f32>() - 0.5) * self.size.x,
                            self.height + rand::random::<f32>() * 10.0,
                            (rand::random::<f32>() - 0.5) * self.size.z
                        );
                        
                        self.particles.push(CloudParticle {
                            position: particle_pos,
                            velocity: wind.direction * wind.speed * 0.2 + Vec3::new(
                                (rand::random::<f32>() - 0.5) * 3.0,
                                rand::random::<f32>() * 2.0,
                                (rand::random::<f32>() - 0.5) * 3.0
                            ),
                            lifetime: 8.0,
                            size: 0.2,
                            type_: CloudParticleType::Dust,
                        });
                    }
                }
                PrecipitationType::Magical { element: _ } => {
                    // Generate magical particles
                    let particle_count = (10.0 * dt) as usize;
                    for _ in 0..particle_count.min(5) {
                        let particle_pos = self.position + Vec3::new(
                            (rand::random::<f32>() - 0.5) * self.size.x,
                            self.height,
                            (rand::random::<f32>() - 0.5) * self.size.z
                        );
                        
                        self.particles.push(CloudParticle {
                            position: particle_pos,
                            velocity: Vec3::new(
                                (rand::random::<f32>() - 0.5) * 2.0,
                                -2.0 - rand::random::<f32>() * 3.0,
                                (rand::random::<f32>() - 0.5) * 2.0
                            ),
                            lifetime: 2.0 + rand::random::<f32>() * 3.0,
                            size: 0.15,
                            type_: CloudParticleType::Magical,
                        });
                    }
                }
            }
        }

        // Update existing particles
        self.particles.retain_mut(|particle| {
            particle.position += particle.velocity * dt;
            particle.lifetime -= dt;
            particle.lifetime > 0.0
        });
    }
}

/// DIABOLICAL Atmospheric Conditions
#[derive(Debug, Clone)]
pub struct AtmosphericConditions {
    pub temperature: f32,
    pub humidity: f32,
    pub pressure: f32,
    pub visibility: f32,
    pub uv_index: f32,
    pub air_quality: f32,
}

impl AtmosphericConditions {
    pub fn new() -> Self {
        Self {
            temperature: 20.0,
            humidity: 0.5,
            pressure: 1013.25,
            visibility: 1.0,
            uv_index: 5.0,
            air_quality: 1.0,
        }
    }

    pub fn update(&mut self, weather_type: WeatherType, time_of_day: f32) {
        match weather_type {
            WeatherType::Clear => {
                self.temperature = 20.0 + time_of_day * 10.0 * (if time_of_day < 0.5 { -1.0 } else { 1.0 });
                self.humidity = 0.3;
                self.visibility = 1.0;
                self.uv_index = 8.0 * (1.0 - time_of_day * 0.7);
            }
            WeatherType::Cloudy => {
                self.temperature = 18.0 + time_of_day * 8.0 * (if time_of_day < 0.5 { -1.0 } else { 1.0 });
                self.humidity = 0.6;
                self.visibility = 0.8;
                self.uv_index = 4.0 * (1.0 - time_of_day * 0.7);
            }
            WeatherType::Rain | WeatherType::HeavyRain => {
                self.temperature = 15.0 + time_of_day * 5.0 * (if time_of_day < 0.5 { -1.0 } else { 1.0 });
                self.humidity = 0.9;
                self.visibility = if weather_type == WeatherType::HeavyRain { 0.3 } else { 0.6 };
                self.uv_index = 2.0 * (1.0 - time_of_day * 0.7);
            }
            WeatherType::Thunderstorm => {
                self.temperature = 16.0;
                self.humidity = 0.95;
                self.visibility = 0.4;
                self.uv_index = 1.0;
            }
            WeatherType::Snow | WeatherType::Blizzard => {
                self.temperature = -5.0 + time_of_day * 3.0 * (if time_of_day < 0.5 { -1.0 } else { 1.0 });
                self.humidity = 0.7;
                self.visibility = if weather_type == WeatherType::Blizzard { 0.2 } else { 0.7 };
                self.uv_index = 1.0;
            }
            WeatherType::Fog => {
                self.temperature = 12.0;
                self.humidity = 0.8;
                self.visibility = 0.1;
                self.uv_index = 3.0;
            }
            WeatherType::Sandstorm => {
                self.temperature = 35.0;
                self.humidity = 0.1;
                self.visibility = 0.3;
                self.uv_index = 10.0;
                self.air_quality = 0.3;
            }
            WeatherType::MagicalStorm => {
                self.temperature = 25.0;
                self.humidity = 0.8;
                self.visibility = 0.5;
                self.uv_index = 15.0;
                self.air_quality = 0.5;
            }
        }
    }
}

/// DIABOLICAL Weather System - Main weather controller
pub struct WeatherSystem {
    pub current_weather: WeatherType,
    pub target_weather: WeatherType,
    pub transition_progress: f32,
    pub wind: WindSystem,
    pub clouds: Vec<Cloud>,
    pub atmospheric: AtmosphericConditions,
    pub time_of_day: f32,
    pub season: Season,
    pub weather_duration: f32,
    pub time_until_change: f32,
    pub lightning_strikes: Vec<LightningStrike>,
    pub ambient_light_color: [f32; 3],
    pub fog_density: f32,
    pub minecraft_renderer: Option<MinecraftRenderer>,
    pub time_system: Option<TimeSystem>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

#[derive(Debug, Clone)]
pub struct LightningStrike {
    pub position: Vec3,
    pub time_to_strike: f32,
    pub intensity: f32,
    pub branches: Vec<Vec3>,
}

impl WeatherSystem {
    pub fn new() -> Self {
        Self {
            current_weather: WeatherType::Clear,
            target_weather: WeatherType::Clear,
            transition_progress: 0.0,
            wind: WindSystem::new(),
            clouds: Vec::new(),
            atmospheric: AtmosphericConditions::new(),
            time_of_day: 0.5, // Noon
            season: Season::Summer,
            weather_duration: 300.0, // 5 minutes per weather type
            time_until_change: 300.0,
            lightning_strikes: Vec::new(),
            ambient_light_color: [1.0, 1.0, 1.0],
            fog_density: 0.0,
            minecraft_renderer: None,
            time_system: None,
        }
    }

    pub fn with_classic_rendering(&mut self, minecraft_renderer: MinecraftRenderer, time_system: TimeSystem) {
        self.minecraft_renderer = Some(minecraft_renderer);
        self.time_system = Some(time_system);
    }

    pub fn update(&mut self, dt: f32, world: &World, player_position: Vec3) {
        // Update time of day (24 hour cycle)
        self.time_of_day = (self.time_of_day + dt / 86400.0) % 1.0;
        
        // Update season (30 day cycle)
        let day_of_year = (world.seed as u32 % 365) as f32;
        self.season = match (day_of_year / 91.25) as u8 {
            0 => Season::Spring,
            1 => Season::Summer,
            2 => Season::Autumn,
            _ => Season::Winter,
        };

        // Update weather transitions
        self.time_until_change -= dt;
        if self.time_until_change <= 0.0 {
            self.change_weather();
        }

        if self.current_weather != self.target_weather {
            self.transition_progress = (self.transition_progress + dt / 30.0).min(1.0);
        } else {
            self.transition_progress = 0.0;
        }

        // Update wind system
        self.wind.update(dt, self.current_weather);

        // Update clouds
        for cloud in &mut self.clouds {
            cloud.update(dt, &self.wind);
        }

        // Remove distant clouds
        self.clouds.retain(|cloud| {
            (cloud.position - player_position).length() < 500.0
        });

        // Generate new clouds
        if self.clouds.len() < 10 && rand::random::<f32>() < 0.02 {
            let cloud_pos = player_position + Vec3::new(
                (rand::random::<f32>() - 0.5) * 400.0,
                100.0 + rand::random::<f32>() * 50.0,
                (rand::random::<f32>() - 0.5) * 400.0
            );
            self.clouds.push(Cloud::new(cloud_pos, self.target_weather));
        }

        // Update atmospheric conditions
        self.atmospheric.update(self.current_weather, self.time_of_day);

        // Update lightning strikes for thunderstorms
        if self.current_weather == WeatherType::Thunderstorm {
            self.update_lightning(dt, player_position);
        }

        // Update ambient lighting
        self.update_ambient_lighting();

        // Update fog
        self.update_fog();
    }

    fn change_weather(&mut self) {
        // Generate new weather based on season and current conditions
        let weather_options = match self.season {
            Season::Spring => vec![
                WeatherType::Clear, WeatherType::Cloudy, WeatherType::Rain, WeatherType::Thunderstorm
            ],
            Season::Summer => vec![
                WeatherType::Clear, WeatherType::Cloudy, WeatherType::Thunderstorm
            ],
            Season::Autumn => vec![
                WeatherType::Clear, WeatherType::Cloudy, WeatherType::Rain, WeatherType::Fog
            ],
            Season::Winter => vec![
                WeatherType::Clear, WeatherType::Cloudy, WeatherType::Snow, WeatherType::Blizzard
            ],
        };

        let idx = (rand::random::<u32>() as usize) % weather_options.len();
        self.target_weather = weather_options[idx];
        self.time_until_change = self.weather_duration * (0.5 + rand::random::<f32>() * 2.0);
        
        log::info!("Weather changing to {:?}", self.target_weather);
    }

    fn update_lightning(&mut self, dt: f32, player_position: Vec3) {
        // Update existing lightning strikes
        self.lightning_strikes.retain_mut(|strike| {
            strike.time_to_strike -= dt;
            strike.time_to_strike > 0.0
        });

        // Generate new lightning strikes
        if rand::random::<f32>() < 0.1 {
            let strike_pos = player_position + Vec3::new(
                (rand::random::<f32>() - 0.5) * 200.0,
                50.0 + rand::random::<f32>() * 100.0,
                (rand::random::<f32>() - 0.5) * 200.0
            );
            
            let mut branches = Vec::new();
            let branch_count = (rand::random::<u32>() % 4) as usize;
            for _ in 0..branch_count {
                branches.push(strike_pos + Vec3::new(
                    (rand::random::<f32>() - 0.5) * 50.0,
                    -rand::random::<f32>() * 30.0,
                    (rand::random::<f32>() - 0.5) * 50.0
                ));
            }

            self.lightning_strikes.push(LightningStrike {
                position: strike_pos,
                time_to_strike: rand::random::<f32>() * 2.0,
                intensity: 0.5 + rand::random::<f32>() * 0.5,
                branches,
            });
        }
    }

    fn update_ambient_lighting(&mut self) {
        // Calculate ambient light color based on weather and time of day
        let base_intensity = (self.time_of_day * std::f32::consts::PI).cos() * 0.5 + 0.5;
        
        let (r, g, b) = match self.current_weather {
            WeatherType::Clear => (1.0, 1.0, 0.9),
            WeatherType::Cloudy => (0.8, 0.8, 0.9),
            WeatherType::Rain => (0.6, 0.6, 0.7),
            WeatherType::HeavyRain => (0.4, 0.4, 0.5),
            WeatherType::Thunderstorm => (0.3, 0.3, 0.4),
            WeatherType::Snow => (0.9, 0.9, 1.0),
            WeatherType::Blizzard => (0.7, 0.7, 0.8),
            WeatherType::Fog => (0.7, 0.7, 0.8),
            WeatherType::Sandstorm => (0.9, 0.8, 0.6),
            WeatherType::MagicalStorm => (0.6, 0.3, 0.8),
        };

        let intensity = base_intensity * self.atmospheric.visibility;
        self.ambient_light_color = [
            r * intensity,
            g * intensity,
            b * intensity,
        ];
    }

    fn update_fog(&mut self) {
        match self.current_weather {
            WeatherType::Fog => {
                self.fog_density = 0.8;
            }
            WeatherType::Rain | WeatherType::HeavyRain => {
                self.fog_density = 0.3;
            }
            WeatherType::Blizzard => {
                self.fog_density = 0.6;
            }
            _ => {
                self.fog_density = 0.0;
            }
        }
    }

    pub fn get_weather_effects(&self) -> WeatherEffects {
        WeatherEffects {
            visibility_modifier: self.atmospheric.visibility,
            movement_speed_modifier: self.get_movement_speed_modifier(),
            block_interaction_modifier: self.get_block_interaction_modifier(),
            ambient_light_color: self.ambient_light_color,
            fog_density: self.fog_density,
            precipitation_type: self.current_weather.get_precipitation_type(),
        }
    }

    fn get_movement_speed_modifier(&self) -> f32 {
        match self.current_weather {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 0.95,
            WeatherType::Rain => 0.8,
            WeatherType::HeavyRain => 0.6,
            WeatherType::Thunderstorm => 0.5,
            WeatherType::Snow => 0.7,
            WeatherType::Blizzard => 0.4,
            WeatherType::Fog => 0.5,
            WeatherType::Sandstorm => 0.3,
            WeatherType::MagicalStorm => 0.8,
        }
    }

    fn get_block_interaction_modifier(&self) -> f32 {
        match self.current_weather {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 1.0,
            WeatherType::Rain => 0.9,
            WeatherType::HeavyRain => 0.7,
            WeatherType::Thunderstorm => 0.5,
            WeatherType::Snow => 0.8,
            WeatherType::Blizzard => 0.6,
            WeatherType::Fog => 0.5,
            WeatherType::Sandstorm => 0.4,
            WeatherType::MagicalStorm => 1.2,
        }
    }

    // Classic Minecraft Rendering Integration
    pub fn update_classic_rendering(&mut self) {
        // Get fog settings first to avoid borrow issues
        let fog_settings = match self.current_weather {
            WeatherType::Clear => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.7, 0.7, 0.8, 1.0],
                fog_end: 6.0,
            },
            WeatherType::Cloudy => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.6, 0.6, 0.7, 1.0],
                fog_end: 5.0,
            },
            WeatherType::Rain => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.4, 0.4, 0.5, 1.0],
                fog_end: 4.0,
            },
            WeatherType::HeavyRain => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.3, 0.3, 0.4, 1.0],
                fog_end: 3.0,
            },
            WeatherType::Thunderstorm => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.2, 0.2, 0.3, 1.0],
                fog_end: 2.5,
            },
            WeatherType::Snow => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.8, 0.8, 0.9, 1.0],
                fog_end: 5.5,
            },
            WeatherType::Blizzard => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.7, 0.7, 0.8, 1.0],
                fog_end: 3.0,
            },
            WeatherType::Fog => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.5, 0.5, 0.6, 1.0],
                fog_end: 2.0,
            },
            WeatherType::Sandstorm => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.8, 0.7, 0.5, 1.0],
                fog_end: 1.5,
            },
            WeatherType::MagicalStorm => FogSettings {
                fog_type: FogType::Classic,
                fog_color: [0.3, 0.1, 0.5, 1.0],
                fog_end: 1.0,
            },
        };
        
        if let Some(renderer) = &mut self.minecraft_renderer {
            // Apply all settings at once
            renderer.set_fog_type(fog_settings.fog_type);
            renderer.fog_color = [
                fog_settings.fog_color[0],
                fog_settings.fog_color[1], 
                fog_settings.fog_color[2], 
                fog_settings.fog_color[3]
            ];
            renderer.fog_end = fog_settings.fog_end;
        }
    }

    
fn update_sky_color(&mut self) {
    let time_modifier = if let Some(ref time_system) = self.time_system {
        let sky_color: [f32; 3] = time_system.get_sky_color();
        sky_color[0] + sky_color[1] + sky_color[2]
    } else {
        2.1 // Default bright sky
    };

    let weather_color = match self.current_weather {
        WeatherType::Clear => [time_modifier, time_modifier, time_modifier],
        WeatherType::Cloudy => [time_modifier * 0.8, time_modifier * 0.8, time_modifier * 0.9],
        WeatherType::Rain => [time_modifier * 0.4, time_modifier * 0.4, time_modifier * 0.5],
        WeatherType::HeavyRain => [time_modifier * 0.3, time_modifier * 0.3, time_modifier * 0.4],
        WeatherType::Thunderstorm => [time_modifier * 0.2, time_modifier * 0.2, time_modifier * 0.3],
        WeatherType::Snow => [time_modifier * 0.9, time_modifier * 0.9, time_modifier * 1.0],
        WeatherType::Blizzard => [time_modifier * 0.7, time_modifier * 0.7, time_modifier * 0.8],
        WeatherType::Fog => [time_modifier * 0.6, time_modifier * 0.6, time_modifier * 0.7],
        WeatherType::Sandstorm => [time_modifier * 0.8, time_modifier * 0.7, time_modifier * 0.5],
        WeatherType::MagicalStorm => [time_modifier * 0.3, time_modifier * 0.1, time_modifier * 0.5],
    };

    if let Some(ref mut renderer) = self.minecraft_renderer {
        renderer.fog_color[0] = weather_color[0];
        renderer.fog_color[1] = weather_color[1];
        renderer.fog_color[2] = weather_color[2];
    }
}

fn update_fog(&mut self) {
    match self.current_weather {
        WeatherType::Fog => {
            self.fog_density = 0.8;
        }
        WeatherType::Rain | WeatherType::HeavyRain => {
            self.fog_density = 0.3;
        }
        WeatherType::Blizzard => {
            self.fog_density = 0.6;
        }
        _ => {
            self.fog_density = 0.0;
        }
    }
}
}

pub fn get_classic_fog_density(&self) -> f32 {
    if let Some(ref renderer) = self.minecraft_renderer {
        renderer.get_fog_density(10.0_f32) // Sample at 10 blocks distance
    } else {
        self.fog_density
    }
}

pub fn get_classic_ambient_light(&self) -> f32 {
    if let Some(ref time_system) = self.time_system {
        let base_light: f32 = time_system.get_ambient_light();
        let weather_modifier = match self.current_weather {
            WeatherType::Clear => 1.0,
            WeatherType::Cloudy => 0.9,
            WeatherType::Rain => 0.7,
            WeatherType::HeavyRain => 0.5,
            WeatherType::Thunderstorm => 0.4,
            WeatherType::Snow => 0.8,
            WeatherType::Blizzard => 0.6,
            WeatherType::Fog => 0.6,
            WeatherType::Sandstorm => 0.5,
            WeatherType::MagicalStorm => 0.3,
        };
        base_light * weather_modifier
    } else {
        1.0
    }
}

pub fn should_use_classic_lighting(&self) -> bool {
    self.minecraft_renderer.is_some()
}

pub fn get_texture_filter_mode(&self) -> Option<u32> {
    self.minecraft_renderer.as_ref().map(|r: &crate::minecraft_rendering::MinecraftRenderer| r.get_texture_filter_mode())
}

pub fn should_use_mipmapping(&self) -> bool {
    self.minecraft_renderer.as_ref().map_or(false, |r: &crate::minecraft_rendering::MinecraftRenderer| r.should_use_mipmapping())
}

pub fn apply_view_bobbing(&self, time: f32) -> Option<Vec3> {
    self.minecraft_renderer.as_ref().map(|r| r.apply_view_bobbing(time))
}

pub fn get_weather_effects(&self) -> WeatherEffects {
    WeatherEffects {
        visibility_modifier: self.atmospheric.visibility,
        movement_speed_modifier: self.get_movement_speed_modifier(),
        block_interaction_modifier: self.get_block_interaction_modifier(),
        ambient_light_color: self.ambient_light_color,
        fog_density: self.fog_density,
        precipitation_type: self.current_weather.get_precipitation_type(),
    }
}

fn get_movement_speed_modifier(&self) -> f32 {
    match self.current_weather {
        WeatherType::Clear => 1.0,
        WeatherType::Cloudy => 0.95,
        WeatherType::Rain => 0.8,
        WeatherType::HeavyRain => 0.7,
        WeatherType::Thunderstorm => 0.6,
        WeatherType::Snow => 0.9,
        WeatherType::Blizzard => 0.5,
        WeatherType::Fog => 0.8,
        WeatherType::Sandstorm => 0.3,
        WeatherType::MagicalStorm => 0.8,
    }
}

fn get_block_interaction_modifier(&self) -> f32 {
    match self.current_weather {
        WeatherType::Clear => 1.0,
        WeatherType::Cloudy => 1.0,
        WeatherType::Rain => 1.0,
        WeatherType::HeavyRain => 1.0,
        WeatherType::Thunderstorm => 1.0,
        WeatherType::Snow => 0.9,
        WeatherType::Blizzard => 0.6,
        WeatherType::Fog => 1.0,
        WeatherType::Sandstorm => 0.4,
        WeatherType::MagicalStorm => 1.2,
    }
}

fn get_movement_speed_modifier(&self) -> f32 {
    match self.current_weather {
        WeatherType::Clear => 1.0,
        WeatherType::Cloudy => 0.95,
        WeatherType::Rain => 0.8,
        WeatherType::HeavyRain => 0.7,
        WeatherType::Thunderstorm => 0.6,
        WeatherType::Snow => 0.9,
        WeatherType::Blizzard => 0.5,
        WeatherType::Fog => 0.8,
        WeatherType::Sandstorm => 0.3,
        WeatherType::MagicalStorm => 0.8,
    }
}

fn get_block_interaction_modifier(&self) -> f32 {
    match self.current_weather {
        WeatherType::Clear => 1.0,
        WeatherType::Cloudy => 1.0,
        WeatherType::Rain => 1.0,
        WeatherType::HeavyRain => 1.0,
        WeatherType::Thunderstorm => 1.0,
        WeatherType::Snow => 0.9,
        WeatherType::Blizzard => 0.6,
        WeatherType::Fog => 1.0,
        WeatherType::Sandstorm => 0.4,
        WeatherType::MagicalStorm => 1.2,
    }
}

#[derive(Debug, Clone)]
struct FogSettings {
    pub fog_type: FogType,
    pub fog_color: [f32; 4],
    pub fog_end: f32,
}

#[derive(Debug, Clone)]
struct AmbientLightingSettings {
    pub ambient_light: f32,
}

#[derive(Debug, Clone)]
struct SkyColorSettings {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl WeatherType {
    pub fn get_precipitation_type(&self) -> Option<PrecipitationType> {
        match self {
            WeatherType::Rain => Some(PrecipitationType::Rain { intensity: 0.5 }),
            WeatherType::HeavyRain => Some(PrecipitationType::Rain { intensity: 0.8 }),
            WeatherType::Snow => Some(PrecipitationType::Snow { intensity: 0.3 }),
            WeatherType::Blizzard => Some(PrecipitationType::Snow { intensity: 0.8 }),
            WeatherType::Sandstorm => Some(PrecipitationType::Dust),
            WeatherType::MagicalStorm => Some(PrecipitationType::Magical { element: "arcane".to_string() }),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeatherEffects {
    pub visibility_modifier: f32,
    pub movement_speed_modifier: f32,
    pub block_interaction_modifier: f32,
    pub ambient_light_color: [f32; 3],
    pub fog_density: f32,
    pub precipitation_type: Option<PrecipitationType>,
}
