// src/memory.rs
#![allow(dead_code)]

use crate::BootInfo;
use alloc::boxed::Box;
use alloc::vec::Vec;
use core::alloc::Layout;
use core::ptr;
use core::sync::atomic::{AtomicUsize, Ordering};
use x86_64::{
    PhysAddr,
    structures::paging::{FrameAllocator as X86FrameAllocator, PhysFrame, Size4KiB},
};
use linked_list_allocator::LockedHeap;

/// A physical memory region (base physical address and length in bytes).
#[derive(Debug, Clone, Copy)]
pub struct MemoryRegion {
    pub start: u64,
    pub length: u64,
}

/// Simple bump-frame allocator over an assigned physical range.
/// Not a full-featured reclaiming allocator, but OK for early use.
pub struct FrameAllocator {
    next_frame: AtomicUsize,
    end_frame: usize,
    frame_size: usize,
}

impl FrameAllocator {
    pub fn new(start: u64, end: u64) -> Option<Self> {
        let frame_size = 4096usize;
        let start_frame = (start as usize) / frame_size;
        let end_frame = (end as usize) / frame_size;
        if start_frame >= end_frame {
            return None;
        }
        Some(FrameAllocator {
            next_frame: AtomicUsize::new(start_frame),
             end_frame,
             frame_size,
        })
    }

    /// Allocate a physical frame and return its physical address
    pub fn allocate_frame_addr(&self) -> Option<PhysAddr> {
        loop {
            let current = self.next_frame.load(Ordering::Acquire);
            if current >= self.end_frame {
                return None;
            }
            let next = current + 1;
            if self.next_frame.compare_exchange(current, next, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                let addr = (current * self.frame_size) as u64;
                return Some(PhysAddr::new(addr));
            }
        }
    }

    /// helper for testing
    pub fn total_frames(&self) -> usize {
        self.end_frame - (self.next_frame.load(Ordering::Relaxed) - 1)
    }
}

/// Implement x86_64 FrameAllocator trait for 4 KiB frames (unsafe as required).
unsafe impl X86FrameAllocator<Size4KiB> for FrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Use the atomic-backed allocate function and convert to PhysFrame
        self.allocate_frame_addr()
        .map(|addr| PhysFrame::containing_address(addr))
    }
}

/// Simple wrapper type so other modules can take a mutable reference to the allocator
pub struct FrameAllocatorRef<'a> {
    inner: &'a FrameAllocator,
}

impl<'a> FrameAllocatorRef<'a> {
    pub fn new(inner: &'a FrameAllocator) -> Self {
        Self { inner }
    }
}

unsafe impl<'a> X86FrameAllocator<Size4KiB> for FrameAllocatorRef<'a> {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // we rely on atomic ops inside FrameAllocator: safe to call from a mutable ref shim
        self.inner.allocate_frame_addr().map(|a| PhysFrame::containing_address(a))
    }
}

/// Kernel heap allocator (global)
#[global_allocator]
static HEAP_ALLOCATOR: LockedHeap = LockedHeap::empty();

/// Initialize memory: set up kernel heap and create a global (leaked) FrameAllocator.
///
/// Returns a `'static` reference to the frame allocator and the heap base/size used.
///
/// Why leak Box: we need a `'static` FrameAllocator reference usable across the kernel.
/// This is acceptable during early kernel boot; later we can replace with a proper global.
///
/// Note: This function does NOT parse UEFI memory map fully; it selects a conservative region
/// after the kernel image to host the heap and frame allocator.
pub fn init(boot: &BootInfo) -> (&'static FrameAllocator, usize, usize) {
    // choose heap start = page-align(kernel_end)
    let heap_start_phys = align_up(boot.kernel_end_phys, 0x1000);
    let heap_size: usize = 8 * 1024 * 1024; // 8 MiB for kernel heap

    // pick frame allocator region immediately after heap (128 MiB window)
    let frame_alloc_start = heap_start_phys + (heap_size as u64);
    let frame_alloc_end = frame_alloc_start + (128 * 1024 * 1024u64); // 128 MiB reserved for frames

    // create frame allocator and leak it to static lifetime
    let fa = FrameAllocator::new(frame_alloc_start, frame_alloc_end)
    .expect("failed to create FrameAllocator");
    let fa_static: &'static FrameAllocator = Box::leak(Box::new(fa));

    // initialize global heap (linked_list_allocator expects pointer+size as usize)
    unsafe {
        HEAP_ALLOCATOR.lock().init(heap_start_phys as usize, heap_size);
    }

    // test allocations (optional)
    // let _ = HEAP_ALLOCATOR.lock(); // if you want to use the allocator now

    (fa_static, heap_start_phys as usize, heap_size)
}

fn align_up(addr: u64, align: u64) -> u64 {
    (addr + align - 1) & !(align - 1)
}

fn align_down(addr: u64, align: u64) -> u64 {
    addr & !(align - 1)
}
