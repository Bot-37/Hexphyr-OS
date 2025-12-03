use crate::BootInfo;
use x86_64::structures::paging::{PageTable, OffsetPageTable};
use x86_64::VirtAddr;

pub fn init(boot_info: &BootInfo) {
    // Convert physical addresses to virtual
    let physical_memory_offset = VirtAddr::new(0xffff_8000_0000_0000);
    
    // TODO: Parse UEFI memory map and initialize frame allocator
    log::info!("Memory map at: {:p}", boot_info.memory_map);
    log::info!("Memory map size: {} bytes", boot_info.map_size);
}

pub struct MemoryController {
    page_table: OffsetPageTable<'static>,
}

impl MemoryController {
    pub unsafe fn new(physical_memory_offset: VirtAddr) -> Self {
        let level4_table = active_level4_table(physical_memory_offset);
        let page_table = OffsetPageTable::new(level4_table, physical_memory_offset);
        
        MemoryController { page_table }
    }
}

unsafe fn active_level4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;
    
    let (level4_table_frame, _) = Cr3::read();
    
    let phys = level4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();
    
    &mut *page_table_ptr
}