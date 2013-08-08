use opcode::*;
use std::num::*;

pub fn local_count(program: &[Opcode]) -> u32 {
    let mut count = 0u32;
    for opcode in program.iter() {
        match *opcode {
            Load(n) => count = max(count, n),
            Store(n) => count = max(count, n),
            _ => ()
        }
    }
    return count + 1;
}