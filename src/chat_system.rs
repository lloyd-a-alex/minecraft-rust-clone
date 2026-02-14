use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use crate::time_system::TimeSystem;
use crate::world::World;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub player_name: String,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub message_type: ChatMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageType {
    Player,
    System,
    Command,
    Death,
    Achievement,
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub affected_systems: Vec<String>,
}

pub struct ChatSystem {
    pub messages: Vec<ChatMessage>,
    pub max_messages: usize,
    pub commands: HashMap<String, Box<dyn CommandHandler>>,
    pub is_chat_open: bool,
    pub input_buffer: String,
    pub cursor_position: usize,
}

impl ChatSystem {
    pub fn new() -> Self {
        let mut chat = Self {
            messages: Vec::new(),
            max_messages: 100,
            commands: HashMap::new(),
            is_chat_open: false,
            input_buffer: String::new(),
            cursor_position: 0,
        };

        // Register built-in commands
        chat.register_command("help", HelpCommand::new());
        chat.register_command("time", TimeCommand::new());
        chat.register_command("weather", WeatherCommand::new());
        chat.register_command("clear", ClearCommand::new());
        chat.register_command("seed", SeedCommand::new());
        chat.register_command("tp", TeleportCommand::new());
        chat.register_command("gamemode", GamemodeCommand::new());
        chat.register_command("give", GiveCommand::new());
        chat.register_command("spawn", SpawnCommand::new());
        chat.register_command("kill", KillCommand::new());

        chat
    }

    pub fn add_message(&mut self, player_name: &str, message: &str, message_type: ChatMessageType) {
        let chat_message = ChatMessage {
            player_name: player_name.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now(),
            message_type,
        };

        self.messages.push(chat_message);

        // Remove old messages if we exceed the limit
        if self.messages.len() > self.max_messages {
            self.messages.remove(0);
        }
    }

    pub fn send_message(&mut self, player_name: &str, message: &str) -> Option<CommandResult> {
        if message.starts_with('/') {
            // This is a command
            let parts: Vec<&str> = message[1..].split_whitespace().collect();
            if parts.is_empty() {
                return Some(CommandResult {
                    success: false,
                    message: "No command specified. Use /help for available commands.".to_string(),
                    affected_systems: vec![],
                });
            }

            let command_name = parts[0].to_lowercase();
            let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

            if let Some(command_handler) = self.commands.get(&command_name) {
                let result = command_handler.execute(&args);
                
                // Log command execution
                self.add_message("System", &result.message, ChatMessageType::Command);
                
                Some(result)
            } else {
                Some(CommandResult {
                    success: false,
                    message: format!("Unknown command: {}. Use /help for available commands.", command_name),
                    affected_systems: vec![],
                })
            }
        } else {
            // This is a regular chat message
            self.add_message(player_name, message, ChatMessageType::Player);
            None
        }
    }

    pub fn register_command(&mut self, name: &str, handler: Box<dyn CommandHandler>) {
        self.commands.insert(name.to_lowercase(), handler);
    }

    pub fn toggle_chat(&mut self) {
        self.is_chat_open = !self.is_chat_open;
        if self.is_chat_open {
            self.input_buffer.clear();
            self.cursor_position = 0;
        }
    }

    pub fn handle_input(&mut self, input: &str) -> Option<CommandResult> {
        if input.is_empty() {
            return None;
        }

        let result = self.send_message("Player", input);
        self.input_buffer.clear();
        self.cursor_position = 0;
        result
    }

    pub fn get_recent_messages(&self, count: usize) -> Vec<&ChatMessage> {
        let start = if self.messages.len() > count {
            self.messages.len() - count
        } else {
            0
        };
        self.messages[start..].iter().collect()
    }

    pub fn clear_chat(&mut self) {
        self.messages.clear();
        self.add_message("System", "Chat cleared.", ChatMessageType::System);
    }
}

pub trait CommandHandler {
    fn execute(&self, args: &[String]) -> CommandResult;
    fn get_help(&self) -> String;
    fn get_usage(&self) -> String;
}

// Built-in Commands

