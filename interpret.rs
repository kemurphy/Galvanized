use opcode::*;
use analysis::*;

/**
 * Represents the runtime environment of the VM.
 */
pub struct Environment {
    /// Stack base pointer.
    bp: u32,
    /// Instruction pointer.
    ip: u32,
    /// Points just past the last instruction.
    end_ip: u32
}

/**
 * Interprets a function.
 *
 * # Arguments
 *
 * * function - The function to interpret.
 */
pub fn interpret(function: &[Opcode]) {
    let stack = &mut ~[];
    let environment = &mut Environment { bp: 0, ip: 0, end_ip: function.len() as u32 };

    // Allocate room on the stack for locals.
    let local_count = local_count(function);
    environment.bp += local_count;
    stack.grow(local_count as uint, &0f32);

    while (environment.ip as uint) < function.len() {
        environment.ip = interpret_opcode(&function[environment.ip], stack, environment);
    }
}

/**
 * Interprets a single opcode.
 *
 * # Arguments
 *
 * * opcode      - The opcode to interpret.
 * * stack       - The VM runtime stack.
 * * environment - The current runtime environment state of the VM.
 *
 * Returns the next value of the instruction pointer.
 */
fn interpret_opcode(opcode: &Opcode, stack: &mut ~[f32], environment: &mut Environment) -> u32 {
    match *opcode {
        Constf(operand) => {
            stack.push(operand);
        }
        Add => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 + v1);
        }
        Subtract => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 - v1);
        }
        Multiply => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 * v1);
        }
        Divide => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 / v1);
        }
        Ret => {
            println(fmt!("Returned: %?", stack.pop()));
            return environment.end_ip;
        }
        Disp => println(fmt!("%?", stack.pop())),
        Store(addr) => {
            stack[environment.bp - addr - 1] = stack.pop();
        }
        Load(addr) => {
            stack.push(stack[environment.bp - addr - 1]);
        }
        Jmp(n) => {
            return n;
        }
        Ifleq(n) => {
            return conditional_branch(stack, n, environment, |v1, v2| { v2 <= v1 } );
        }
        Ifgeq(n) => {
            return conditional_branch(stack, n, environment, |v1, v2| { v2 >= v1 } );
        }
        Iflt(n) => {
            return conditional_branch(stack, n, environment, |v1, v2| { v2 < v1 } );
        }
        Ifgt(n) => {
            return conditional_branch(stack, n, environment, |v1, v2| { v2 > v1 } );
        }
        Ifeq(n) => {
            return conditional_branch(stack, n, environment, |v1, v2| { v2 == v1 } );
        }
        Ifneq(n) => {
            return conditional_branch(stack, n, environment, |v1, v2| { v2 != v1 } );
        }
        Nop => { }
    }
    
    environment.ip + 1
}

/**
 * Helper function for a conditional branch opcode.
 *
 * # Arguments
 *
 * * stack          - The VM runtime stack.
 * * target_address - The address to branch to if the specified condition is true.
 * * environment    - The current runtime environment state of the VM.
 * * f              - A function that takes two values from the stack and returns a bool.
 *
 * Returns the new value of the instruction pointer.
 */
fn conditional_branch(stack: &mut ~[f32], 
                      target_address: u32, 
                      environment: &mut Environment,
                      f: &fn(v1: f32, v2: f32) -> bool) -> u32 {
    let v1 = stack.pop();
    let v2 = stack.pop();
    return if f(v2, v1) {
        target_address
    } else {
        environment.ip + 1
    }
}
