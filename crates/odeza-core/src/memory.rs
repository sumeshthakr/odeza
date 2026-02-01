//! Memory Management
//!
//! Custom allocators optimized for game engine workloads:
//! - Frame allocator for per-frame temporary data
//! - Arena allocator for grouped allocations
//! - Pool allocator for fixed-size objects
//! - Memory tracking and budget enforcement

use std::alloc::{alloc, dealloc, Layout};
use std::cell::UnsafeCell;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

use parking_lot::Mutex;

/// Memory budget configuration for subsystems
#[derive(Debug, Clone)]
pub struct MemoryBudget {
    /// Maximum bytes for this subsystem
    pub max_bytes: usize,
    /// Warning threshold (percentage of max)
    pub warning_threshold: f32,
}

impl Default for MemoryBudget {
    fn default() -> Self {
        Self {
            max_bytes: 256 * 1024 * 1024, // 256 MB default
            warning_threshold: 0.8,
        }
    }
}

/// Memory tracking statistics
#[derive(Debug, Default)]
pub struct MemoryStats {
    /// Total bytes allocated
    pub allocated: AtomicUsize,
    /// Peak bytes allocated
    pub peak: AtomicUsize,
    /// Number of allocations
    pub allocation_count: AtomicUsize,
}

impl MemoryStats {
    /// Create new memory stats
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an allocation
    pub fn record_alloc(&self, size: usize) {
        let new_size = self.allocated.fetch_add(size, Ordering::Relaxed) + size;
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        
        // Update peak if needed
        let mut peak = self.peak.load(Ordering::Relaxed);
        while new_size > peak {
            match self.peak.compare_exchange_weak(
                peak,
                new_size,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(current) => peak = current,
            }
        }
    }

    /// Record a deallocation
    pub fn record_dealloc(&self, size: usize) {
        self.allocated.fetch_sub(size, Ordering::Relaxed);
    }

    /// Get current allocated bytes
    pub fn current(&self) -> usize {
        self.allocated.load(Ordering::Relaxed)
    }

    /// Get peak allocated bytes
    pub fn peak_usage(&self) -> usize {
        self.peak.load(Ordering::Relaxed)
    }

    /// Get total allocation count
    pub fn count(&self) -> usize {
        self.allocation_count.load(Ordering::Relaxed)
    }

    /// Reset statistics
    pub fn reset(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.peak.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
    }
}

/// Frame allocator for per-frame temporary allocations
///
/// Allocations from this allocator are valid only for the current frame.
/// At the end of each frame, the allocator is reset, freeing all memory at once.
pub struct FrameAllocator {
    /// Base pointer to the memory block
    base: NonNull<u8>,
    /// Size of the memory block
    capacity: usize,
    /// Current offset into the block
    offset: AtomicUsize,
    /// Memory statistics
    stats: MemoryStats,
}

impl FrameAllocator {
    /// Create a new frame allocator with the given capacity
    pub fn new(capacity: usize) -> Self {
        let layout = Layout::from_size_align(capacity, 16).expect("Invalid layout");
        let ptr = unsafe { alloc(layout) };
        let base = NonNull::new(ptr).expect("Allocation failed");
        
        Self {
            base,
            capacity,
            offset: AtomicUsize::new(0),
            stats: MemoryStats::new(),
        }
    }

    /// Allocate memory from the frame allocator
    ///
    /// Returns None if there is not enough space remaining
    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let align = align.max(1);
        
        loop {
            let current_offset = self.offset.load(Ordering::Relaxed);
            
            // Calculate aligned offset
            let aligned_offset = (current_offset + align - 1) & !(align - 1);
            let new_offset = aligned_offset + size;
            
            if new_offset > self.capacity {
                return None;
            }
            
            // Try to reserve the space
            match self.offset.compare_exchange_weak(
                current_offset,
                new_offset,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => {
                    self.stats.record_alloc(size);
                    let ptr = unsafe { self.base.as_ptr().add(aligned_offset) };
                    return NonNull::new(ptr);
                }
                Err(_) => continue,
            }
        }
    }

    /// Allocate and zero-initialize memory
    pub fn alloc_zeroed(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let ptr = self.alloc(size, align)?;
        unsafe {
            std::ptr::write_bytes(ptr.as_ptr(), 0, size);
        }
        Some(ptr)
    }

    /// Reset the allocator for the next frame
    ///
    /// This invalidates all previous allocations!
    pub fn reset(&self) {
        self.offset.store(0, Ordering::Relaxed);
        self.stats.reset();
    }

    /// Get the current usage
    pub fn used(&self) -> usize {
        self.offset.load(Ordering::Relaxed)
    }

    /// Get the capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Get remaining space
    pub fn remaining(&self) -> usize {
        self.capacity - self.used()
    }

    /// Get memory statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }
}

