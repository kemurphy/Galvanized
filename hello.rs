use std::libc::*;

mod jit;

enum Opcode {
    Add,
    Subtract,
    Multiply,
    Divide,
    Pushf(f32),
    Ret,
    Disp
}

fn main() {
    let mut program = ~[
        Pushf(12.3),
        Pushf(13.8),
        Add
    ];

    program.push(Ret);

    interpret(program);
    
    let context = jit::Context::new();
    let function = compile(program, context);

    let args: ~[*c_void] = ~[];
    let mut retval: f32 = 15.0;
    function.apply(args, &mut retval);
    println(fmt!("GOT %?", retval));
}

fn compile(program: &[Opcode], context: &jit::Context) -> ~jit::Function {
    context.build_start();

    let return_type = jit::Types::get_float32();
    let params: ~[~jit::Type] = ~[];

    let signature = jit::Type::create_signature(jit::CDECL, return_type, params);

    let function = context.create_function(signature);

    let stack = &mut ~[];

    for program.iter().advance |opcode| {
        compile_opcode(opcode, function, stack);
    }

    function.compile();
    context.build_end();

    return function;
}

fn compile_opcode(opcode: &Opcode, function: &jit::Function, stack: &mut ~[~jit::Value]) {
    match *opcode {
        Pushf(ref operand)  => {
            stack.push(function.constant_float32(*operand)); 
        },
        Add             => {
            let v1 = stack.pop(); 
            let v2 = stack.pop();
            stack.push(function.insn_add(v1, v2));
        },
        Subtract        => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(function.insn_add(v1, v2));
        },
        Multiply        => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(function.insn_add(v1, v2));
        },
        Divide          => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(function.insn_add(v1, v2));
        },
        Ret             => { function.insn_return(stack.pop()) },
        Disp            => {} //println(fmt!("%?", stack.pop()))
    }
}

fn interpret(program: &[Opcode]) {
    let stack = &mut ~[];
    for program.iter().advance |opcode| {
        interpret_opcode(opcode, stack);
    }
}

fn interpret_opcode(opcode: &Opcode, stack: &mut ~[f32]) {
    match *opcode {
        Pushf(ref operand)  => {
            stack.push(*operand); 
        },
        Add             => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v1 + v2);
        },
        Subtract        => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v1 - v2);
        },
        Multiply        => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v1 * v2);
        },
        Divide          => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v1 / v2);
        },
        Ret             => {},
        Disp            => println(fmt!("%?", stack.pop()))
    }
}