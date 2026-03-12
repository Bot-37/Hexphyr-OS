// kernel/src/memory.rs
//
// Physical-memory frame allocator.
//
// This module provides a lock-free bump allocator over a fixed physical range.
// It is intentionally minimal: it does not reclaim freed frames.  The intended
// use pattern is to call it during early boot to allocate page-table frames,
// then hand off to a full allocator once virtual memory is up.

#![allow(dead_code)]

use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator as X86FrameAllocator, PhysFrame, Size4KiB},
};

/// A physical memory region described by a base address and byte length.
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start:  u64,
    pub length: u64,
}

/// Lock-free bump-pointer frame allocator.
///
/// Thread safety: `allocate_frame_addr` uses an atomic CAS loop so that
/// concurrent allocations from multiple CPUs are safe without a mutex.
pub struct FrameAllocator {
    next_frame: AtomicUsize,
    end_frame:  usize,
    frame_size: usize,
}

impl FrameAllocator {
    /// Create a new allocator covering the physical range `[start, end)`.
    ///
    /// Returns `None` if the range is empty or `start >= end`.
    pub fn new(start: u64, end: u64) -> Option<Self> {
        const FRAME_SIZE: usize = 4096;
        let start_frame = (start as usize).checked_add(FRAME_SIZE - 1)? / FRAME_SIZE;
        let end_frame   = (end as usize) / FRAME_SIZE;
        if start_frame >= end_frame {
            return None;
        }
        Some(FrameAllocator {
            next_frame: AtomicUsize::new(start_frame),
            end_frame,
            frame_size: FRAME_SIZE,
        })
    }

    /// Allocate one 4 KiB physical frame and return its base `PhysAddr`.
    ///
    /// Returns `None` when the pool is exhausted.
    pub fn allocate_frame_addr(&self) -> Option<PhysAddr> {
        loop {
            let current = self.next_frame.load(Ordering::Acquire);
            if current >= self.end_frame {
                return None;
            }
            let next = current + 1;
            match self.next_frame.compare_exchange(
                current, next,
                Ordering::SeqCst,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(PhysAddr::new((current * self.frame_size) as u64)),
                Err(_) => continue, // lost the race; retry
            }
        }
    }

    /// Number of frames remaining in the pool.
    pub fn frames_remaining(&self) -> usize {
        self.end_frame
            .saturating_sub(self.next_frame.load(Ordering::Relaxed))
    }
}

/// Implement the x86_64 `FrameAllocator` trait so that `FrameAllocator` can be
/// passed directly to `Mapper::map_to` and related functions.
///
/// # Safety
/// Caller guarantees that the physical range supplied to `FrameAllocator::new`
/// does not overlap with memory already in use.
unsafe impl X86FrameAllocator<Size4KiB> for FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.allocate_frame_addr()
            .map(|addr| PhysFrame::containing_address(addr))
    }
}

/// Borrows a `FrameAllocator` by shared reference while still satisfying the
/// `x86_64::FrameAllocator` trait (which takes `&mut self`).
///
/// This allows passing a single `FrameAllocator` into multiple functions that
/// each consume an `impl FrameAllocator` without transferring ownership.
pub struct FrameAllocatorRef<'a> {
    inner: &'a FrameAllocator,
}

impl<'a> FrameAllocatorRef<'a> {
    pub fn new(inner: &'a FrameAllocator) -> Self {
        FrameAllocatorRef { inner }
    }
}

/// # Safety
/// Same contract as `FrameAllocator`'s implementation.
unsafe impl<'a> X86FrameAllocator<Size4KiB> for FrameAllocatorRef<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.inner
            .allocate_frame_addr()
            .map(|addr| PhysFrame::containing_address(addr))
    }
}


