// src/arch/x86_64/paging.rs
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{
        OffsetPageTable, Page, PageTable, PageTableFlags,
        PhysFrame, Size4KiB, Mapper, mapper::MapperFlush,
    },
    registers::control::Cr3,
};
use log::info;
use crate::memory::FrameAllocatorRef;

/// Initialize an OffsetPageTable using the active level 4 table and
/// a virtual `physical_memory_offset` that maps physical memory.
pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    let level4_table = active_level4_table(physical_memory_offset);
    OffsetPageTable::new(level4_table, physical_memory_offset)
}

/// Helper: get mutable reference to the active level 4 page table
unsafe fn active_level4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    let (level4_frame, _) = Cr3::read();
    let phys = level4_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    &mut *page_table_ptr
}

/// Map a physical range [start, end) into virtual addresses starting at (physical_memory_offset + phys)
/// i.e. for every physical frame at `p`, create a mapping virtual = physical_memory_offset + p.
/// This is the standard higher-half mapping approach.
pub fn map_phys_to_virt_range<A>(
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut A,
    phys_start: PhysAddr,
    phys_end: PhysAddr,
    physical_memory_offset: VirtAddr,
    flags: PageTableFlags,
) -> Result<(), &'static str>
where
A: x86_64::structures::paging::FrameAllocator<Size4KiB>,
{
    // Frame containing the first physical page
    let start_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(phys_start);
    // Inclusive last frame (phys_end is exclusive)
    let end_frame: PhysFrame<Size4KiB> =
    PhysFrame::containing_address(PhysAddr::new(phys_end.as_u64().saturating_sub(1)));

    for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
        // physical address base
        let phys = frame.start_address();
        // virtual address = physical_memory_offset + physical_base
        let virt = physical_memory_offset + phys.as_u64();
        let page = Page::containing_address(VirtAddr::new(virt.as_u64()));

        unsafe {
            mapper
            .map_to(page, frame, flags, frame_allocator)
            .map_err(|_| "map_to failed")?
            .flush();
        }
    }
    Ok(())
}

/// Convenience: map kernel frames using standard kernel flags (present + writable)
pub fn kernel_flags() -> PageTableFlags {
    PageTableFlags::PRESENT | PageTableFlags::WRITABLE
}

/// Convenience: user flags
pub fn user_flags() -> PageTableFlags {
    PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::USER_ACCESSIBLE
}
