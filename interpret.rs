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
        Constf32(operand) => {
            stack.push(operand);
        }
        Consti32(operand) => {
            stack.push(operand as f32);
        }
        Add => {
            do binary_opcode(stack) |v1, v2| { v1 + v2 };
        }
        Subtract => {
            do binary_opcode(stack) |v1, v2| { v1 - v2 };
        }
        Multiply => {
            do binary_opcode(stack) |v1, v2| { v1 * v2 };
        }
        Divide => {
            do binary_opcode(stack) |v1, v2| { v1 / v2 };
        }
        And => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) & (v2 as uint)) as f32 };
        }
        Or => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) | (v2 as uint)) as f32 };
        }
        Xor => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) ^ (v2 as uint)) as f32 };
        }
        Eq => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) == (v2 as uint)) as f32 };
        }
        Neq => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) != (v2 as uint)) as f32 };
        }
        Leq => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) <= (v2 as uint)) as f32 };
        }
        Geq => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) >= (v2 as uint)) as f32 };
        }
        Lt => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) < (v2 as uint)) as f32 };
        }
        Gt => {
            do binary_opcode(stack) |v1, v2| { ((v1 as uint) > (v2 as uint)) as f32 };
        }
        Negate => {
            do unary_opcode(stack) |value| { -(value as uint) as f32 };
        }
        Not => {
            do unary_opcode(stack) |value| { !(value as uint) as f32 };
        }
        Ret => {
            println(fmt!("Returned: %?", stack.pop()));
            return environment.end_ip;
        }
        Disp => println(fmt!("%?", stack.pop())),
        Store(addr) => {
            stack[environment.bp - addr - 1] = stack.pop();
        }
        Loadf32(addr) => {
            stack.push(stack[environment.bp - addr - 1]);
        }
        Loadi32(addr) => {
            stack.push(stack[environment.bp - addr - 1]);
        }
        Jmp(n) => {
            return n;
        }
        Iftrue(n) => {
            return do conditional_branch(stack, n, environment) |value| { (value as bool) };
        }
        Iffalse(n) => {
            return do conditional_branch(stack, n, environment) |value| { !(value as bool) };
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
                      f: &fn(value: f32) -> bool)
                      -> u32 {

    let value = stack.pop();
    if f(value) {
        target_address
    } else {
        environment.ip + 1
    }
}

/**
 * Helper function for a binary opcode that pops two values from the stack and pushes the result.
 *
 * # Arguments
 *
 * * stack          - The VM runtime stack.
 * * f              - A function that takes two values from the stack and returns a result value.
 */
fn binary_opcode(stack: &mut ~[f32],
                 f: &fn(v1: f32, v2: f32) -> f32) {

    let v1 = stack.pop();
    let v2 = stack.pop();
    stack.push(f(v2, v1));
}

/**
 * Helper function for a unary opcode that pops a value from the stack and pushes the result.
 *
 * # Arguments
 *
 * * stack          - The VM runtime stack.
 * * f              - A function that takes a value from the stack and returns a result value.
 */
fn unary_opcode(stack: &mut ~[f32],
                 f: &fn(value: f32) -> f32) {

    let value = stack.pop();
    stack.push(f(value));
}
