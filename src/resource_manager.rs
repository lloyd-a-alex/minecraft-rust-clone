//! Resource management and cleanup utilities
//! 
//! This module provides centralized resource management to prevent memory exhaustion
//! and ensure proper cleanup of game resources.

use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Global resource limits configuration
pub struct ResourceLimits {
    pub max_chunks: usize,
    pub max_entities: usize,
    pub max_particles: usize,
    pub max_pending_tasks: usize,
    pub mesh_memory_limit_mb: usize,
    pub texture_memory_limit_mb: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_chunks: 2000,           // Maximum number of loaded chunks
            max_entities: 1000,          // Maximum number of entities
            max_particles: 5000,        // Maximum number of particles
            max_pending_tasks: 100,     // Maximum pending mesh tasks
            mesh_memory_limit_mb: 512,  // 512MB for mesh data
            texture_memory_limit_mb: 256, // 256MB for textures
        }
    }
}

/// Resource usage tracking
pub struct ResourceTracker {
    pub chunks_loaded: AtomicUsize,
    pub entities_active: AtomicUsize,
    pub particles_active: AtomicUsize,
    pub pending_tasks: AtomicUsize,
    pub mesh_memory_mb: AtomicUsize,
    pub texture_memory_mb: AtomicUsize,
    last_cleanup: Instant,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            chunks_loaded: AtomicUsize::new(0),
            entities_active: AtomicUsize::new(0),
            particles_active: AtomicUsize::new(0),
            pending_tasks: AtomicUsize::new(0),
            mesh_memory_mb: AtomicUsize::new(0),
            texture_memory_mb: AtomicUsize::new(0),
            last_cleanup: Instant::now(),
        }
    }

    pub fn check_limits(&self, limits: &ResourceLimits) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.chunks_loaded.load(Ordering::Relaxed) > limits.max_chunks {
            warnings.push(format!("Chunk limit exceeded: {}/{}", 
                self.chunks_loaded.load(Ordering::Relaxed), limits.max_chunks));
        }

        if self.entities_active.load(Ordering::Relaxed) > limits.max_entities {
            warnings.push(format!("Entity limit exceeded: {}/{}", 
                self.entities_active.load(Ordering::Relaxed), limits.max_entities));
        }

        if self.particles_active.load(Ordering::Relaxed) > limits.max_particles {
            warnings.push(format!("Particle limit exceeded: {}/{}", 
                self.particles_active.load(Ordering::Relaxed), limits.max_particles));
        }

        if self.pending_tasks.load(Ordering::Relaxed) > limits.max_pending_tasks {
            warnings.push(format!("Pending task limit exceeded: {}/{}", 
                self.pending_tasks.load(Ordering::Relaxed), limits.max_pending_tasks));
        }

        if self.mesh_memory_mb.load(Ordering::Relaxed) > limits.mesh_memory_limit_mb {
            warnings.push(format!("Mesh memory limit exceeded: {}MB/{}MB", 
                self.mesh_memory_mb.load(Ordering::Relaxed), limits.mesh_memory_limit_mb));
        }

        if self.texture_memory_mb.load(Ordering::Relaxed) > limits.texture_memory_limit_mb {
            warnings.push(format!("Texture memory limit exceeded: {}MB/{}MB", 
                self.texture_memory_mb.load(Ordering::Relaxed), limits.texture_memory_limit_mb));
        }

        warnings
    }

    pub fn should_cleanup(&self) -> bool {
        self.last_cleanup.elapsed() > Duration::from_secs(30)
    }

    pub fn mark_cleanup(&mut self) {
        self.last_cleanup = Instant::now();
    }
}

/// Cleanup strategies for different resource types
pub enum CleanupStrategy {
    /// Remove oldest resources
    OldestFirst,
    /// Remove farthest from player
    FarthestFromPlayer,
    /// Remove least recently used
    LeastRecentlyUsed,
    /// Random removal (for particles)
    Random,
}

/// Resource cleanup manager
pub struct ResourceCleanupManager {
    tracker: ResourceTracker,
    limits: ResourceLimits,
}

impl ResourceCleanupManager {
    pub fn new() -> Self {
        Self {
            tracker: ResourceTracker::new(),
            limits: ResourceLimits::default(),
        }
    }

    pub fn with_limits(limits: ResourceLimits) -> Self {
        Self {
            tracker: ResourceTracker::new(),
            limits,
        }
    }

    pub fn tracker(&self) -> &ResourceTracker {
        &self.tracker
    }

    pub fn limits(&self) -> &ResourceLimits {
        &self.limits
    }

