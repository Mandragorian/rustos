pub struct Block {
    next: Option<&'static mut Block>,
    size: usize,
}

impl Block {
    unsafe fn ref_from_usize(addr: usize) -> &'static mut Block {
        let block = addr as *mut Block;
        unsafe { &mut *block }
    }

    pub fn size(&mut self, size: usize) {
        self.size = size;
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
    fn new() -> BlockList {
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
            let block_ref = unsafe { Block::ref_from_usize(addr) };
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
