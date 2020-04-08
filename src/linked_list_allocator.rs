use alloc::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;

use crate::{println};

use crate::stack::{
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

    pub fn allocate(&mut self, size: usize ) -> Option<usize> {

        let mut block = Block {
            next: self.list.head.take(),
            size: 0,
        };

        let mut cur = &mut block;
        while let Some(ref mut b) = cur.next {
            if let Ok(()) = Block::split(b, size) {
                let addr = Block::usize_from_ref(b);
                println!("allocating: {:x}", addr); 
                cur.next = b.next.take();
                self.list.head = block.next.take();
                return Some(addr);
            } else {
                cur = cur.next.as_mut().unwrap();
            }
        }
        self.list.head = block.next.take();
        None
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
        list.allocate(size)
            .map_or_else(null_mut, |ptr| ptr as *mut u8)
    }
    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        let raw = ptr as *mut Block;
        let b = &mut *raw;

        let mut list = self.list.lock();
        list.deallocate(b);
    }
}
