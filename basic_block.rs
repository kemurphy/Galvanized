
use opcode::*;
use std::trie::*;
use std::ptr::*;

struct BasicBlock {
    prev_blocks: ~[@mut BasicBlock],
    next_block: Option<@mut BasicBlock>,
    conditional_block: Option<@mut BasicBlock>,
    opcodes: ~[Opcode]
}

impl BasicBlock {
    pub fn new() -> @mut BasicBlock {
        @mut BasicBlock {
            prev_blocks: ~[],
            conditional_block: None,
            next_block: None,
            opcodes: ~[]
        }
    }

    pub fn push_opcode(&mut self, opcode: Opcode) {
        self.opcodes.push(opcode);
    }

    pub fn print(&self) {
        println(fmt!("BasicBlock: 0x%X", to_unsafe_ptr(self) as uint));
        
        println("prev_blocks:");
        for block in self.prev_blocks.iter() {
            println(fmt!("  0x%X", to_unsafe_ptr(*block) as uint));
        }
        
        print("next_block: ");
        match self.next_block {
            None => { println("none"); }
            Some(b) => { println(fmt!("0x%X", to_unsafe_ptr(b) as uint)); }
        }
        
        print("conditional_block: ");
        match self.conditional_block {
            None => { println("none"); }
            Some(b) => { println(fmt!("0x%X", to_unsafe_ptr(b) as uint)); }   
        }

        println("opcodes");
        for opcode in self.opcodes.iter() {
            println(fmt!("  %?", opcode));
        }

        println("");
    }
}


pub fn get_basic_blocks(function: &[Opcode]) -> ~[@mut BasicBlock] {
    let mut basic_blocks_map = TrieMap::new();
    basic_blocks_map.insert(0u, BasicBlock::new());

    for (index, opcode) in range(0, function.len()).zip(function.iter()) {
        match *opcode {
            Jmp(n) | Iftrue(n) => {
                match basic_blocks_map.find(&(n as uint)) {
                    None => {
                        basic_blocks_map.insert(n as uint, BasicBlock::new());
                    }
                    _ => { }
                }
                basic_blocks_map.insert(index + 1, BasicBlock::new());
            }
            _ => { }
        }
    }    

    // r-values lifetime bug again.
    let temp = basic_blocks_map.find(&0);
    let mut current_block: @mut BasicBlock = **temp.get_ref();

    // TODO: the types here are an absolute mess
    for (index, opcode) in range(0, function.len()).zip(function.iter()) {
        match *opcode {
            Jmp(n) => {
                let next_block = basic_blocks_map.find(&(n as uint));
                next_block.get_ref().prev_blocks.push(current_block);
                current_block.next_block = Some(**next_block.get_ref());
                current_block = **next_block.get_ref();
            }
            Iftrue(n) => {
                let conditional_block = basic_blocks_map.find(&(n as uint));
                conditional_block.get_ref().prev_blocks.push(current_block);
                current_block.conditional_block = Some(**conditional_block.get_ref());

                let next_block = basic_blocks_map.find(&(index + 1));
                current_block.next_block = Some(**next_block.get_ref());
                current_block = **next_block.get_ref();
            }
            _ => { 
                let temp = basic_blocks_map.find(&index);
                match temp {
                    Some(b) => { 
                        current_block.next_block = Some(*b);
                        b.prev_blocks.push(current_block);
                        current_block = *b; 
                    }
                    _ => { }
                }  
                current_block.push_opcode(*opcode); 
            }
        }
    }

    let mut basic_blocks: ~[@mut BasicBlock] = ~[];
    basic_blocks_map.each_value(|v| {
        basic_blocks.push(*v);
        true
    });

    return basic_blocks;
}

pub fn print_basic_blocks(basic_blocks: ~[@mut BasicBlock]) {
    for basic_block in basic_blocks.iter() {
        basic_block.print();
    }    
}