use std::collections::HashMap;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex};
use log::{Level, LevelFilter, Record};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: Instant,
    pub level: Level,
    pub target: String,
    pub message: String,
    pub module: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableLogEntry {
    pub timestamp_secs: u64,
    pub level: String,
    pub target: String,
    pub message: String,
    pub module: String,
}

impl From<&LogEntry> for SerializableLogEntry {
    fn from(entry: &LogEntry) -> Self {
        Self {
            timestamp_secs: entry.timestamp.elapsed().as_secs(),
            level: format!("{:?}", entry.level),
            target: entry.target.clone(),
            message: entry.message.clone(),
            module: entry.module.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogMetrics {
    pub total_logs: u64,
    pub error_count: u64,
    pub warn_count: u64,
    pub info_count: u64,
    pub debug_count: u64,
    pub trace_count: u64,
    pub last_error: Option<Instant>,
    pub last_warning: Option<Instant>,
}

pub struct AdvancedLogger {
    entries: Arc<Mutex<Vec<LogEntry>>>,
    metrics: Arc<Mutex<LogMetrics>>,
    module_filters: Arc<Mutex<HashMap<String, LevelFilter>>>,
    max_entries: usize,
    last_cleanup: Arc<Mutex<Instant>>,
    cleanup_interval: Duration,
}

impl AdvancedLogger {
    pub fn new(max_entries: usize, cleanup_interval: Duration) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            metrics: Arc::new(Mutex::new(LogMetrics {
                total_logs: 0,
                error_count: 0,
                warn_count: 0,
                info_count: 0,
                debug_count: 0,
                trace_count: 0,
                last_error: None,
                last_warning: None,
            })),
            module_filters: Arc::new(Mutex::new(HashMap::new())),
            max_entries,
            last_cleanup: Arc::new(Mutex::new(Instant::now())),
            cleanup_interval,
        }
    }
    
    pub fn log(&self, record: &Record) {
        let now = Instant::now();
        
        // Update metrics
        {
            let mut metrics = self.metrics.lock().unwrap();
            metrics.total_logs += 1;
            
            match record.level() {
                Level::Error => {
                    metrics.error_count += 1;
                    metrics.last_error = Some(now);
                }
                Level::Warn => {
                    metrics.warn_count += 1;
                    metrics.last_warning = Some(now);
                }
                Level::Info => metrics.info_count += 1,
                Level::Debug => metrics.debug_count += 1,
                Level::Trace => metrics.trace_count += 1,
            }
        }
        
        // Check module filter
        let should_log = {
            let filters = self.module_filters.lock().unwrap();
            if let Some(filter) = filters.get(record.target()) {
                record.level() <= *filter
            } else {
                true // Default to allowing all logs
            }
        };
        
        if !should_log {
            return;
        }
        
        // Add entry
        let entry = LogEntry {
            timestamp: now,
            level: record.level(),
            target: record.target().to_string(),
            message: record.args().to_string(),
            module: record.module_path().unwrap_or("unknown").to_string(),
        };
        
        {
            let mut entries = self.entries.lock().unwrap();
            entries.push(entry);
            
            // Cleanup old entries if needed
            if entries.len() > self.max_entries {
                entries.remove(0);
            }
        }
        
        // Periodic cleanup
        {
            let mut last_cleanup = self.last_cleanup.lock().unwrap();
            if now.duration_since(*last_cleanup) >= self.cleanup_interval {
                self.cleanup_old_entries();
                *last_cleanup = now;
            }
        }
    }
    
    pub fn set_module_filter(&self, module: &str, level: LevelFilter) {
        let mut filters = self.module_filters.lock().unwrap();
        filters.insert(module.to_string(), level);
    }
    
    pub fn get_recent_entries(&self, count: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        let start = if entries.len() > count { entries.len() - count } else { 0 };
        entries[start..].to_vec()
    }
    
    pub fn get_entries_by_level(&self, level: Level, count: usize) -> Vec<LogEntry> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .rev()
            .filter(|entry| entry.level == level)
            .take(count)
            .cloned()
            .collect()
    }
    
