use crate::{print, println};

pub struct Block {
    next: Option<&'static mut Block>,
    size: usize,
}

pub fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}


fn copy_pointer(r: & Block) ->  &'static mut Block {
    let r_as_raw = r as *const Block;
    let new = r_as_raw.clone() as usize;
    unsafe { Block::ref_from_address(new) }
}

impl Block {

    pub const fn new() -> Block {
        Block {
            size: 0,
            next: None,
        }
    }

    pub unsafe fn ref_from_address(addr: usize) -> &'static mut Block {
        let block = addr as *mut Block;
        &mut *block
    }

    pub fn raw_from_ref(b: &Block) -> *mut u8 {
        let raw = b as *const Block;
        raw as *mut u8
    }

    pub fn usize_from_ref(b: &Block) -> usize {
        let raw = b as *const Block;
        raw as usize
    }

    pub fn size(&mut self, size: usize) {
        self.size = size;
    }

    pub fn get_size(&self) -> usize {
        self.size
    }

    /// Check if the block is big enough to allocate requested_size memory
    ///
    /// If the block is too big, a new block will be created to prevent fragmentation
    pub fn split(block: &mut Block, requested_size: usize) -> Result<Option<&'static mut Block>, ()> {
        let min_block_size = core::mem::size_of::<Block>();

        // We can't allocate memory less than min_block_size, otherwise we would not
        // be able to reinsert it in the list
        let size = core::cmp::max(requested_size, min_block_size);

        if block.get_size() < size {
            return Err(());
        }

        // The block is big enough, but too big
        // We split
        if block.get_size() > size + min_block_size {
            let addr = Block::usize_from_ref(block);
            let new_addr = addr + size;
            let old_size = block.get_size();

            let mut new_block = unsafe { Block::ref_from_address(new_addr) };
            new_block.size(old_size - size);
            Ok(Some(new_block))
        } else {
            Ok(None)
        }
    }

    /// Check if the block is big enough to allocate `requested_size` memeory aligned to `align`.
    pub fn split_aligned(
        block: &mut Block,
        requested_size: usize,
        align: usize,
    ) -> Result<(Option<&'static mut Block>, Option<&'static mut Block>), ()> {

        let min_block_size = core::mem::size_of::<Block>();
        let block_size = block.get_size();
        let block_start = Block::usize_from_ref(block);
        let block_end = block_start + block_size - 1;

        // If block already aligned, just call split
        if block_start % align == 0 {
            Block::split(block, requested_size).map(|r| (None, r))
        } else {
            let mut aligned_start = align_up(block_start, align);

            // Increase aligned_start untill padding is big enough to hold a block
            while aligned_start - block_start < min_block_size {
                aligned_start = align_up(aligned_start + 1, align);
            }

            // Aligned address exceeds block memory region
            if aligned_start > block_end {
                return Err(());
            }

            // Size of block after alignment is too small
            if block_end - aligned_start + 1 < requested_size {
                return Err(());
            }

            // XXX: This API probably sucks
            //
            // due to the way the linked list was written this API does some weird
            // juggling. The linked list in find_block used to assume that if split() returns Ok,
            // then the current block can be used for allocation. When memory alignment was added
            // this is no longer true as the aligned address will not be the same as the block's
            // address. This resulted in quite a bit of convoluted code. I should sanitize it.

            // rest is the block that holds the memory region after the alignment padding
            // The unwraps here are because we already know that the split will be successful, and
            // also create a new block
            let rest = Block::split(block, aligned_start - block_start).unwrap().unwrap();

            // right is the block that holds the memory after the allocated region
            // The unwrap here is because we know that the split will be successful, but we don't
            // know if it will create a new block.
            let right = Block::split(rest, requested_size).unwrap();
            block.size(aligned_start - block_start);

            Ok((Some(rest), right))

        }
    }
}

impl core::fmt::Debug for Block {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self.next {
            None => f.debug_struct("Block")
                .field("next", &"None")
                .field("size", &(self.size))
                .finish(),
            Some(b_ptr) => {
                let raw = *b_ptr as *const Block;
                f.debug_struct("Block")
                    .field("next", &(raw as u64))
                    .field("size", &(self.size as u64))
                    .finish()
            }
        }
    }
}

#[derive(Debug)]
pub struct BlockList {
    len: usize,
    pub head: Block,
}

