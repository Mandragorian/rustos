extern crate alloc;

use core::alloc::{AllocErr, Layout};
use alloc::alloc::AllocRef;

use core::ptr::NonNull;

const SMALL_HEAP_START: usize = 0xaab7000;
const FRAME_SIZE: usize = 0x1000;
const SLAB_SIZE: usize = 1 * FRAME_SIZE;

const SLAB_1_FRAGSIZE: usize = 128;
const SLAB_2_FRAGSIZE: usize = 256;
const SLAB_3_FRAGSIZE: usize = 512;
const SLAB_4_FRAGSIZE: usize = 1024;

pub const SLAB_1_START: usize = SMALL_HEAP_START;
pub const SLAB_2_START: usize = SLAB_1_START + SLAB_SIZE;
pub const SLAB_3_START: usize = SLAB_2_START + SLAB_SIZE;
pub const SLAB_4_START: usize = SLAB_3_START + SLAB_SIZE;


use crate::stack::{SizedBlockStack, Block};

struct SingleSlabAlloc {
    frags: SizedBlockStack,
    frag_size: usize
}

impl SingleSlabAlloc {
    pub fn new(slab_start: usize, frag_size: usize, frag_num: usize) -> SingleSlabAlloc {
        let frags = SizedBlockStack::new(slab_start, frag_size, frag_num);
        SingleSlabAlloc {
            frags,
            frag_size,
        }
    }

    pub fn allocate(&mut self) -> Result<NonNull<u8>, AllocErr> {
        self.frags.pop().ok_or(AllocErr).map(|block_ptr| {
            let block_raw = block_ptr as *mut Block;
            let addr = block_raw as u64;
            let u8_ptr = addr as *mut u8;
            NonNull::new(u8_ptr).unwrap()
        })
    }

    unsafe fn deallocate(&mut self, ptr: NonNull<u8>) {
        let block: *mut Block = ptr.cast().as_ptr();
        (*block).size(self.frag_size);
        self.frags.push(&mut *block);
    }
}

pub struct SmallAllocator {
    slab_alloc128: SingleSlabAlloc,
    slab_alloc256: SingleSlabAlloc,
    slab_alloc512: SingleSlabAlloc,
    slab_alloc1024: SingleSlabAlloc,
}

impl SmallAllocator {
    pub fn new(slab128_start: usize, slab256_start: usize,
               slab512_start: usize, slab1024_start: usize,
               slab_heap_end: usize,
    ) -> SmallAllocator {

        let slab128_size = (slab256_start - slab128_start) as usize;
        let frag128_num = slab128_size / 128;
        let slab_alloc128 = SingleSlabAlloc::new(slab128_start, 128, frag128_num);

        let slab256_size = (slab512_start - slab256_start) as usize;
        let frag256_num = slab256_size / 256;
        let slab_alloc256 = SingleSlabAlloc::new(slab256_start, 256, frag256_num);

        let slab512_size = (slab1024_start - slab512_start) as usize;
        let frag512_num = slab512_size / 512;
        let slab_alloc512 = SingleSlabAlloc::new(slab512_start, 512, frag512_num);

        let slab1024_size = (slab_heap_end - slab1024_start) as usize;
        let frag1024_num = slab1024_size / 1024;
        let slab_alloc1024 = SingleSlabAlloc::new(slab128_start, 128, frag1024_num);
        SmallAllocator {
            slab_alloc128,
            slab_alloc256,
            slab_alloc512,
            slab_alloc1024,
        }
    }

        fn allocate_next_frag(&mut self, size: usize) -> Result<(NonNull<u8>, usize), AllocErr> {
        if size <= SLAB_1_FRAGSIZE {
            self.slab_alloc128.allocate().map(|r| (r, SLAB_1_FRAGSIZE))
        } else if size <= SLAB_2_FRAGSIZE {
            self.slab_alloc256.allocate().map(|r| (r, SLAB_2_FRAGSIZE))
        } else if size <= SLAB_3_FRAGSIZE {
            self.slab_alloc512.allocate().map(|r| (r, SLAB_3_FRAGSIZE))
        } else if size <= SLAB_4_FRAGSIZE {
            self.slab_alloc1024.allocate().map(|r| (r, SLAB_4_FRAGSIZE))
        } else {
            Err(AllocErr)
        }
    }
}

unsafe impl AllocRef for SmallAllocator {
    fn alloc(&mut self, layout: Layout) -> Result<(NonNull<u8>, usize), AllocErr> {
        let size = layout.size();
        self.allocate_next_frag(size)
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let size = layout.size();

        if size < SLAB_1_FRAGSIZE {
            self.slab_alloc128.deallocate(ptr)
        } else if size < SLAB_2_FRAGSIZE {
            self.slab_alloc256.deallocate(ptr)
        } else if size < SLAB_3_FRAGSIZE {
            self.slab_alloc512.deallocate(ptr)
        } else if size == SLAB_4_FRAGSIZE {
            self.slab_alloc1024.deallocate(ptr)
        }
    }
}

