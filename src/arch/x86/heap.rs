use x86_64::{
    structures::paging::{
        mapper::MapToError,
        FrameAllocator,
        Mapper,
        Page,
        PageTableFlags,
        UnusedPhysFrame,
        Size4KiB,
    },
    VirtAddr,
};

const FRAME_SIZE: usize = 0x1000;
const SLAB_SIZE: usize = 16 * FRAME_SIZE;


pub const HEAP_START: usize = 0x_4444_4444_0000;

pub const SLAB_1_START: usize = HEAP_START;
pub const SLAB_2_START: usize = SLAB_1_START + SLAB_SIZE;
pub const SLAB_3_START: usize = SLAB_2_START + SLAB_SIZE;
pub const SLAB_4_START: usize = SLAB_3_START + SLAB_SIZE;

pub const LINKED_LIST_START: usize = SLAB_4_START + SLAB_SIZE;
pub const LINKED_LIST_SIZE: usize = 500 * 1024; // 100KiB

pub const HEAP_END: usize = LINKED_LIST_START + LINKED_LIST_SIZE - 1usize;
pub const HEAP_SIZE: usize = HEAP_END - HEAP_START;


pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    let page_range = {
        let heap_start = VirtAddr::new(HEAP_START as u64);
        let heap_end = heap_start + HEAP_SIZE - 1u64;
        let heap_start_page = Page::containing_address(heap_start);
        let heap_end_page = Page::containing_address(heap_end);
        Page::range_inclusive(heap_start_page, heap_end_page)
    };

    for page in page_range {
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        mapper.map_to(page, frame, flags, frame_allocator)?.flush();
    }

    unsafe {
        crate::ALLOCATOR.lock().init(LINKED_LIST_START, LINKED_LIST_SIZE);
    }

    Ok(())
}
