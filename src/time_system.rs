use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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
