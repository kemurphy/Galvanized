use opcode::*;
use libjit::*;
use analysis::*;
use std::vec;
use basic_block::*;
use variable_type::*;

/**
 * JIT compiles a function.
 * 
 * # Arguments
 *
 * * function - The function to compile.
 * * context - The JIT context within which to compile the function.
 */
pub fn compile(function: &[Opcode], context: &Context) -> ~Function {
    context.build_start();

    // TODO: at the moment, functions take no arguments and return a single float.
    let return_type = Types::get_float32();
    let params: &[&Type] = &[];
    let signature = Type::create_signature(CDECL, return_type, params);

    let jit_function = context.create_function(signature);

    // Convert stream of opcodes to basic block representation.
    let basic_blocks = get_basic_blocks(function);

    // Pre-create Values for each local variable.
    let mut locals = reserve_locals(function, basic_blocks, jit_function);

    for basic_block in basic_blocks.iter() {
        compile_basic_block(*basic_block, jit_function, &mut locals);
    }

    jit_function.compile();
    context.build_end();

    jit_function
}

/**
 * JIT compiles a single basic block.
 * 
 * # Arguments
 *
 * * basic_block - The basic block to compile.
 * * function    - The function that is being compiled.
 * * context     - The JIT context within which to compile the function.
 */
fn compile_basic_block(basic_block: @mut BasicBlock, 
                       function: &Function, 
                       locals: &mut ~[~Value]) {

    // The evaluation stack must be empty on entering a basic block.
    let mut stack = ~[];

    // Create a Label for this basic block.
    function.insn_set_label(basic_block.label);

    for opcode in basic_block.opcodes.iter() {
        compile_opcode(opcode, function, &mut stack, locals);
    }

    // If the basic block ends in a conditional branch (Iftrue),
    // emit a conditional branch JIT instruction here.
    match basic_block.conditional_block {
        Some(b) => {
            let value = stack.pop();
            function.insn_branch_if(value, b.label);
        }
    _   => { }
    }

    // If the basic block has a successor, emit a JIT branch
    // instruction. LibJIT (hopefully) optimises these out
    // if they are unnecessary.
    match basic_block.next_block {
        Some(b) => {
            function.insn_branch(b.label);
        }
    _   => { /* TODO must end in a Ret? */ }
    }
}

/**
 * JIT compiles a single opcode.
 * 
 * # Arguments
 *
 * * opcode   - The Opcode to compile.
 * * function - The JIT function object.
 * * stack    - The VM stack.
 * * locals   - The list of the function's local variable Values.
 */
fn compile_opcode(opcode: &Opcode,
                  function: &Function, 
                  stack: &mut ~[~Value], 
                  locals: &mut ~[~Value]) {

    match *opcode {
        Constf32(operand) => {
            stack.push(function.constant_float32(operand)); 
        }
        Consti32(operand) => {
            stack.push(function.constant_int32(operand));
        }
        Add => { 
            do binary_opcode(stack) |v1, v2| { function.insn_add(v1, v2) };
        }
        Subtract => { 
            do binary_opcode(stack) |v1, v2| { function.insn_sub(v1, v2) };
        }
        Multiply => { 
            do binary_opcode(stack) |v1, v2| { function.insn_mul(v1, v2) };
        }
        Divide => { 
            do binary_opcode(stack) |v1, v2| { function.insn_div(v1, v2) };
        }
        And => { 
            do binary_opcode(stack) |v1, v2| { function.insn_and(v1, v2) };
        }
        Or => { 
            do binary_opcode(stack) |v1, v2| { function.insn_or(v1, v2) };
        }
        Xor => { 
            do binary_opcode(stack) |v1, v2| { function.insn_xor(v1, v2) };
        }
        Eq => { 
            do binary_opcode(stack) |v1, v2| { function.insn_eq(v1, v2) };
        }
        Neq => { 
            do binary_opcode(stack) |v1, v2| { function.insn_neq(v1, v2) };
        }
        Leq => { 
            do binary_opcode(stack) |v1, v2| { function.insn_leq(v1, v2) };
        }
        Geq => { 
            do binary_opcode(stack) |v1, v2| { function.insn_geq(v1, v2) };
        }
        Lt => { 
            do binary_opcode(stack) |v1, v2| { function.insn_lt(v1, v2) };
        }
        Gt => { 
            do binary_opcode(stack) |v1, v2| { function.insn_gt(v1, v2) };
        }
        Negate => { 
            do unary_opcode(stack) |value| { function.insn_neg(value) };
        }
        Not => { 
            do unary_opcode(stack) |value| { function.insn_not(value) };
        }
        Ret => { 
            let v = stack.pop();
            function.insn_return(v);
        },
        Disp => {} // TODO
        Store(addr) => {
            let v = stack.pop();
            function.insn_store(locals[addr], v);
        }
        Loadf32(addr) => {
            let v = locals[addr].clone();
            let new_value = function.insn_dup(v);
            stack.push(new_value);
        }
        Loadi32(addr) => {
            let v = locals[addr].clone();
            let new_value = function.insn_dup(v);
            stack.push(new_value);
        }
        _ => { }
    }
}

/**
 * Pre-creates some JIT Values for use as the local variables in a function.
 *
 * * function     - The function to pre-create JIT Values for.
 * * basic_blocks - The basic block representation of the function.
 * * jit_function - The JIT function object to create values for.
 *
 * Returns the list of pre-created local variable Values.
 */
fn reserve_locals(function: &[Opcode], basic_blocks: &[@mut BasicBlock], jit_function: &Function) -> ~[~Value] {
    // TODO: count from basic blocks instead.
    let local_count = local_count(function);

    let types = infer_local_types(basic_blocks, local_count);

    let locals: ~[~Value] = do vec::from_fn(local_count as uint) |index| {
        jit_function.create_value( if types[index] == Float32 { Types::get_float32() } else { Types::get_int() })
    };
    return locals;
}

/**
 * Helper function for a binary opcode.
 *
 * Pops 2 Values from the stack and pushes the resulting Value.
 *
 * # Arguments
 *
 * * stack - The VM stack.
 * * f     - A function that takes 2 Values from the stack and returns a result Value.
 */
fn binary_opcode(stack: &mut ~[~Value], f: &fn(v1: &Value, v2: &Value) -> ~Value) {
    let v1 = stack.pop();
    let v2 = stack.pop();
    stack.push(f(v2, v1));
}

/**
 * Helper function for a binary opcode.
 *
 * Pops 2 Values from the stack and pushes the resulting Value.
 *
 * # Arguments
 *
 * * stack - The VM stack.
 * * f     - A function that takes 2 Values from the stack and returns a result Value.
 */
fn unary_opcode(stack: &mut ~[~Value], f: &fn(value: &Value) -> ~Value) {
    let value = stack.pop();
    stack.push(f(value));
}
