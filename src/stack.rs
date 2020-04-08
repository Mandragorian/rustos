
pub struct Block {
    pub next: Option<&'static mut Block>,
    pub size: usize,
}

fn copy_pointer(r: &mut Block) ->  &'static mut Block {
    let r_as_raw = r as *const Block;
    let new = r_as_raw.clone() as usize;
    unsafe { Block::ref_from_address(new) }
}

impl Block {
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

    pub fn split(block: &mut Block, requested_size: usize) -> Result<(), ()> {
        let min_block_size = core::mem::size_of::<Block>();
        let size = core::cmp::max(requested_size, min_block_size);
        if block.get_size() < size {
            return Err(());
        } else if block.get_size() > size + min_block_size {
            let raw = Block::raw_from_ref(block);
            let addr = raw as usize;
            let new_addr = addr + size;
            let old_size = block.get_size();

            let mut new_block = unsafe { Block::ref_from_address(new_addr) };
            new_block.size(old_size - size);
            new_block.next = block.next.take();
            block.next = Some(new_block);
        }
        Ok(())
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
    pub head: Option<&'static mut Block>,
}

impl BlockList {
    pub const fn new() -> BlockList {
        BlockList { len: 0, head: None }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn push(&mut self, block: &'static mut Block) {
        let mut hint = self.head.take();
        self.insert(&mut hint, block);
        self.head = hint.take();
    }

    pub fn pop(&mut self) -> Option<&'static mut Block> {
        self.head.take().map(|block| {
            self.len -= 1;
            self.head = block.next.take();
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

    pub fn iter(&mut self) -> Iter {
        let head = self.head.take();
        let (cur, mut head) = match head {
            None => (None, None),
            Some(r) => (Some(copy_pointer(r)), Some(r))
        };
        self.head = head.take();
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

        let mut stack = SizedBlockStack::new(crate::slab::SLAB_1_START, size, num);
        let mut start_addr = crate::slab::SLAB_1_START + (num - 1) * size;

        while let Some(b_ptr) = stack.pop() {
            assert_eq!(b_ptr.size, size);
            let raw = b_ptr as *const Block;
            assert_eq!(start_addr, raw as usize);
            start_addr -= size;
        }
        serial_println!("[ok]");
    }
}
