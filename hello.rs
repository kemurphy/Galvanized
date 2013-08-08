use std::libc::*;

mod jit;

enum Opcode {
    Add,
    Subtract,
    Multiply,
    Divide,
    Constf(f32),
    Ret,
    Disp,
    Store(u32),
    Load(u32),
    Reserve(u32),
    Jmp(i32),
    Ifleq(i32)
}

struct Environment {
    bp: u32,
    ip: i32
}

fn main() {

    /*// Computes the factorial of 10.
    let factorial = ~[
        // Reserve space    //
        // on stack for     //
        // locals n & f     //
        Reserve(2),         //
                            //
        // n := 10          //
        Constf(10f32),      // 
        Store(0),           // 
                            //
        // f := 1           //
        Constf(1f32),       //
        Store(1),           // 
                            //
        // if f <= 1 go end //
        Load(0),            // <---------
        Constf(1f32),       //          |
        Ifleq(10),          // ------   |
                            //      |   |
        // f := n * f       //      |   |
        Load(0),            //      |   |
        Load(1),            //      |   |
        Multiply,           //      |   |
        Store(1),           //      |   |
                            //      |   |
        // n := f - 1       //      |   |
        Load(0),            //      |   |
        Constf(1f32),       //      |   |
        Subtract,           //      |   |
        Store(0),           //      |   |
                            //      |   |
        // loop             //      |   |
        Jmp(-11),           // -----+----
                            //      |
        // return f         //      |
        Load(1),            // <-----
        Ret                 //
    ];*/

    // Computes the factorial of 10.
    let factorial = ~[
        Reserve(2),
        Constf(100f32),
        Constf(102.22),
        Multiply,
        Store(0),
        Constf(100f32),
        Constf(102.22),
        Add,
        Store(1),
        Load(0),
        Load(1),
        Subtract,
        Load(1),
        Load(1),
        Add,
        Multiply,
        Ret
    ];

    println("Interpreting factorial...");
    interpret(factorial);

    println("");
    println("Jitting factorial...");
    
    let context = jit::Context::new();
    let function = compile(factorial, context);

    let args: ~[*c_void] = ~[];
    let retval: @mut f32 = @mut 15.0;
    function.apply(args, retval);
    println(fmt!("%?", *retval));
}

fn compile(program: &[Opcode], context: &jit::Context) -> ~jit::Function {
    context.build_start();

    let return_type = jit::Types::get_float32();
    let params: &[&jit::Type] = &[];

    let signature = jit::Type::create_signature(jit::CDECL, return_type, params);

    let function = context.create_function(signature);

    let mut stack = ~[];
    let stack = &mut stack;
    let environment = &mut Environment { bp: 0, ip: 0 };

    for opcode in program.iter() {
        compile_opcode(opcode, function, stack, environment);
    }

    function.compile();
    context.build_end();

    return function;
}

fn compile_opcode(opcode: &Opcode, function: &jit::Function, stack: &mut ~[Option<~jit::Value>], environment: &mut Environment) {
    match *opcode {
        Constf(operand)  => {
            stack.push(Some(function.constant_float32(operand))); 
        }
        Add             => {
            let v1 = stack.pop();
            let temp1 = v1.get_ref(); 
            let v2 = stack.pop();
            let temp2 = v2.get_ref();
            stack.push(Some(function.insn_add(*temp2, *temp1)));
        }
        Subtract        => {
            let v1 = stack.pop();
            let temp1 = v1.get_ref(); 
            let v2 = stack.pop();
            let temp2 = v2.get_ref();
            stack.push(Some(function.insn_sub(*temp2, *temp1)));
        }
        Multiply        => {
            let v1 = stack.pop();
            let temp1 = v1.get_ref(); 
            let v2 = stack.pop();
            let temp2 = v2.get_ref();
            stack.push(Some(function.insn_mul(*temp2, *temp1)));
        }
        Divide          => {
            let v1 = stack.pop();
            let temp1 = v1.get_ref(); 
            let v2 = stack.pop();
            let temp2 = v2.get_ref();
            stack.push(Some(function.insn_div(*temp2, *temp1)));
        }
        Ret             => { 
            let v = stack.pop();
            let temp = v.get_ref();
            function.insn_return(*temp) 
        },
        Disp            => {} //println(fmt!("%?", stack.pop()))
        Store(addr) => {
            let v = stack.pop();
            let temp = v.get_ref();
            stack[environment.bp - addr - 1] = Some(function.insn_dup(*temp));
        }
        Load(addr) => {
            //stack.push(stack[environment.bp - addr - 1]);
            /*stack.push(None);
            let len = stack.len();
            stack.swap((environment.bp - addr - 1) as uint, len - 1);*/
            let v = stack[environment.bp - addr - 1].clone();
            let temp = v.get_ref();
            let new_value = Some(function.insn_dup(*temp));
            stack.push(new_value);
        }
        Reserve(n) => {
            environment.bp += n;
            stack.grow(n as uint, ~None);
        }
        Jmp(n) => {

        }
        Ifleq(n) => {
            
        }
    }
}

fn interpret(program: &[Opcode]) {
    let stack = &mut ~[];
    let environment = &mut Environment { bp: 0, ip: 0 };
    while (environment.ip as uint) < program.len() {
        environment.ip += interpret_opcode(&program[environment.ip], stack, environment);
    }
}

fn interpret_opcode(opcode: &Opcode, stack: &mut ~[f32], environment: &mut Environment) -> i32 {
    match *opcode {
        Constf(operand)  => {
            stack.push(operand);
        }
        Add             => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 + v1);
        }
        Subtract        => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 - v1);
        }
        Multiply        => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 * v1);
        }
        Divide          => {
            let v1 = stack.pop(); 
            let v2 = stack.pop(); 
            stack.push(v2 / v1);
        }
        Ret             => println(fmt!("%?", stack.pop())),
        Disp            => println(fmt!("%?", stack.pop())),
        Store(addr) => {
            stack[environment.bp - addr - 1] = stack.pop();
        }
        Load(addr) => {
            stack.push(stack[environment.bp - addr - 1]);
        }
        Reserve(n) => {
            environment.bp += n;
            stack.grow(n as uint, &0f32);
        }
        Jmp(n) => {
            return n;
        }
        Ifleq(n) => {
            let v1 = stack.pop();
            let v2 = stack.pop();
            if v2 <= v1 {
                return n;
            }
        }
    }
    return 1;
}