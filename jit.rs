use opcode::*;
use libjit::*;
use analysis::*;
use std::vec;

/**
 * Represents a single opcode with additional annotations.
 */
struct AnnotatedOpcode {
    /// The instruction opcode.
    opcode: Opcode,
    /// Index of instruction that targets this one.
    jmp_from: Option<u32>,
    /// Index of instruction that this one targets.
    jmp_to: Option<u32>,
    /// Label representing the instruction.
    label: ~Label,
}

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

    let mut stack = ~[];
    let mut locals = reserve_locals(function, jit_function);

    let mut annotated_function = annotate_labels(function);

    let mut index = 0;
    while index < annotated_function.len() {
        compile_opcode(&mut annotated_function, index, jit_function, &mut stack, &mut locals);      
        index += 1;
    }

    jit_function.compile();
    context.build_end();

    jit_function
}

/**
 * JIT compiles a single opcode.
 * 
 * # Arguments
 *
 * * annotated_function - The annotated function.
 * * index             - The index of the opcode in the annotated program to compile.
 * * function          - The JIT function object.
 * * stack             - The VM stack.
 * * locals            - The list of the function's local variable Values.
 */
fn compile_opcode(annotated_function: &mut ~[~AnnotatedOpcode], 
                  index: uint, 
                  function: &Function, 
                  stack: &mut ~[~Value], 
                  locals: &mut ~[~Value]) {

    // If this instruction is the target of a branch, create a label.
    match annotated_function[index].jmp_from {
        Some(_) => {
            function.insn_set_label(annotated_function[index].label);
        }
        _ => ()
    }

    let opcode = annotated_function[index].opcode;
    match opcode {
        Constf(operand) => {
            stack.push(function.constant_float32(operand)); 
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
        Ret => { 
            let v = stack.pop();
            function.insn_return(v);
        },
        Disp => {} // TODO
        Store(addr) => {
            let v = stack.pop();
            function.insn_store(locals[addr], v);
        }
        Load(addr) => {
            let v = locals[addr].clone();
            let new_value = function.insn_dup(v);
            stack.push(new_value);
        }
        Jmp(n) => {
            function.insn_branch(annotated_function[n].label);
        }
        Ifleq(n) => {
            do conditional_branch(annotated_function, stack, n, function) |v1, v2| { function.insn_leq(v1, v2) };
        }
        Ifgeq(n) => {
            do conditional_branch(annotated_function, stack, n, function) |v1, v2| { function.insn_geq(v1, v2) };
        }
        Iflt(n) => {
            do conditional_branch(annotated_function, stack, n, function) |v1, v2| { function.insn_lt(v1, v2) };
        }
        Ifgt(n) => {
            do conditional_branch(annotated_function, stack, n, function) |v1, v2| { function.insn_gt(v1, v2) };
        }
        Ifeq(n) => {
            do conditional_branch(annotated_function, stack, n, function) |v1, v2| { function.insn_eq(v1, v2) };
        }
        Ifneq(n) => {
            do conditional_branch(annotated_function, stack, n, function) |v1, v2| { function.insn_neq(v1, v2) };
        }
        Nop => { }
    }
}

/**
 * Pre-creates some JIT Values for use as the local variables in a function.
 *
 * * function     - The function to pre-create JIT Values for.
 * * jit_function - The JIT function object to create values for.
 *
 * Returns the list of pre-created local variable Values.
 */
fn reserve_locals(function: &[Opcode], jit_function: &Function) -> ~[~Value] {
    let local_count = local_count(function) as uint;
    let locals: ~[~Value] = do vec::from_fn(local_count) |_| {
        jit_function.create_value(Types::get_float32())
    };
    return locals;
}

/**
 * Annotates a function with labels representing the start of basic blocks.
 *
 * # Arguments
 * 
 * * function - The function to annotate.
 *
 * Returns the annotated function.
 */
fn annotate_labels(function: &[Opcode]) -> ~[~AnnotatedOpcode] {
    let mut annotated_function: ~[~AnnotatedOpcode] = vec::from_fn(function.len(), |i| {
        ~AnnotatedOpcode { opcode: function[i], jmp_to: None, jmp_from: None, label: Label::new()  }
    });

    let mut index = 0;
    for opcode in function.iter() {
        match *opcode {
            Jmp(n) => {
                annotated_function[index].jmp_to = Some(n);
                annotated_function[n].jmp_from = Some(index);
            }
            Ifleq(n) => {
                annotated_function[index].jmp_to = Some(n);
                annotated_function[n].jmp_from = Some(index);
            }
            _ => ()
        }
        index += 1;
    }
    return annotated_function;
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
 * Helper function for a conditional branch opcode.
 *
 * Pops 2 Values from the stack and branches if they meet the specified condition.
 *
 * # Arguments
 *
 * * annotated_function - The annotated function.
 * * stack              - The VM stack.
 * * target_address     - The address to branch to if the condition is satisfied.
 * * function           - The JIT function object.
 * * f                  - A function that takes two Values from the stack and returns a result Value.
 */
fn conditional_branch(annotated_function: &mut ~[~AnnotatedOpcode], 
                      stack: &mut ~[~Value],
                      target_address: u32,
                      function: &Function,
                      f: &fn(v1: &Value, v2: &Value) -> ~Value) {

    let v1 = stack.pop();
    let v2 = stack.pop();
    let temp_result = f(v2, v1);
    function.insn_branch_if(temp_result, annotated_function[target_address].label);
}
