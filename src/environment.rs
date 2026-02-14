// DIABOLICAL WEATHER SYSTEM - Advanced Atmospheric Simulation
// 
// This module provides comprehensive weather simulation including:
// - Dynamic weather patterns (rain, snow, thunderstorms, clear skies)
// - Realistic cloud generation and movement
// - Temperature and humidity simulation
// - Weather effects on gameplay (visibility, movement speed, block interactions)
// - Day/night cycle integration
// - Seasonal variations

use glam::Vec3;
use crate::graphics::{MinecraftRenderer, FogType};
use crate::resources::NoiseGenerator;
use crate::engine::Chunk;

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

        // Update sky color based on weather
        self.update_sky_color();

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
            let renderer: &mut crate::graphics::MinecraftRenderer = renderer;
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
        let sky_color: [f32; 4] = time_system.get_sky_color();
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


}
#[derive(Debug, Clone)]
struct FogSettings {
    pub fog_type: FogType,
    pub fog_color: [f32; 4],
    pub fog_end: f32,
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

use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TimeOfDay {
    Dawn,
    Morning,
    Noon,
    Afternoon,
    Dusk,
    Night,
    Midnight,
}

#[derive(Debug, Clone)]
pub struct TimeSystem {
    pub current_time: f64, // Time in ticks (0-24000)
    pub day_count: u32,
    pub time_scale: f64, // How fast time passes (1.0 = normal)
    pub last_update: Instant,
    pub is_paused: bool,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            current_time: 6000.0, // Start at noon (12:00)
            day_count: 0,
            time_scale: 1.0,
            last_update: Instant::now(),
            is_paused: false,
        }
    }

    pub fn update(&mut self, delta_time: f64) {
        if self.is_paused {
            return;
        }

        self.current_time += delta_time * self.time_scale * 20.0; // 20 ticks per second at 1x speed
        
        // Handle day transitions
        if self.current_time >= 24000.0 {
            self.current_time -= 24000.0;
            self.day_count += 1;
        }
    }

    pub fn get_time_of_day(&self) -> TimeOfDay {
        let time = self.current_time;
        
        if time < 2000.0 {
            TimeOfDay::Night
        } else if time < 4000.0 {
            TimeOfDay::Dawn
        } else if time < 8000.0 {
            TimeOfDay::Morning
        } else if time < 12000.0 {
            TimeOfDay::Noon
        } else if time < 16000.0 {
            TimeOfDay::Afternoon
        } else if time < 18000.0 {
            TimeOfDay::Dusk
        } else {
            TimeOfDay::Night
        }
    }

    pub fn get_sky_color(&self) -> [f32; 4] {
        let time = self.current_time;
        let (base_color, intensity) = match self.get_time_of_day() {
            TimeOfDay::Dawn => {
                // Dawn: Orange to blue gradient
                let t = (time - 2000.0) / 2000.0; // 0 to 1 during dawn
                let orange = [1.0, 0.6, 0.3];
                let blue = [0.5, 0.8, 1.0];
                let color = [
                    orange[0] * (1.0 - t) + blue[0] * t,
                    orange[1] * (1.0 - t) + blue[1] * t,
                    orange[2] * (1.0 - t) + blue[2] * t,
                ];
                (color, 0.3 + t * 0.7)
            }
            TimeOfDay::Morning => {
                // Morning: Bright blue sky
                ([0.5, 0.8, 1.0], 1.0)
            }
            TimeOfDay::Noon => {
                // Noon: Brightest blue sky
                ([0.4, 0.7, 1.0], 1.0)
            }
            TimeOfDay::Afternoon => {
                // Afternoon: Slightly warmer blue
                ([0.6, 0.8, 0.9], 1.0)
            }
            TimeOfDay::Dusk => {
                // Dusk: Blue to orange gradient
                let t = (time - 16000.0) / 2000.0; // 0 to 1 during dusk
                let blue = [0.5, 0.8, 1.0];
                let orange = [1.0, 0.4, 0.2];
                let color = [
                    blue[0] * (1.0 - t) + orange[0] * t,
                    blue[1] * (1.0 - t) + orange[1] * t,
                    blue[2] * (1.0 - t) + orange[2] * t,
                ];
                (color, 0.7 * (1.0 - t) + 0.3 * t)
            }
            TimeOfDay::Night => {
                // Night: Dark blue to black
                let t = if time < 2000.0 {
                    time / 2000.0 // 0 to 1 from midnight to dawn
                } else {
                    (time - 18000.0) / 6000.0 // 0 to 1 from dusk to midnight
                };
                let dark_blue = [0.1, 0.1, 0.3];
                let black = [0.05, 0.05, 0.1];
                let color = [
                    dark_blue[0] * (1.0 - t) + black[0] * t,
                    dark_blue[1] * (1.0 - t) + black[1] * t,
                    dark_blue[2] * (1.0 - t) + black[2] * t,
                ];
                (color, 0.1)
            }
            TimeOfDay::Midnight => {
                // Midnight: Darkest
                ([0.05, 0.05, 0.1], 0.05)
            }
        };

        [base_color[0] as f32, base_color[1] as f32, base_color[2] as f32, intensity as f32]
    }

    pub fn get_sun_moon_position(&self) -> (f32, f32, f32, f32) {
        let time = self.current_time;
        let (angle, is_sun) = if time >= 0.0 && time < 12000.0 {
            // Sun rises at 0 (6am), sets at 12000 (6pm)
            let sun_angle = (time / 12000.0) * std::f64::consts::PI; // 0 to PI
            (sun_angle, true)
        } else {
            // Moon rises at 12000 (6pm), sets at 24000 (6am)
            let moon_angle = ((time - 12000.0) / 12000.0) * std::f64::consts::PI; // 0 to PI
            (moon_angle, false)
        };

        let x = (angle.cos() * 100.0) as f32;
        let y = (angle.sin() * 100.0) as f32;
        let z = 0.0;
        let brightness = if is_sun { 1.0 } else { 0.3 };

        (x, y, z, brightness)
    }

    pub fn get_fog_color(&self) -> [f32; 4] {
        let sky_color = self.get_sky_color();
        // Fog is slightly darker than sky
        [sky_color[0] * 0.8, sky_color[1] * 0.8, sky_color[2] * 0.8, sky_color[3]]
    }

    pub fn get_ambient_light(&self) -> f32 {
        match self.get_time_of_day() {
            TimeOfDay::Dawn => 0.3,
            TimeOfDay::Morning => 0.8,
            TimeOfDay::Noon => 1.0,
            TimeOfDay::Afternoon => 0.9,
            TimeOfDay::Dusk => 0.4,
            TimeOfDay::Night => 0.1,
            TimeOfDay::Midnight => 0.05,
        }
    }

    pub fn set_time(&mut self, time: f64) {
        self.current_time = time % 24000.0;
    }

    pub fn set_time_of_day(&mut self, time_of_day: TimeOfDay) {
        self.current_time = match time_of_day {
            TimeOfDay::Dawn => 3000.0,
            TimeOfDay::Morning => 6000.0,
            TimeOfDay::Noon => 6000.0,
            TimeOfDay::Afternoon => 12000.0,
            TimeOfDay::Dusk => 17000.0,
            TimeOfDay::Night => 18000.0,
            TimeOfDay::Midnight => 0.0,
        };
    }

    pub fn get_formatted_time(&self) -> String {
        let hours = (self.current_time / 1000.0) as u32;
        let minutes = ((self.current_time % 1000.0) * 60.0 / 1000.0) as u32;
        format!("{:02}:{:02}", hours % 24, minutes)
    }

    pub fn is_daytime(&self) -> bool {
        matches!(self.get_time_of_day(), TimeOfDay::Morning | TimeOfDay::Noon | TimeOfDay::Afternoon)
    }

    pub fn is_nighttime(&self) -> bool {
        matches!(self.get_time_of_day(), TimeOfDay::Night | TimeOfDay::Midnight)
    }

    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    pub fn resume(&mut self) {
        self.is_paused = false;
        self.last_update = Instant::now();
    }

    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.clamp(0.1, 100.0);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherEvent {
    pub event_type: WeatherEventType,
    pub start_time: f64,
    pub duration: f64,
    pub intensity: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WeatherEventType {
    Rain,
    Thunderstorm,
    Snow,
    Clear,
}

impl TimeSystem {
    pub fn should_spawn_mobs(&self) -> bool {
        self.is_nighttime()
    }

    pub fn get_mob_spawn_rate_modifier(&self) -> f32 {
        match self.get_time_of_day() {
            TimeOfDay::Night => 2.0, // Double spawn rate at night
            TimeOfDay::Dawn | TimeOfDay::Dusk => 1.5,
            _ => 1.0,
        }
    }
}
use crate::engine::{BlockPos, BlockType, CHUNK_SIZE_X, CHUNK_SIZE_Y, CHUNK_SIZE_Z, WORLD_HEIGHT};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WaterType {
    River,
    Ocean,
    Lake,
}

#[derive(Debug, Clone)]
pub struct RiverData {
    pub path: Vec<Vec3>,
    pub width: f32,
    pub depth: f32,
    pub flow_direction: Vec3,
    pub water_type: WaterType,
}

#[derive(Debug, Clone)]
pub struct OceanData {
    pub seed: u32,
    pub islands: Vec<Vec3>,
    pub ocean_level: f32,
    pub beach_width: f32,
}

pub struct WaterSystem {
    pub rivers: HashMap<(i32, i32, i32), RiverData>,
    pub oceans: OceanData,
    pub water_level: f32,
}

impl WaterSystem {
    pub fn new(seed: u32) -> Self {
        Self {
            rivers: HashMap::new(),
            oceans: OceanData {
                seed,
                islands: Vec::new(),
                ocean_level: 60.0,
                beach_width: 8.0,
            },
            water_level: 60.0,
        }
    }

    pub fn generate_rivers(&mut self, world: &mut World, noise_gen: &NoiseGenerator) {
        // Generate river paths using noise
        let river_count = 3 + (noise_gen.get_noise_octaves(0.0, 0.0, 0.0, 4) * 2.0) as i32;
        
        for i in 0..river_count {
            let river = self.generate_single_river(noise_gen, i);
            
            // Apply river to world chunks
            for &pos in &river.path {
                let chunk_pos = (
                    (pos.x as i32 / 16),
                    (pos.y as i32 / 16),
                    (pos.z as i32 / 16)
                );
                
                if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
                    let lx = pos.x.rem_euclid(16.0) as usize;
                    let ly = pos.y.rem_euclid(16.0) as usize;
                    let lz = pos.z.rem_euclid(16.0) as usize;
                    
                    // Create river bed and water
                    self.create_river_at_position(chunk, lx, ly, lz, &river);
                }
            }
            
            self.rivers.insert((i, 0, 0), river);
        }
    }

    fn generate_single_river(&self, noise_gen: &NoiseGenerator, seed: i32) -> RiverData {
        let mut path = Vec::new();
        let mut current_pos = Vec3::new(
            (seed as f32 * 1000.0).sin() * 200.0,
            60.0,
            (seed as f32 * 1000.0).cos() * 200.0
        );
        
        // Generate river path using noise
        for step in 0..500 {
            path.push(current_pos);
            
            // Use noise to determine river direction
            let angle = noise_gen.get_noise_octaves(
                current_pos.x as f64 * 0.01,
                current_pos.z as f64 * 0.01,
                step as f64 * 0.1,
                4
            ) * std::f64::consts::PI * 2.0;
            
            let next_direction = Vec3::new(
                angle.cos() as f32,
                0.0,
                angle.sin() as f32
            ).normalize();
            
            current_pos = current_pos + next_direction * 2.0;
            
            // Keep river at water level
            current_pos.y = 60.0;
            
            // Stop river if it goes too far from origin
            if current_pos.length() > 300.0 {
                break;
            }
        }
        
        RiverData {
            path,
            width: 4.0 + (noise_gen.get_noise_octaves(0.0, 0.0, 0.0, 2) * 2.0) as f32,
            depth: 3.0,
            flow_direction: Vec3::new(1.0, 0.0, 0.0),
            water_type: WaterType::River,
        }
    }

    fn create_river_at_position(&self, chunk: &mut Chunk, lx: usize, ly: usize, lz: usize, river: &RiverData) {
        let river_width = river.width as i32;
        let _river_depth = river.depth as i32;
        
        // Create river bed (sand/gravel)
        for dx in -river_width..=river_width {
            for dz in -river_width..=river_width {
                let distance = (dx * dx + dz * dz) as f32;
                if distance <= (river_width * river_width) as f32 {
                    let tlx = lx as i32 + dx;
                    let tlz = lz as i32 + dz;
                    
                    if tlx >= 0 && tlx < 16 && tlz >= 0 && tlz < 16 {
                        let current = chunk.get_block(tlx as usize, ly as usize, tlz as usize);
                        if current == BlockType::Stone || current == BlockType::Dirt || current == BlockType::Grass {
                            // Replace with sand or gravel for river bed
                            let river_bed = if (tlx + tlz) % 3 == 0 {
                                BlockType::Gravel
                            } else {
                                BlockType::Sand
                            };
                            chunk.set_block(tlx as usize, ly as usize, tlz as usize, river_bed);
                        }
                    }
                }
            }
        }
        
        // Create water
        for dx in -river_width..=river_width {
            for dz in -river_width..=river_width {
                let distance = (dx * dx + dz * dz) as f32;
                if distance <= (river_width * river_width) as f32 {
                    let tlx = lx as i32 + dx;
                    let tlz = lz as i32 + dz;
                    
                    if tlx >= 0 && tlx < 16 && tlz >= 0 && tlz < 16 {
                        chunk.set_block(tlx as usize, ly as usize, tlz as usize, BlockType::Water);
                        chunk.is_empty = false;
                    }
                }
            }
        }
    }

    pub fn generate_oceans(&mut self, world: &mut World, noise_gen: &NoiseGenerator) {
        // Generate ocean islands using noise
        let island_count = 5 + (noise_gen.get_noise_octaves(0.0, 0.0, 0.0, 3) * 3.0) as i32;
        
        for i in 0..island_count {
            let island_pos = Vec3::new(
                (i as f32 * 200.0).sin() * 300.0,
                60.0,
                (i as f32 * 200.0).cos() * 300.0
            );
            
            self.oceans.islands.push(island_pos);
            
            // Generate island terrain
            self.generate_island(world, noise_gen, island_pos);
        }
    }

    fn generate_island(&self, world: &mut World, noise_gen: &NoiseGenerator, center: Vec3) {
        let island_radius = 80.0;
        let beach_width = self.oceans.beach_width;
        
        // Generate chunks around island center
        let cx_min = ((center.x - island_radius) / 16.0) as i32;
        let cx_max = ((center.x + island_radius) / 16.0) as i32;
        let cz_min = ((center.z - island_radius) / 16.0) as i32;
        let cz_max = ((center.z + island_radius) / 16.0) as i32;
        
        for cx in cx_min..=cx_max {
            for cz in cz_min..=cz_max {
                for cy in 0..(WORLD_HEIGHT / 16) {
                    let chunk_pos = (cx, cy, cz);
                    if !world.chunks.contains_key(&chunk_pos) {
                        self.generate_island_chunk(world, noise_gen, chunk_pos, center, island_radius, beach_width);
                    }
                }
            }
        }
    }

    fn generate_island_chunk(&self, world: &mut World, noise_gen: &NoiseGenerator, chunk_pos: (i32, i32, i32), center: Vec3, island_radius: f32, beach_width: f32) {
        let mut chunk = Chunk::new();
        let chunk_x_world = chunk_pos.0 * 16;
        let chunk_y_world = chunk_pos.1 * 16;
        let chunk_z_world = chunk_pos.2 * 16;
        
        let _distance_from_center = ((chunk_x_world as f32 - center.x).powi(2) + (chunk_z_world as f32 - center.z).powi(2)).sqrt();
        
        for lx in 0..CHUNK_SIZE_X {
            for lz in 0..CHUNK_SIZE_Z {
                let wx = chunk_x_world + lx as i32;
                let wz = chunk_z_world + lz as i32;
                
                // Calculate distance from island center
                let distance = ((wx as f32 - center.x).powi(2) + (wz as f32 - center.z).powi(2)).sqrt();
                
                for ly in 0..CHUNK_SIZE_Y {
                    let y_world = chunk_y_world + ly as i32;
                    let mut block = BlockType::Water; // Default to water
                    
                    if distance < island_radius {
                        // Generate island terrain
                        let height = self.get_island_height(noise_gen, wx, y_world, wz, center, island_radius, beach_width);
                        
                        if height > self.water_level {
                            // Above water level - generate terrain
                            let density = noise_gen.get_density(wx, y_world, wz, 0.5, 0.5, 0.5);
                            
                            if density > 0.0 {
                                if distance < island_radius - beach_width {
                                    // Main island - grass/dirt/stone
                                    if y_world < 70 {
                                        block = BlockType::Grass;
                                    } else if y_world < 80 {
                                        block = BlockType::Dirt;
                                    } else {
                                        block = BlockType::Stone;
                                    }
                                } else {
                                    // Beach area - sand
                                    block = BlockType::Sand;
                                }
                            }
                        }
                    }
                    
                    // Set block
                    chunk.set_block(lx, ly, lz, block);
                    if block != BlockType::Air {
                        chunk.is_empty = false;
                    }
                }
            }
        }
        
        world.chunks.insert(chunk_pos, chunk);
    }

    fn get_island_height(&self, noise_gen: &NoiseGenerator, x: i32, y: i32, z: i32, center: Vec3, island_radius: f32, _beach_width: f32) -> f32 {
        let distance = ((x as f32 - center.x).powi(2) + (z as f32 - center.z).powi(2)).sqrt();
        
        if distance > island_radius {
            return self.water_level; // Ocean level
        }
        
        let normalized_distance = distance / island_radius;
        let height_factor = (1.0 - normalized_distance).max(0.0);
        
        // Generate height using noise
        let base_height = 70.0 + height_factor * 30.0;
        let noise = noise_gen.get_noise_octaves(x as f64 * 0.02, y as f64 * 0.02, z as f64 * 0.02, 4) as f32;
        
        base_height + noise * 10.0
    }

    pub fn update_water_flow(&mut self, world: &mut World) {
        // Update river flow directions based on terrain
        for (_, river) in self.rivers.iter_mut() {
            for i in 1..river.path.len() {
                let from = river.path[i - 1];
                let to = river.path[i];
                let flow_direction = (to - from).normalize();
                
                // Update flow direction for this segment
                river.flow_direction = flow_direction;
                
                // Apply flow to water blocks
                let chunk_pos = (
                    (to.x as i32 / 16),
                    (to.y as i32 / 16),
                    (to.z as i32 / 16)
                );
                
                if let Some(chunk) = world.chunks.get_mut(&chunk_pos) {
                    let lx = to.x.rem_euclid(16.0) as usize;
                    let ly = to.y.rem_euclid(16.0) as usize;
                    let lz = to.z.rem_euclid(16.0) as usize;
                    
                    if chunk.get_block(lx, ly, lz) == BlockType::Water {
                        // Update water flow direction (would be used for rendering)
                        // This could be stored in a separate water flow map
                    }
                }
            }
        }
    }

    pub fn is_water_at(&self, pos: BlockPos) -> bool {
        // Check if position is in water (river or ocean)
        let _chunk_pos = (
            pos.x / 16,
            pos.y / 16,
            pos.z / 16
        );
        
        // Check if in river
        for (_, river) in &self.rivers {
            for river_pos in &river.path {
                let distance = (*river_pos - Vec3::new(pos.x as f32, pos.y as f32, pos.z as f32)).length();
                if distance < river.width {
                    return true;
                }
            }
        }
        
        // Check if in ocean (below water level and not in land)
        if pos.y < self.water_level as i32 {
            // Would need to check if this position is land or water
            // For now, assume below water level is ocean
            return true;
        }
        
        false
    }

    pub fn get_water_level(&self) -> f32 {
        self.water_level
    }

    pub fn set_water_level(&mut self, level: f32) {
        self.water_level = level;
        self.oceans.ocean_level = level;
    }
}
use serde::{Serialize, Deserialize};
use crate::engine::World;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobType {
    Zombie,
    Skeleton,
    Creeper,
    Spider,
    Cow,
    Pig,
    Sheep,
    Chicken,
    Enderman,
    Witch,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MobState {
    Idle,
    Wandering,
    Chasing,
    Attacking,
    Fleeing,
    Dead,
}

#[derive(Debug, Clone)]
pub struct MobDrop {
    pub item_type: BlockType,
    pub min_count: u8,
    pub max_count: u8,
    pub chance: f32, // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub struct Mob {
    pub mob_type: MobType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Vec3,
    pub health: f32,
    pub max_health: f32,
    pub state: MobState,
    pub target: Option<BlockPos>,
    pub attack_cooldown: f32,
    pub wander_timer: f32,
    pub age: f32,
    pub drops: Vec<MobDrop>,
    pub experience_value: u32,
}

impl Mob {
    pub fn new(mob_type: MobType, position: Vec3) -> Self {
        let (health, max_health, drops, exp) = match mob_type {
            MobType::Zombie => (20.0, 20.0, vec![
                MobDrop { item_type: BlockType::RottenFlesh, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Carrot, min_count: 0, max_count: 1, chance: 0.1 },
                MobDrop { item_type: BlockType::Potato, min_count: 0, max_count: 1, chance: 0.1 },
            ], 5),
            MobType::Skeleton => (20.0, 20.0, vec![
                MobDrop { item_type: BlockType::Arrow, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Bone, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Bow, min_count: 0, max_count: 1, chance: 0.1 },
            ], 5),
            MobType::Creeper => (20.0, 20.0, vec![
                MobDrop { item_type: BlockType::Gunpowder, min_count: 0, max_count: 2, chance: 1.0 },
            ], 5),
            MobType::Spider => (16.0, 16.0, vec![
                MobDrop { item_type: BlockType::String, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::SpiderEye, min_count: 0, max_count: 1, chance: 0.3 },
            ], 5),
            MobType::Cow => (10.0, 10.0, vec![
                MobDrop { item_type: BlockType::Leather, min_count: 0, max_count: 3, chance: 1.0 },
                MobDrop { item_type: BlockType::Beef, min_count: 1, max_count: 3, chance: 1.0 },
            ], 3),
            MobType::Pig => (10.0, 10.0, vec![
                MobDrop { item_type: BlockType::Porkchop, min_count: 1, max_count: 3, chance: 1.0 },
            ], 3),
            MobType::Sheep => (8.0, 8.0, vec![
                MobDrop { item_type: BlockType::Wool, min_count: 1, max_count: 1, chance: 1.0 },
                MobDrop { item_type: BlockType::Mutton, min_count: 1, max_count: 2, chance: 1.0 },
            ], 3),
            MobType::Chicken => (4.0, 4.0, vec![
                MobDrop { item_type: BlockType::Feather, min_count: 0, max_count: 2, chance: 1.0 },
                MobDrop { item_type: BlockType::Chicken, min_count: 0, max_count: 2, chance: 1.0 },
            ], 3),
            MobType::Enderman => (40.0, 40.0, vec![
                MobDrop { item_type: BlockType::EnderPearl, min_count: 0, max_count: 1, chance: 0.5 },
            ], 5),
            MobType::Witch => (26.0, 26.0, vec![
                MobDrop { item_type: BlockType::GlowstoneDust, min_count: 0, max_count: 2, chance: 0.125 },
                MobDrop { item_type: BlockType::Redstone, min_count: 0, max_count: 2, chance: 0.125 },
                MobDrop { item_type: BlockType::SpiderEye, min_count: 0, max_count: 1, chance: 0.125 },
                MobDrop { item_type: BlockType::Gunpowder, min_count: 0, max_count: 2, chance: 0.125 },
            ], 5),
        };

        Self {
            mob_type,
            position,
            velocity: Vec3::ZERO,
            rotation: Vec3::ZERO,
            health,
            max_health,
            state: MobState::Idle,
            target: None,
            attack_cooldown: 0.0,
            wander_timer: 0.0,
            age: 0.0,
            drops,
            experience_value: exp,
        }
    }

    pub fn take_damage(&mut self, amount: f32) -> bool {
        self.health -= amount;
        if self.health <= 0.0 {
            self.health = 0.0;
            self.state = MobState::Dead;
            true
        } else {
            false
        }
    }

    pub fn get_drops(&self) -> Vec<(BlockType, u8)> {
        let mut drops = Vec::new();
        
        for drop in &self.drops {
            if rand::random::<f32>() <= drop.chance {
                let count = if drop.min_count == drop.max_count {
                    drop.min_count
                } else {
                    use rand::Rng;
                    rand::thread_rng().gen_range(drop.min_count..=drop.max_count)
                };
                drops.push((drop.item_type, count));
            }
        }
        
        drops
    }

    pub fn update(&mut self, world: &World, player_pos: Vec3, dt: f32) {
        self.age += dt;
        
        // Update attack cooldown
        if self.attack_cooldown > 0.0 {
            self.attack_cooldown -= dt;
        }

        match self.state {
            MobState::Dead => return,
            MobState::Idle => {
                self.wander_timer -= dt;
                if self.wander_timer <= 0.0 {
                    self.state = MobState::Wandering;
                    let mut rng = crate::engine::SimpleRng::new((self.position.x * 1000.0) as u64);
                    self.wander_timer = rng.gen_range(2.0, 5.0);
                }
                
                // Check for player proximity
                let distance_to_player = (player_pos - self.position).length();
                let detection_range = match self.mob_type {
                    MobType::Zombie | MobType::Skeleton | MobType::Creeper => 16.0,
                    MobType::Spider => 12.0,
                    MobType::Enderman => 64.0,
                    MobType::Witch => 16.0,
                    _ => 8.0,
                };
                
                if distance_to_player < detection_range {
                    self.state = MobState::Chasing;
                    self.target = Some(BlockPos {
                        x: player_pos.x as i32,
                        y: player_pos.y as i32,
                        z: player_pos.z as i32,
                    });
                }
            }
            MobState::Wandering => {
                self.wander_timer -= dt;
                
                // Random movement
                if self.wander_timer <= 0.0 {
                    let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
                    self.velocity.x = angle.cos() * 0.5;
                    self.velocity.z = angle.sin() * 0.5;
                    let mut rng = crate::engine::SimpleRng::new((self.position.z * 1000.0) as u64);
                    self.wander_timer = rng.gen_range(1.0, 3.0);
                }
                
                // Check for player proximity
                let distance_to_player = (player_pos - self.position).length();
                let detection_range = match self.mob_type {
                    MobType::Zombie | MobType::Skeleton | MobType::Creeper => 16.0,
                    MobType::Spider => 12.0,
                    MobType::Enderman => 64.0,
                    MobType::Witch => 16.0,
                    _ => 8.0,
                };
                
                if distance_to_player < detection_range {
                    self.state = MobState::Chasing;
                    self.target = Some(BlockPos {
                        x: player_pos.x as i32,
                        y: player_pos.y as i32,
                        z: player_pos.z as i32,
                    });
                }
            }
            MobState::Chasing => {
                if let Some(target) = self.target {
                    let target_pos = Vec3::new(
                        target.x as f32 + 0.5,
                        target.y as f32,
                        target.z as f32 + 0.5,
                    );
                    
                    let direction = (target_pos - self.position).normalize();
                    let speed = match self.mob_type {
                        MobType::Zombie | MobType::Skeleton => 1.0,
                        MobType::Spider => 1.2,
                        MobType::Creeper => 0.9,
                        MobType::Enderman => 3.0,
                        MobType::Witch => 0.8,
                        MobType::Cow | MobType::Pig => 0.8,
                        MobType::Sheep => 0.8,
                        MobType::Chicken => 0.6,
                    };
                    
                    self.velocity = direction * speed;
                    
                    // Check if close enough to attack
                    let distance = (target_pos - self.position).length();
                    if distance < 2.0 && self.attack_cooldown <= 0.0 {
                        self.state = MobState::Attacking;
                        self.attack_cooldown = match self.mob_type {
                            MobType::Zombie => 1.0,
                            MobType::Skeleton => 1.5,
                            MobType::Spider => 1.0,
                            MobType::Creeper => 1.5,
                            _ => 1.0,
                        };
                    }
                } else {
                    self.state = MobState::Idle;
                }
            }
            MobState::Attacking => {
                // Attack animation and damage would be handled here
                self.state = MobState::Chasing;
            }
            MobState::Fleeing => {
                // Fleeing behavior (for passive mobs)
                let direction = (self.position - player_pos).normalize();
                let speed = 1.5;
                self.velocity = direction * speed;
                
                let distance = (player_pos - self.position).length();
                if distance > 16.0 {
                    self.state = MobState::Idle;
                }
            }
        }
        
        // Apply physics
        self.apply_physics(world, dt);
    }

    fn apply_physics(&mut self, world: &World, dt: f32) {
        // Gravity
        self.velocity.y -= 20.0 * dt;
        
        // Simple collision detection
        let next_pos = self.position + self.velocity * dt;
        
        // Ground collision
        let ground_check = BlockPos {
            x: next_pos.x.floor() as i32,
            y: (next_pos.y - 0.5).floor() as i32,
            z: next_pos.z.floor() as i32,
        };
        
        if world.get_block(ground_check).is_solid() {
            self.position.y = ground_check.y as f32 + 1.0;
            self.velocity.y = 0.0;
        } else {
            self.position.y = next_pos.y;
        }
        
        // Horizontal movement with simple collision
        let horizontal_check = BlockPos {
            x: next_pos.x.floor() as i32,
            y: (self.position.y).floor() as i32,
            z: next_pos.z.floor() as i32,
        };
        
        if !world.get_block(horizontal_check).is_solid() {
            self.position.x = next_pos.x;
            self.position.z = next_pos.z;
        }
        
        // Update rotation to face movement direction
        if self.velocity.length_squared() > 0.01 {
            self.rotation.y = self.velocity.z.atan2(self.velocity.x);
        }
    }

    pub fn get_size(&self) -> f32 {
        match self.mob_type {
            MobType::Zombie | MobType::Skeleton => 0.6,
            MobType::Creeper => 0.7,
            MobType::Spider => 0.5,
            MobType::Cow => 0.9,
            MobType::Pig => 0.8,
            MobType::Sheep => 0.8,
            MobType::Chicken => 0.4,
            MobType::Enderman => 0.6,
            MobType::Witch => 0.6,
        }
    }

    pub fn get_height(&self) -> f32 {
        match self.mob_type {
            MobType::Zombie | MobType::Skeleton => 1.8,
            MobType::Creeper => 1.7,
            MobType::Spider => 0.9,
            MobType::Cow => 1.4,
            MobType::Pig => 0.9,
            MobType::Sheep => 1.3,
            MobType::Chicken => 0.7,
            MobType::Enderman => 2.6,
            MobType::Witch => 1.8,
        }
    }
}

pub struct MobSpawner {
    pub spawn_radius: f32,
    pub max_mobs_per_chunk: usize,
    pub spawn_rate: f32, // mobs per second
    pub last_spawn: f32,
    pub mob_biomes: HashMap<MobType, Vec<String>>,
    pub walk_time: f32, // Added for SimpleRng
}

impl MobSpawner {
    pub fn new() -> Self {
        let mut mob_biomes = HashMap::new();
        
        // Define which biomes each mob can spawn in
        mob_biomes.insert(MobType::Zombie, vec!["plains".to_string(), "forest".to_string(), "desert".to_string()]);
        mob_biomes.insert(MobType::Skeleton, vec!["plains".to_string(), "forest".to_string(), "desert".to_string()]);
        mob_biomes.insert(MobType::Creeper, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Spider, vec!["plains".to_string(), "forest".to_string(), "desert".to_string()]);
        mob_biomes.insert(MobType::Cow, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Pig, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Sheep, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Chicken, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Enderman, vec!["plains".to_string(), "forest".to_string()]);
        mob_biomes.insert(MobType::Witch, vec!["swamp".to_string(), "forest".to_string()]);
        
        Self {
            spawn_radius: 32.0,
            max_mobs_per_chunk: 8,
            spawn_rate: 0.1,
            last_spawn: 0.0,
            mob_biomes,
            walk_time: 0.0,
        }
    }

    pub fn try_spawn(&mut self, world: &World, player_pos: Vec3, current_time: f32) -> Option<Mob> {
        if current_time - self.last_spawn < 1.0 / self.spawn_rate {
            return None;
        }

        // Find a suitable spawn position
        let spawn_attempts = 10;
        for _ in 0..spawn_attempts {
            let angle = rand::random::<f32>() * 2.0 * std::f32::consts::PI;
            let mut rng = crate::engine::SimpleRng::new(self.walk_time as u64 + 1);
            
            let distance = rng.gen_range(8.0, self.spawn_radius);
            
            let spawn_x = player_pos.x + angle.cos() * distance;
            let spawn_z = player_pos.z + angle.sin() * distance;
            
            // Get ground height at this position
            let ground_y = world.get_height_at(spawn_x as i32, spawn_z as i32) as f32;
            let spawn_pos = Vec3::new(spawn_x, ground_y + 1.0, spawn_z);
            
            // Check if position is valid for spawning
            if self.is_valid_spawn_position(world, spawn_pos) {
                // Get biome at this position
                let biome = self.get_biome_at(world, spawn_x as i32, spawn_z as i32);
                
                // Choose a random mob that can spawn in this biome
                let available_mobs: Vec<MobType> = self.mob_biomes
                    .iter()
                    .filter(|(_, biomes)| biomes.contains(&biome))
                    .map(|(mob_type, _)| *mob_type)
                    .collect();
                
                if !available_mobs.is_empty() {
                    let mut rng = crate::engine::SimpleRng::new(current_time as u64);
                    let mob_index = rng.gen_range(0.0, available_mobs.len() as f32) as usize;
                    let mob_type = available_mobs[mob_index];
                    self.last_spawn = current_time;
                    return Some(Mob::new(mob_type, spawn_pos));
                }
            }
        }
        
        None
    }

    fn is_valid_spawn_position(&self, world: &World, pos: Vec3) -> bool {
        // Check if ground is solid
        let ground_pos = BlockPos {
            x: pos.x.floor() as i32,
            y: (pos.y - 1.0).floor() as i32,
            z: pos.z.floor() as i32,
        };
        
        if !world.get_block(ground_pos).is_solid() {
            return false;
        }
        
        // Check if spawn position is not solid
        let spawn_pos = BlockPos {
            x: pos.x.floor() as i32,
            y: pos.y.floor() as i32,
            z: pos.z.floor() as i32,
        };
        
        if world.get_block(spawn_pos).is_solid() {
            return false;
        }
        
        // Check if above spawn position is not solid (for tall mobs)
        let above_pos = BlockPos {
            x: pos.x.floor() as i32,
            y: (pos.y + 1.0).floor() as i32,
            z: pos.z.floor() as i32,
        };
        
        if world.get_block(above_pos).is_solid() {
            return false;
        }
        
        // Check light level (most hostile mobs spawn in darkness)
        let light_level = world.get_light_world(spawn_pos);
        if light_level > 7 {
            // Allow passive mobs to spawn in higher light
            return false; // For now, only spawn in darkness
        }
        
        true
    }

    fn get_biome_at(&self, _world: &World, _x: i32, _z: i32) -> String {
        // This would use the world's biome generation system
        // For now, return a default biome
        "plains".to_string()
    }
}

