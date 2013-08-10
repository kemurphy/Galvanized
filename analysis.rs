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

pub fn infer_local_types(basic_blocks: &[@mut BasicBlock], local_count: u32) -> ~[VariableType] {
    // TODO: get local count in this function.
    let mut locals = vec::from_elem(local_count as uint, Unknown);
    let mut changed = true;

    while changed {
        changed = false;
        for basic_block in basic_blocks.iter() {
            changed |= infer_local_types_for_basic_block(*basic_block, locals);
        }
    }

    return locals;
}

fn infer_local_types_for_basic_block(basic_block: &mut BasicBlock, locals: &mut [VariableType]) -> bool {
    let mut changed = false;
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
                    if locals[addr] != Unknown && locals[addr] != t {
                        println(fmt!("Warning... local(%?) has conflicting types", addr));
                    } else if locals[addr] != t {
                        locals[addr] = t;
                        changed = true;
                    }
                }
            }
            Loadi32(addr) | Loadf32(addr) => {
                stack.push(locals[addr]);
            }
            _ => { }
        }
    }
    return changed;
}