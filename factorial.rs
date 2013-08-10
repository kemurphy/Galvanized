use std::libc::*;
use opcode::*;
use interpret::*;
use libjit::*;
use jit::*;

mod variable_type;
mod libjit;
mod opcode;
mod interpret;
mod jit;
mod basic_block;
mod analysis;

fn main() {

    // Sample VM function that omputes the factorial of 10.
    let factorial = ~[
        // n := 10          //
        Constf32(10f32),    // 
        Store(0),           // 
                            //
        // f := 1           //
        Constf32(1f32),     //
        Store(1),           // 
                            //
        // if n <= 1 go end //
        Loadf32(0),            // <---------
        Constf32(1f32),     //          |
        Leq,                //          |
        Iftrue(17),         // ------   |
                            //      |   |
        // f := n * f       //      |   |
        Loadf32(0),            //      |   |
        Loadf32(1),            //      |   |
        Multiply,           //      |   |
        Store(1),           //      |   |
                            //      |   |
        // n := n - 1       //      |   |
        Loadf32(0),            //      |   |
        Constf32(1f32),     //      |   |
        Subtract,           //      |   |
        Store(0),           //      |   |
                            //      |   |
        // loop             //      |   |
        Jmp(4),             // -----+----
                            //      |
        // return f         //      |
        Loadf32(1),            // <-----
        Ret                 //
    ];

    println("Interpreting factorial(10)...");
    interpret(factorial);

    println("");
    println("Jitting factorial(10)...");
    
    let context = Context::new();
    let function = compile(factorial, context);

    function.dump("factorial");
    println("");

    let args: ~[*c_void] = ~[];
    let mut retval: ~i32 = ~0;
    function.apply(args, retval);

    println("Returned:");
    println(fmt!("%?", *retval));

    println("");
    println("Closure factorial(10)...");

    let f: extern "C" fn() -> c_int = function.closure();
    let ret = f();

    println(fmt!("%?", ret));
}