    /// Perform cleanup if needed and return cleanup statistics
    pub fn cleanup_if_needed(&mut self) -> CleanupStats {
        if !self.tracker.should_cleanup() {
            return CleanupStats::default();
        }

        let warnings = self.tracker.check_limits(&self.limits);
        if warnings.is_empty() {
            self.tracker.mark_cleanup();
            return CleanupStats::default();
        }

        log::info!("Starting resource cleanup: {:?}", warnings);
        let stats = self.perform_cleanup();
        self.tracker.mark_cleanup();
        
        log::info!("Cleanup completed: {:?}", stats);
        stats
    }

    fn perform_cleanup(&mut self) -> CleanupStats {
        let mut stats = CleanupStats::default();

        // This would be called from the main game loop with actual resource references
        // For now, we'll just log what would be cleaned up
        
        if self.tracker.chunks_loaded.load(Ordering::Relaxed) > self.limits.max_chunks {
            let excess = self.tracker.chunks_loaded.load(Ordering::Relaxed) - self.limits.max_chunks;
            stats.chunks_cleaned = excess;
            log::debug!("Would clean up {} excess chunks", excess);
        }

        if self.tracker.entities_active.load(Ordering::Relaxed) > self.limits.max_entities {
            let excess = self.tracker.entities_active.load(Ordering::Relaxed) - self.limits.max_entities;
            stats.entities_cleaned = excess;
            log::debug!("Would clean up {} excess entities", excess);
        }

        if self.tracker.particles_active.load(Ordering::Relaxed) > self.limits.max_particles {
            let excess = self.tracker.particles_active.load(Ordering::Relaxed) - self.limits.max_particles;
            stats.particles_cleaned = excess;
            log::debug!("Would clean up {} excess particles", excess);
        }

        stats
    }
}

/// Cleanup statistics
#[derive(Debug, Default)]
pub struct CleanupStats {
    pub chunks_cleaned: usize,
    pub entities_cleaned: usize,
    pub particles_cleaned: usize,
    pub tasks_cancelled: usize,
    pub memory_freed_mb: usize,
}

impl CleanupStats {
    pub fn total_cleaned(&self) -> usize {
        self.chunks_cleaned + self.entities_cleaned + self.particles_cleaned + self.tasks_cancelled
    }

    pub fn has_cleaned_anything(&self) -> bool {
        self.total_cleaned() > 0 || self.memory_freed_mb > 0
    }
}

/// Global resource manager instance
static mut RESOURCE_MANAGER: Option<ResourceCleanupManager> = None;
static INIT: std::sync::Once = std::sync::Once::new();

/// Get the global resource manager
#[allow(static_mut_refs)]
pub fn get_resource_manager() -> &'static mut ResourceCleanupManager {
    unsafe {
        INIT.call_once(|| {
            RESOURCE_MANAGER = Some(ResourceCleanupManager::new());
        });
        RESOURCE_MANAGER.as_mut().unwrap()
    }
}

/// Initialize the resource manager with custom limits
pub fn init_resource_manager(limits: ResourceLimits) {
    unsafe {
        INIT.call_once(|| {
            RESOURCE_MANAGER = Some(ResourceCleanupManager::with_limits(limits));
        });
    }
}

/// Convenience functions for tracking resource usage
pub fn track_chunk_usage(count: usize) {
    get_resource_manager().tracker.chunks_loaded.store(count, Ordering::Relaxed);
}

pub fn track_entity_usage(count: usize) {
    get_resource_manager().tracker.entities_active.store(count, Ordering::Relaxed);
}

pub fn track_particle_usage(count: usize) {
    get_resource_manager().tracker.particles_active.store(count, Ordering::Relaxed);
}

pub fn track_pending_tasks(count: usize) {
    get_resource_manager().tracker.pending_tasks.store(count, Ordering::Relaxed);
}

pub fn track_mesh_memory_mb(mb: usize) {
    get_resource_manager().tracker.mesh_memory_mb.store(mb, Ordering::Relaxed);
}

pub fn track_texture_memory_mb(mb: usize) {
    get_resource_manager().tracker.texture_memory_mb.store(mb, Ordering::Relaxed);
}

/// Perform cleanup if needed
pub fn cleanup_if_needed() -> CleanupStats {
    get_resource_manager().cleanup_if_needed()
}

/// Check if any resource limits are exceeded
pub fn check_resource_limits() -> Vec<String> {
    let manager = get_resource_manager();
    manager.tracker.check_limits(&manager.limits)
}