impl Drop for FrameAllocator {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 16).expect("Invalid layout");
        unsafe {
            dealloc(self.base.as_ptr(), layout);
        }
    }
}

// Safety: The frame allocator uses atomic operations for thread safety
unsafe impl Send for FrameAllocator {}
unsafe impl Sync for FrameAllocator {}

/// Arena allocator for grouped allocations
///
/// All allocations from an arena are freed together when the arena is dropped or reset.
pub struct ArenaAllocator {
    /// Memory blocks
    blocks: Mutex<Vec<ArenaBlock>>,
    /// Current block index
    current_block: AtomicUsize,
    /// Block size
    block_size: usize,
    /// Memory statistics
    stats: MemoryStats,
}

struct ArenaBlock {
    base: NonNull<u8>,
    capacity: usize,
    offset: usize,
}

impl ArenaBlock {
    fn new(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 16).expect("Invalid layout");
        let ptr = unsafe { alloc(layout) };
        let base = NonNull::new(ptr).expect("Allocation failed");
        
        Self {
            base,
            capacity: size,
            offset: 0,
        }
    }

    fn alloc(&mut self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let aligned_offset = (self.offset + align - 1) & !(align - 1);
        let new_offset = aligned_offset + size;
        
        if new_offset > self.capacity {
            return None;
        }
        
        self.offset = new_offset;
        let ptr = unsafe { self.base.as_ptr().add(aligned_offset) };
        NonNull::new(ptr)
    }

    fn reset(&mut self) {
        self.offset = 0;
    }
}

impl Drop for ArenaBlock {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(self.capacity, 16).expect("Invalid layout");
        unsafe {
            dealloc(self.base.as_ptr(), layout);
        }
    }
}

impl ArenaAllocator {
    /// Create a new arena allocator with the given block size
    pub fn new(block_size: usize) -> Self {
        let initial_block = ArenaBlock::new(block_size);
        
        Self {
            blocks: Mutex::new(vec![initial_block]),
            current_block: AtomicUsize::new(0),
            block_size,
            stats: MemoryStats::new(),
        }
    }

    /// Allocate memory from the arena
    pub fn alloc(&self, size: usize, align: usize) -> Option<NonNull<u8>> {
        let mut blocks = self.blocks.lock();
        let current = self.current_block.load(Ordering::Relaxed);
        
        // Try to allocate from current block
        if let Some(block) = blocks.get_mut(current) {
            if let Some(ptr) = block.alloc(size, align) {
                self.stats.record_alloc(size);
                return Some(ptr);
            }
        }
        
        // Need a new block
        let new_block_size = self.block_size.max(size);
        let mut new_block = ArenaBlock::new(new_block_size);
        let ptr = new_block.alloc(size, align);
        blocks.push(new_block);
        self.current_block.store(blocks.len() - 1, Ordering::Relaxed);
        
        self.stats.record_alloc(size);
        ptr
    }

    /// Allocate and initialize with a value
    pub fn alloc_init<T>(&self, value: T) -> Option<&mut T> {
        let size = std::mem::size_of::<T>();
        let align = std::mem::align_of::<T>();
        let ptr = self.alloc(size, align)?;
        
        unsafe {
            let typed_ptr = ptr.as_ptr() as *mut T;
            std::ptr::write(typed_ptr, value);
            Some(&mut *typed_ptr)
        }
    }

    /// Reset the arena, freeing all allocations
    pub fn reset(&self) {
        let mut blocks = self.blocks.lock();
        for block in blocks.iter_mut() {
            block.reset();
        }
        // Keep only the first block
        blocks.truncate(1);
        self.current_block.store(0, Ordering::Relaxed);
        self.stats.reset();
    }

