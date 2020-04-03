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


pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub fn init(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    init_heap(mapper, frame_allocator)
        .and_then(|_| init_all_slabs(mapper, frame_allocator))
}

fn init_heap(
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
        crate::ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
}

fn init_all_slabs(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>> {
    init_slab(mapper, frame_allocator, crate::slab::SLAB_1_START)
        .and_then(|_| init_slab(mapper, frame_allocator,
                                crate::slab::SLAB_2_START))
        .and_then(|_| init_slab(mapper, frame_allocator,
                                crate::slab::SLAB_3_START))
        .and_then(|_| init_slab(mapper, frame_allocator,
                                crate::slab::SLAB_4_START))

}
pub fn init_slab(
    mapper: &mut impl Mapper<Size4KiB>,
    allocator: &mut impl FrameAllocator<Size4KiB>,
    slab_start: usize,
) -> Result<(), MapToError<Size4KiB>> {
    let frame = allocator.allocate_frame().unwrap().frame();

    let page = Page::containing_address(VirtAddr::new(slab_start as u64));

    use x86_64::structures::paging::PageTableFlags as Flags;
    let flags = Flags::PRESENT | Flags::WRITABLE;

    unsafe {
        mapper.map_to(page, UnusedPhysFrame::new(frame), flags, allocator)?
            .flush();
    }
    Ok(())
}
