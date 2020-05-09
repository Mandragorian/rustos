use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use crate::heap::stack::{
    BlockList,
    Block,
};

pub struct LinkedListAllocator {
    list: BlockList,
}

impl LinkedListAllocator {
    pub const fn new() -> LinkedListAllocator {
        let list = BlockList::new();
        LinkedListAllocator {
            list,
        }
    }

    pub fn init(&mut self, start: usize, size: usize) {
        let block = unsafe { Block::ref_from_address(start) };
        block.size(size);
        self.list.push(block);
    }

    pub fn allocate(&mut self, size: usize, align: usize) -> Option<usize> {
        self.list.find_block(size, align).map(|b| Block::usize_from_ref(b))
    }

    pub fn deallocate(&mut self,block: &'static mut Block) {
        self.list.push(block)
    }
}

use crate::sync::Locked;
use spin::MutexGuard;

pub struct LockedList {
    list: Locked<LinkedListAllocator>,
}

impl LockedList {
    pub const fn empty() -> LockedList {
        let list = LinkedListAllocator::new();
        let list = Locked::new(list);
        LockedList {
            list,
        }
    }

    pub fn lock(&self) -> MutexGuard<LinkedListAllocator> {
        self.list.lock()
    }
}

unsafe impl GlobalAlloc for LockedList {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut list = self.list.lock();
        let size = layout.size();
        let align = layout.align();
        list.allocate(size, align)
            .map_or_else(null_mut, |ptr| {
                assert_eq!(ptr % align, 0);
                assert!(ptr >= crate::arch::heap::LINKED_LIST_START);
                assert!(ptr + size - 1 < crate::arch::heap::HEAP_END);
                ptr as *mut u8
            })
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let raw = ptr as *mut Block;
        let b: &mut Block = &mut *raw;

        let align = layout.align();
        let size = layout.size();

        b.size(size);

        let mut list = self.list.lock();
        list.deallocate(b);
    }
}
