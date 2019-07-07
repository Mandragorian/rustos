use bootloader::bootinfo::{MemoryMap, MemoryRegionType};

use x86_64::structures::paging::{Mapper, Page, PageTable, RecursivePageTable};
use x86_64::structures::paging::{Size4KiB, FrameAllocator};
use x86_64::structures::paging::{PhysFrame, MapperAllSizes, MappedPageTable};
use x86_64::structures::paging::mapper::TranslateError;
use x86_64::{VirtAddr, PhysAddr};

///// Creates a RecursivePageTable instance from the level 4 address.
/////
///// This function is unsafe because it can break memory safety if an invalid
///// address is passed.
//#[feature(recursive_page_table)]
//pub unsafe fn init(level_4_table_addr: usize) -> RecursivePageTable<'static> {
//    fn init_inner(level_4_table_addr: usize) -> RecursivePageTable<'static> {
//        let level_4_table_ptr = level_4_table_addr as *mut PageTable;
//        let level_4_table = unsafe { &mut *level_4_table_ptr };
//        RecursivePageTable::new(level_4_table).unwrap()
//    }
//
//    init_inner(level_4_table_addr)
//}
//
///// Returns the physical address for the given virtual address, or `None` if the
///// virtual address is not mapped.
//#[feature(recursive_page_table)]
//pub fn translate_addr_recursive_mapping(addr: u64, recursive_page_table: &RecursivePageTable)
//    -> Result<PhysAddr, TranslateError>
//{
//    let addr = VirtAddr::new(addr);
//    let page: Page = Page::containing_address(addr);
//
//    // perform the translation
//    let frame = recursive_page_table.translate_page(page);
//    frame.map(|frame| frame.start_address() + u64::from(addr.page_offset()))
//}
//
//
//#[feature(recursive_page_table)]
//pub fn create_example_mapping(
//    recursive_page_table: &mut RecursivePageTable,
//    frame_allocator: &mut impl FrameAllocator<Size4KiB>,) {
//    use x86_64::structures::paging::PageTableFlags as Flags;
//
//    let page: Page = Page::containing_address(VirtAddr::new(0xdeadbeef000));
//    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
//    let flags = Flags::PRESENT | Flags::WRITABLE;
//
//    let map_to_result = unsafe {
//        recursive_page_table.map_to(page, frame, flags, frame_allocator)
//    };
//    map_to_result.expect("map_to failed").flush();
//}
pub struct BootInfoFrameAllocator<I>
where I: Iterator<Item = PhysFrame>
{
    pub frames: I,
}

impl<I> FrameAllocator<Size4KiB> for BootInfoFrameAllocator<I>
where I: Iterator<Item = PhysFrame>
{
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        self.frames.next()
    }
}

//impl<I> FrameDeallocator<Size4KiB> for BootInfoFrameAllocator<I>
//where I: Iterator<Item = PhysFrame>
//{
//    fn deallocate_frame(&mut self, frame: PhysFrame) {
//        self.frames.
//    }
//}

/// Create a FrameAllocator from the passed memory map
pub fn init_frame_allocator(
    memory_map: &'static MemoryMap,
) -> BootInfoFrameAllocator<impl Iterator<Item = PhysFrame>> {
    // get usable regions from memory map
    let regions = memory_map
        .iter()
        .filter(|r| r.region_type == MemoryRegionType::Usable);
    // map each region to its address range
    let addr_ranges = regions.map(|r| r.range.start_addr()..r.range.end_addr());
    // transform to an iterator of frame start addresses
    let frame_addresses = addr_ranges.flat_map(|r| r.into_iter().step_by(4096));
    // create `PhysFrame` types from the start addresses
    let frames = frame_addresses.map(|addr| {
        PhysFrame::containing_address(PhysAddr::new(addr))
    });

    BootInfoFrameAllocator { frames }
}

/// Translates frame to a virtual memory address
fn frame_to_page(frame: PhysFrame, physical_memory_offset: u64) -> VirtAddr {
    let phys = frame.start_address().as_u64();
    VirtAddr::new(phys + physical_memory_offset)
}

/// Initialize a new MappedPageTable.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: u64) -> impl MapperAllSizes {
    let level_4_table = active_level_4_table(physical_memory_offset);
    let phys_to_virt = move |frame: PhysFrame| -> *mut PageTable {
        //let phys = frame.start_address().as_u64();
        //let virt = VirtAddr::new(phys + physical_memory_offset);
        //virt.as_mut_ptr()
        frame_to_page(frame, physical_memory_offset).as_mut_ptr()
    };
    MappedPageTable::new(level_4_table, phys_to_virt)
}

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
#[feature(map_physical_memory)]
unsafe fn active_level_4_table(physical_memory_offset: u64)
    -> &'static mut PageTable
{
    use x86_64::{registers::control::Cr3, VirtAddr};

    let (level_4_table_frame, _) = Cr3::read();

    let virt = frame_to_page(level_4_table_frame, physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr // unsafe
}

/// Private function that is called by `translate_addr`.
///
/// This function is safe to limit the scope of `unsafe` because Rust treats
/// the whole body of unsafe functions as an unsafe block. This function must
/// only be reachable through `unsafe fn` from outside of this module.
#[feature(map_physical_memory)]
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: u64)
    -> Option<PhysAddr>
{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    let table_indexes = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];
    let mut frame = level_4_table_frame;

    // traverse the multi-level page table
    for &index in &table_indexes {
        // convert the frame into a page table reference
        let virt = frame.start_address().as_u64() + physical_memory_offset;
        let table_ptr: *const PageTable = VirtAddr::new(virt).as_ptr();
        let table = unsafe {&*table_ptr};

        // read the page table entry and update `frame`
        let entry = &table[index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

    // calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))
}

/// Returns the physical address for the given virtual address, or `None` if the
/// virtual address is not mapped.
#[feature(map_physical_memory)]
pub unsafe fn translate_addr_full_mapping(addr: VirtAddr, physical_memory_offset: u64) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

/// Creates an example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}
