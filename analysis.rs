use opcode::*;
use std::num::*;

/**
 * Returns the number of local variables in the function.
 *
 * # Arguments
 *
 * * function - The function.
 */
pub fn local_count(function: &[Opcode]) -> u32 {
    let mut count = 0u32;
    for opcode in function.iter() {
        match *opcode {
            Loadi32(n) | Loadf32(n) => count = max(count, n),
            Store(n) => count = max(count, n),
            _ => ()
        }
    }
    return count + 1;
}