pub struct HelpCommand;
impl HelpCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for HelpCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            CommandResult {
                success: true,
                message: "Available commands: /help, /time, /weather, /clear, /seed, /tp, /gamemode, /give, /spawn, /kill. Use /help <command> for specific help.".to_string(),
                affected_systems: vec!["chat".to_string()],
            }
        } else {
            let help_text = match args[0].as_str() {
                "time" => "/time set <day|night|dawn|dusk|noon|midnight> - Sets the time of day",
                "weather" => "/weather <clear|rain|thunder|snow> - Changes the weather",
                "clear" => "/clear - Clears the chat history",
                "seed" => "/seed - Shows the world seed",
                "tp" => "/tp <x> <y> <z> - Teleports to coordinates",
                "gamemode" => "/gamemode <0|1|2> - Sets game mode (0=survival, 1=creative, 2=adventure)",
                "give" => "/give <item> [count] - Gives items to player",
                "spawn" => "/spawn <mob> [count] - Spawns mobs",
                "kill" => "/kill [target] - Kills entities (player or all mobs)",
                _ => "Unknown command. Use /help for available commands.",
            };
            
            CommandResult {
                success: true,
                message: help_text.to_string(),
                affected_systems: vec!["chat".to_string()],
            }
        }
    }

    fn get_help(&self) -> String {
        "Shows help for commands".to_string()
    }

    fn get_usage(&self) -> String {
        "/help [command]".to_string()
    }
}

pub struct TimeCommand;
impl TimeCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for TimeCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.len() < 2 || args[0] != "set" {
            return CommandResult {
                success: false,
                message: "Usage: /time set <day|night|dawn|dusk|noon|midnight>".to_string(),
                affected_systems: vec![],
            };
        }

        let time_of_day = match args[1].as_str() {
            "day" => "morning",
            "night" => "night",
            "dawn" => "dawn",
            "dusk" => "dusk",
            "noon" => "noon",
            "midnight" => "midnight",
            _ => {
                return CommandResult {
                    success: false,
                    message: "Invalid time. Use: day, night, dawn, dusk, noon, or midnight".to_string(),
                    affected_systems: vec![],
                };
            }
        };

        CommandResult {
            success: true,
            message: format!("Time set to {}", time_of_day),
            affected_systems: vec!["time".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Controls the time of day".to_string()
    }

    fn get_usage(&self) -> String {
        "/time set <day|night|dawn|dusk|noon|midnight>".to_string()
    }
}

pub struct WeatherCommand;
impl WeatherCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for WeatherCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /weather <clear|rain|thunder|snow>".to_string(),
                affected_systems: vec![],
            };
        }

        let weather = args[0].as_str();
        match weather {
            "clear" | "rain" | "thunder" | "snow" => {
                CommandResult {
                    success: true,
                    message: format!("Weather set to {}", weather),
                    affected_systems: vec!["weather".to_string()],
                }
            }
            _ => CommandResult {
                success: false,
                message: "Invalid weather type. Use: clear, rain, thunder, or snow".to_string(),
                affected_systems: vec![],
            },
        }
    }

    fn get_help(&self) -> String {
        "Controls the weather".to_string()
    }

    fn get_usage(&self) -> String {
        "/weather <clear|rain|thunder|snow>".to_string()
    }
}