    /// Get memory statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }
}

/// Pool allocator for fixed-size objects
///
/// Efficient allocation and deallocation of objects of a single size.
pub struct PoolAllocator<T> {
    /// Storage for objects
    storage: Mutex<PoolStorage<T>>,
    /// Memory statistics
    stats: MemoryStats,
}

struct PoolStorage<T> {
    /// Allocated chunks
    chunks: Vec<Box<[UnsafeCell<std::mem::MaybeUninit<T>>]>>,
    /// Free list
    free_list: Vec<*mut T>,
    /// Chunk size
    chunk_size: usize,
}

impl<T> PoolAllocator<T> {
    /// Create a new pool allocator with the given chunk size
    pub fn new(chunk_size: usize) -> Self {
        let chunk_size = chunk_size.max(16);
        
        Self {
            storage: Mutex::new(PoolStorage {
                chunks: Vec::new(),
                free_list: Vec::with_capacity(chunk_size),
                chunk_size,
            }),
            stats: MemoryStats::new(),
        }
    }

    /// Allocate an object from the pool
    pub fn alloc(&self) -> Option<NonNull<T>> {
        let mut storage = self.storage.lock();
        
        // Try to get from free list
        if let Some(ptr) = storage.free_list.pop() {
            self.stats.record_alloc(std::mem::size_of::<T>());
            return NonNull::new(ptr);
        }
        
        // Allocate new chunk
        let chunk: Box<[UnsafeCell<std::mem::MaybeUninit<T>>]> = (0..storage.chunk_size)
            .map(|_| UnsafeCell::new(std::mem::MaybeUninit::uninit()))
            .collect();
        
        // Add all slots to free list (except the first which we return)
        for slot in chunk.iter().skip(1) {
            storage.free_list.push(slot.get() as *mut T);
        }
        
        let first_ptr = chunk[0].get() as *mut T;
        storage.chunks.push(chunk);
        
        self.stats.record_alloc(std::mem::size_of::<T>());
        NonNull::new(first_ptr)
    }

    /// Allocate and initialize an object
    pub fn alloc_init(&self, value: T) -> Option<&mut T> {
        let ptr = self.alloc()?;
        unsafe {
            std::ptr::write(ptr.as_ptr(), value);
            Some(&mut *ptr.as_ptr())
        }
    }

    /// Return an object to the pool
    ///
    /// # Safety
    /// The pointer must have been allocated from this pool
    pub unsafe fn dealloc(&self, ptr: NonNull<T>) {
        let mut storage = self.storage.lock();
        storage.free_list.push(ptr.as_ptr());
        self.stats.record_dealloc(std::mem::size_of::<T>());
    }

    /// Get memory statistics
    pub fn stats(&self) -> &MemoryStats {
        &self.stats
    }
}

// Safety: Pool uses mutex for synchronization
unsafe impl<T: Send> Send for PoolAllocator<T> {}
unsafe impl<T: Send> Sync for PoolAllocator<T> {}

/// Global memory tracker for all subsystems
pub struct MemoryTracker {
    subsystems: Mutex<Vec<(String, MemoryStats, MemoryBudget)>>,
}

impl MemoryTracker {
    /// Create a new memory tracker
    pub fn new() -> Self {
        Self {
            subsystems: Mutex::new(Vec::new()),
        }
    }

    /// Register a subsystem for tracking
    pub fn register_subsystem(&self, name: &str, budget: MemoryBudget) -> usize {
        let mut subsystems = self.subsystems.lock();
        let id = subsystems.len();
        subsystems.push((name.to_string(), MemoryStats::new(), budget));
        id
    }

    /// Record an allocation for a subsystem
    pub fn record_alloc(&self, subsystem_id: usize, size: usize) -> bool {
        let subsystems = self.subsystems.lock();
        if let Some((_, stats, budget)) = subsystems.get(subsystem_id) {
            let new_size = stats.allocated.load(Ordering::Relaxed) + size;
            if new_size > budget.max_bytes {
                return false; // Would exceed budget
            }
            stats.record_alloc(size);
            true
        } else {
            false
        }
    }