    pub fn get_metrics(&self) -> LogMetrics {
        self.metrics.lock().unwrap().clone()
    }
    
    pub fn get_error_summary(&self) -> Vec<String> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .rev()
            .filter(|entry| entry.level == Level::Error)
            .take(10)
            .map(|entry| {
                let age = entry.timestamp.elapsed();
                format!("[{:?}s ago] {}: {}", age.as_secs(), entry.target, entry.message)
            })
            .collect()
    }
    
    pub fn get_warning_summary(&self) -> Vec<String> {
        let entries = self.entries.lock().unwrap();
        entries
            .iter()
            .rev()
            .filter(|entry| entry.level == Level::Warn)
            .take(10)
            .map(|entry| {
                let age = entry.timestamp.elapsed();
                format!("[{:?}s ago] {}: {}", age.as_secs(), entry.target, entry.message)
            })
            .collect()
    }
    
    fn cleanup_old_entries(&self) {
        let mut entries = self.entries.lock().unwrap();
        let cutoff = Instant::now() - Duration::from_secs(300); // Keep last 5 minutes
        
        entries.retain(|entry| entry.timestamp > cutoff);
        
        // If still too many, keep only the most recent
        if entries.len() > self.max_entries {
            let excess = entries.len() - self.max_entries;
            entries.drain(0..excess);
        }
    }
    
    pub fn export_logs(&self, format: LogFormat) -> String {
        let entries = self.entries.lock().unwrap();
        match format {
            LogFormat::Json => {
                let serializable: Vec<SerializableLogEntry> = entries.iter().map(|e| e.into()).collect();
                serde_json::to_string_pretty(&serializable).unwrap_or_else(|_| "[]".to_string())
            }
            LogFormat::Text => {
                entries
                    .iter()
                    .map(|entry| {
                        let time = entry.timestamp.elapsed().as_secs_f32();
                        format!("[{:6.3}s] [{:5}] {:20} | {}", 
                               time, 
                               entry.level, 
                               entry.target, 
                               entry.message)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            }
        }
    }
    
    pub fn clear(&self) {
        let mut entries = self.entries.lock().unwrap();
        entries.clear();
        
        let mut metrics = self.metrics.lock().unwrap();
        *metrics = LogMetrics {
            total_logs: 0,
            error_count: 0,
            warn_count: 0,
            info_count: 0,
            debug_count: 0,
            trace_count: 0,
            last_error: None,
            last_warning: None,
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub enum LogFormat {
    Json,
    Text,
}

pub struct LogGuard {
    logger: Arc<AdvancedLogger>,
}

impl log::Log for LogGuard {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }
    
    fn log(&self, record: &Record) {
        self.logger.log(record);
    }
    
    fn flush(&self) {
        // Nothing to flush for in-memory logger
    }
}

pub fn init_advanced_logging() -> Arc<AdvancedLogger> {
    let logger = Arc::new(AdvancedLogger::new(10000, Duration::from_secs(60)));
    let guard = LogGuard { logger: logger.clone() };
    
    // Set as global logger
    log::set_boxed_logger(Box::new(guard)).expect("Failed to set logger");
    log::set_max_level(log::LevelFilter::Debug);
    
    logger
}

// Macro for enhanced logging with context
#[macro_export]
macro_rules! log_context {
    ($level:expr, $ctx:expr, $($arg:tt)*) => {
        log::log!(target: $ctx, $level, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_error {
    ($ctx:expr, $($arg:tt)*) => {
        log::log!(target: $ctx, log::Level::Error, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_warn {
    ($ctx:expr, $($arg:tt)*) => {
        log::log!(target: $ctx, log::Level::Warn, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_info {
    ($ctx:expr, $($arg:tt)*) => {
        log::log!(target: $ctx, log::Level::Info, $($arg)*);
    };
}

#[macro_export]
macro_rules! log_debug {
    ($ctx:expr, $($arg:tt)*) => {
        log::log!(target: $ctx, log::Level::Debug, $($arg)*);
    };
}