pub struct ClearCommand;
impl ClearCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for ClearCommand {
    fn execute(&self, _args: &[String]) -> CommandResult {
        CommandResult {
            success: true,
            message: "Chat cleared.".to_string(),
            affected_systems: vec!["chat".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Clears the chat history".to_string()
    }

    fn get_usage(&self) -> String {
        "/clear".to_string()
    }
}

pub struct SeedCommand;
impl SeedCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for SeedCommand {
    fn execute(&self, _args: &[String]) -> CommandResult {
        // This would need access to the world to get the actual seed
        CommandResult {
            success: true,
            message: "World seed: 12345".to_string(), // Placeholder
            affected_systems: vec![],
        }
    }

    fn get_help(&self) -> String {
        "Shows the world seed".to_string()
    }

    fn get_usage(&self) -> String {
        "/seed".to_string()
    }
}

pub struct TeleportCommand;
impl TeleportCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for TeleportCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.len() != 3 {
            return CommandResult {
                success: false,
                message: "Usage: /tp <x> <y> <z>".to_string(),
                affected_systems: vec![],
            };
        }

        let x: f32 = match args[0].parse() {
            Ok(val) => val,
            Err(_) => return CommandResult {
                success: false,
                message: "Invalid X coordinate".to_string(),
                affected_systems: vec![],
            },
        };

        let y: f32 = match args[1].parse() {
            Ok(val) => val,
            Err(_) => return CommandResult {
                success: false,
                message: "Invalid Y coordinate".to_string(),
                affected_systems: vec![],
            },
        };

        let z: f32 = match args[2].parse() {
            Ok(val) => val,
            Err(_) => return CommandResult {
                success: false,
                message: "Invalid Z coordinate".to_string(),
                affected_systems: vec![],
            },
        };

        CommandResult {
            success: true,
            message: format!("Teleported to ({}, {}, {})", x, y, z),
            affected_systems: vec!["player".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Teleports to coordinates".to_string()
    }

    fn get_usage(&self) -> String {
        "/tp <x> <y> <z>".to_string()
    }
}

pub struct GamemodeCommand;
impl GamemodeCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for GamemodeCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /gamemode <0|1|2>".to_string(),
                affected_systems: vec![],
            };
        }

        let mode = match args[0].as_str() {
            "0" | "survival" => "Survival",
            "1" | "creative" => "Creative",
            "2" | "adventure" => "Adventure",
            _ => {
                return CommandResult {
                    success: false,
                    message: "Invalid game mode. Use: 0 (survival), 1 (creative), or 2 (adventure)".to_string(),
                    affected_systems: vec![],
                };
            }
        };

        CommandResult {
            success: true,
            message: format!("Game mode set to {}", mode),
            affected_systems: vec!["player".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Changes the game mode".to_string()
    }

    fn get_usage(&self) -> String {
        "/gamemode <0|1|2>".to_string()
    }
}

pub struct GiveCommand;
impl GiveCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for GiveCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /give <item> [count]".to_string(),
                affected_systems: vec![],
            };
        }

        let item = &args[0];
        let count = if args.len() > 1 {
            match args[1].parse::<u32>() {
                Ok(val) => val,
                Err(_) => return CommandResult {
                    success: false,
                    message: "Invalid count".to_string(),
                    affected_systems: vec![],
                },
            }
        } else {
            1
        };

        CommandResult {
            success: true,
            message: format!("Gave {} x{} to player", item, count),
            affected_systems: vec!["player".to_string(), "inventory".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Gives items to player".to_string()
    }

    fn get_usage(&self) -> String {
        "/give <item> [count]".to_string()
    }
}

pub struct SpawnCommand;
impl SpawnCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for SpawnCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        if args.is_empty() {
            return CommandResult {
                success: false,
                message: "Usage: /spawn <mob> [count]".to_string(),
                affected_systems: vec![],
            };
        }

        let mob = &args[0];
        let count = if args.len() > 1 {
            match args[1].parse::<u32>() {
                Ok(val) => val,
                Err(_) => return CommandResult {
                    success: false,
                    message: "Invalid count".to_string(),
                    affected_systems: vec![],
                },
            }
        } else {
            1
        };

        CommandResult {
            success: true,
            message: format!("Spawned {} x{}", mob, count),
            affected_systems: vec!["world".to_string(), "mobs".to_string()],
        }
    }

    fn get_help(&self) -> String {
        "Spawns mobs".to_string()
    }

    fn get_usage(&self) -> String {
        "/spawn <mob> [count]".to_string()
    }
}

pub struct KillCommand;
impl KillCommand {
    pub fn new() -> Box<dyn CommandHandler> {
        Box::new(Self)
    }
}

impl CommandHandler for KillCommand {
    fn execute(&self, args: &[String]) -> CommandResult {
        let target = if args.is_empty() {
            "player"
        } else {
            &args[0]
        };

        match target {
            "player" => CommandResult {
                success: true,
                message: "Player killed".to_string(),
                affected_systems: vec!["player".to_string()],
            },
            "all" => CommandResult {
                success: true,
                message: "All mobs killed".to_string(),
                affected_systems: vec!["world".to_string(), "mobs".to_string()],
            },
            _ => CommandResult {
                success: false,
                message: "Invalid target. Use: player or all".to_string(),
                affected_systems: vec![],
            },
        }
    }

    fn get_help(&self) -> String {
        "Kills entities".to_string()
    }

    fn get_usage(&self) -> String {
        "/kill [player|all]".to_string()
    }
}
