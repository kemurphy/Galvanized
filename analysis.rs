use opcode::*;
use std::num::*;
use std::vec;
use variable_type::*;
use basic_block::*;

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

/**
 * Infers the types of local variables in the function.
 *
 * # Arguments
 *
 * * basic_blocks - The basic blocks that comprise the function.
 * * local_count  - The number of local variables in the function.
 *
 * Returns a list of VariableTypes representing the inferred types of the locals.
 */
pub fn infer_local_types(basic_blocks: &[@mut BasicBlock], local_count: u32) -> ~[VariableType] {
    // TODO: get local count in this function.
    let mut locals = vec::from_elem(local_count as uint, Unknown);
    let mut changed = true;

    // Repeat over the entire function until we can infer no more information.
    while changed {
        changed = false;
        for basic_block in basic_blocks.iter() {
            changed |= infer_local_types_for_basic_block(*basic_block, locals);
        }
    }

    return locals;
}

/**
 * Infers the types of local variables in a single basic block.
 *
 * # Arguments
 *
 * * basic_block - The basic blocks within which to infer local types.
 * * local_types - The current state of inferred local types.
 *
 * Returns a boolean indicating whether any additional type information was inferred.
 */
fn infer_local_types_for_basic_block(basic_block: &mut BasicBlock, local_types: &mut [VariableType]) -> bool {
    let mut changed = false;

    // The evaluation stack should be empty on entering a basic block.
    let mut stack = ~[];

    for opcode in basic_block.opcodes.iter() {
        match *opcode {
            Constf32(_) => {
                stack.push(Float32); 
            }
            Consti32(_) => {
                stack.push(Int32);
            }
            Add | Subtract | Multiply | Divide => { 
                let t = stack.pop();
                let t2 = stack.pop();
                if t != t2 && t != Unknown && t2 != Unknown {
                    // TODO: may cause problems if one is Unknown
                    println(fmt!("Warning... conflicting types... taking as %?", t));
                }
                stack.push(t);
            }
            And | Or | Xor | Eq | Neq | Leq | Geq | Lt | Gt => { 
                // TODO: warning if float
                stack.pop();
                stack.pop();
                stack.push(Int32);
            }
            Negate => { /* pop & push same type */ }
            Not => { 
                stack.pop();
                // TODO: warning if float
                stack.push(Int32);
            }
            Store(addr) => {
                let t = stack.pop();
                if t != Unknown {
                    if local_types[addr] != Unknown && local_types[addr] != t {
                        println(fmt!("Warning... local(%?) has conflicting types", addr));
                    } else if local_types[addr] != t {
                        local_types[addr] = t;
                        changed = true;
                    }
                }
            }
            Loadi32(addr) | Loadf32(addr) => {
                stack.push(local_types[addr]);
            }
            _ => { }
        }
    }
    
    changed
}