    /// Record a deallocation for a subsystem
    pub fn record_dealloc(&self, subsystem_id: usize, size: usize) {
        let subsystems = self.subsystems.lock();
        if let Some((_, stats, _)) = subsystems.get(subsystem_id) {
            stats.record_dealloc(size);
        }
    }

    /// Get usage report for all subsystems
    pub fn get_report(&self) -> Vec<(String, usize, usize, f32)> {
        let subsystems = self.subsystems.lock();
        subsystems
            .iter()
            .map(|(name, stats, budget)| {
                let used = stats.current();
                let usage_percent = used as f32 / budget.max_bytes as f32 * 100.0;
                (name.clone(), used, budget.max_bytes, usage_percent)
            })
            .collect()
    }
}

impl Default for MemoryTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_allocator() {
        let allocator = FrameAllocator::new(1024);
        
        let ptr1 = allocator.alloc(100, 8);
        assert!(ptr1.is_some());
        
        let ptr2 = allocator.alloc(200, 16);
        assert!(ptr2.is_some());
        
        assert!(allocator.used() >= 300);
        
        allocator.reset();
        assert_eq!(allocator.used(), 0);
    }

    #[test]
    fn test_frame_allocator_overflow() {
        let allocator = FrameAllocator::new(100);
        
        let ptr1 = allocator.alloc(50, 8);
        assert!(ptr1.is_some());
        
        // This should fail - not enough space
        let ptr2 = allocator.alloc(100, 8);
        assert!(ptr2.is_none());
    }

    #[test]
    fn test_arena_allocator() {
        let arena = ArenaAllocator::new(256);
        
        let ptr1 = arena.alloc(64, 8);
        assert!(ptr1.is_some());
        
        let ptr2 = arena.alloc(64, 8);
        assert!(ptr2.is_some());
        
        // Allocate something larger than block size
        let ptr3 = arena.alloc(512, 8);
        assert!(ptr3.is_some());
        
        arena.reset();
    }

    #[test]
    fn test_arena_alloc_init() {
        let arena = ArenaAllocator::new(256);
        
        let value = arena.alloc_init(42u64);
        assert!(value.is_some());
        assert_eq!(*value.unwrap(), 42);
    }

    #[test]
    fn test_pool_allocator() {
        let pool: PoolAllocator<u64> = PoolAllocator::new(16);
        
        let ptr1 = pool.alloc();
        assert!(ptr1.is_some());
        
        let ptr2 = pool.alloc();
        assert!(ptr2.is_some());
        
        // Return one to pool
        unsafe {
            pool.dealloc(ptr1.unwrap());
        }
        
        // Should reuse the freed slot
        let ptr3 = pool.alloc();
        assert!(ptr3.is_some());
    }

    #[test]
    fn test_pool_alloc_init() {
        let pool: PoolAllocator<String> = PoolAllocator::new(8);
        
        let value = pool.alloc_init(String::from("hello"));
        assert!(value.is_some());
        assert_eq!(value.unwrap(), "hello");
    }

    #[test]
    fn test_memory_tracker() {
        let tracker = MemoryTracker::new();
        
        let id = tracker.register_subsystem("test", MemoryBudget {
            max_bytes: 1000,
            warning_threshold: 0.8,
        });
        
        assert!(tracker.record_alloc(id, 500));
        assert!(tracker.record_alloc(id, 400));
        
        // This should fail - would exceed budget
        assert!(!tracker.record_alloc(id, 200));
        
        tracker.record_dealloc(id, 300);
        
        // Now this should succeed
        assert!(tracker.record_alloc(id, 200));
    }

    #[test]
    fn test_memory_stats() {
        let stats = MemoryStats::new();
        
        stats.record_alloc(100);
        stats.record_alloc(200);
        
        assert_eq!(stats.current(), 300);
        assert_eq!(stats.peak_usage(), 300);
        assert_eq!(stats.count(), 2);
        
        stats.record_dealloc(100);
        
        assert_eq!(stats.current(), 200);
        assert_eq!(stats.peak_usage(), 300); // Peak unchanged
    }
}
