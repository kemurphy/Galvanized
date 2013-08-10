use opcode::*;
use std::trie::*;
use std::ptr::*;
use libjit::*;
use std::to_bytes::*;
use std::hash::*;

/**
 * Represents a basic block.
 * http://en.wikipedia.org/wiki/Basic_block
 */
#[Deriving(Hash)]
struct BasicBlock {
    /// Basic blocks that control can flow to this one from.
    prev_blocks: ~[@mut BasicBlock],

    /// The next basic block in the control flow. This is either
    /// because it starts with the next instruction, is the target
    /// or an unconditional branch, or is the fall-through for
    /// a conditional branch.
    next_block: Option<@mut BasicBlock>,

    /// The target basic block for a condiditional (Iftrue) branch.
    conditional_block: Option<@mut BasicBlock>,

    /// The instructions within the basic block.
    opcodes: ~[Opcode],

    /// The JIT Label that marks the start of this basic block.
    label: ~Label
}

impl IterBytes for BasicBlock {
    pub fn iter_bytes(&self, lsb0: bool, f: Cb) -> bool {
        (to_unsafe_ptr(self) as u64).iter_bytes(lsb0, f)
    }
}

impl Eq for BasicBlock {
    pub fn eq(&self, other: &BasicBlock) -> bool {
        return self.hash() == other.hash();
    }
}

impl BasicBlock {
    /**
     * Creates a new BasicBlock.
     */
    pub fn new() -> @mut BasicBlock {
        @mut BasicBlock {
            prev_blocks: ~[],
            conditional_block: None,
            next_block: None,
            opcodes: ~[],
            label: Label::new()
        }
    }

    /**
     * Pushes an Opcode to the end of the basic block.
     */
    pub fn push_opcode(&mut self, opcode: Opcode) {
        self.opcodes.push(opcode);
    }

    /**
     * Prints a basic block for diagnostic purposes.
     */
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

/**
 * Computes the basic blocks for a stream of Opcodes.
 * 
 * # Arguments
 *
 * * function - The function to compute a basic block representation of.
 *
 * Returns a list of basic blocks.
 */
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
                if (index != 0) {
                    let temp = basic_blocks_map.find(&index);
                    match temp {
                        Some(b) => {
                            match current_block.next_block {
                                None => {
                                    current_block.next_block = Some(*b);
                                    b.prev_blocks.push(current_block);
                                }
                                _ => { }
                            }
                            current_block = *b; 
                        }
                        _ => { }
                    }  
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

/**
 * Prints a list of basic blocks for diagnostic purposes.
 * @type {[type]}
 */
pub fn print_basic_blocks(basic_blocks: &[@mut BasicBlock]) {
    for basic_block in basic_blocks.iter() {
        basic_block.print();
    }    
}