impl BlockList {
    pub const fn new() -> BlockList {
        let head = Block::new();
        BlockList { len: 0, head }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, block: &'static mut Block) {
        let mut hint = self.head.next.take();
        self.insert(&mut hint, block);
        self.head.next = hint.take();
    }

    pub fn pop(&mut self) -> Option<&'static mut Block> {
        self.head.next.take().map(|block| {
            self.len -= 1;
            self.head.next = block.next.take();
            block
        })
    }

    pub fn insert(
        &mut self,
        hint: &mut Option<&'static mut Block>,
        block: &'static mut Block,
    ) {
        self.len += 1;
        block.next = hint.take();
        *hint = Some(block);
    }

    pub fn find_block(&mut self, size: usize, align: usize) -> Option<&'static mut Block> {
        let mut cur = &mut self.head;
            while let Some(ref mut b) = cur.next {
                let res = Block::split_aligned(b, size, align);
                if let Ok(ok) = res {
                    let (new_next, to_return): (Option<&'static mut Block>, _) = match ok {
                        // No alignment padding, and no splitted block.
                        // Change cur.next to point to b.next.
                        // Return b's start address
                        (None, None) => (b.next.take(), copy_pointer(b)),

                        // Alignment padding but not splitted.
                        // cur.next should still be pointing to this block.
                        // Return alloc, which is the block that starts at the aligned address
                        (Some(alloc), None) => {
                            (Some(copy_pointer(b)), copy_pointer(alloc))
                        },

                        // No alignment but splitted block
                        // cur.next should be right. right.next should be b.next to keep list
                        // connected
                        // Return the start address of b
                        (None, Some(mut right)) => {
                            right.next = b.next.take();
                            (Some(right), copy_pointer(b))
                        },

                        // Alignment pad and splitted block
                        // cur.next should still be this block, this block's next should be right,
                        // right next should be this block's next to keep list connected.
                        // Return alloc which is a block with aligned address.
                        (Some(alloc), Some(mut right)) => {
                            right.next = b.next.take();
                            b.next = Some(right);
                            (Some(copy_pointer(b)), copy_pointer(alloc))
                        },
                    };
                    cur.next = new_next;
                    return Some(to_return);
                } else {
                    cur = cur.next.as_mut().unwrap();
                }
            }
            None
    }

    pub fn iter(&mut self) -> Iter {
        let cur = match &self.head.next {
            None => None,
            Some(r) => Some(copy_pointer(r)),
        };
        Iter {
            cur,
        }
    }
}

pub struct Iter {
    cur: Option<&'static mut Block>,
}

impl core::iter::Iterator for Iter {
    type Item = &'static mut Block;
    fn next(&mut self) -> Option<Self::Item> {
        let old_cur = self.cur.take();
        old_cur.map(|b| {
            self.cur = b.next.take();
            b
        })
    }
}

#[derive(Debug)]
pub struct SizedBlockStack {
    list: BlockList,
}

impl SizedBlockStack {
    pub fn new(slab_start: usize, frag_size: usize, frag_num: usize) -> SizedBlockStack {
        let mut list = BlockList::new();
        let mut addr = slab_start;
        for _ in 0..frag_num {
            let block_ref = unsafe { Block::ref_from_address(addr) };
            (*block_ref).size(frag_size);
            list.push(block_ref); 
            addr += frag_size;
        }
        SizedBlockStack { list }
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    fn empty() -> SizedBlockStack {
        let list = BlockList::new();
        SizedBlockStack { list }
    }

    pub fn push(&mut self, block: &'static mut Block) {
        self.list.push(block)
    }

    pub fn pop(&mut self) -> Option<&'static mut Block> {
        self.list.pop()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{serial_print, serial_println};

    #[test_case]
    fn stack_test() {
        let size = core::mem::size_of::<Block>();
        let num = 4096 / size;
        assert!( size <= 128 );
        assert!( (size & (size - 1)) == 0 );

        serial_print!("testing SizedBlockStack...");

        let mut stack = SizedBlockStack::new(crate::arch::heap::SLAB_1_START, size, num);
        let mut start_addr = crate::arch::heap::SLAB_1_START + (num - 1) * size;

        while let Some(b_ptr) = stack.pop() {
            assert_eq!(b_ptr.size, size);
            let raw = b_ptr as *const Block;
            assert_eq!(start_addr, raw as usize);
            start_addr -= size;
        }
        serial_println!("[ok]");
    }
}
