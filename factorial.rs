use std::libc::*;
use opcode::*;
use interpret::*;
use libjit::*;
use jit::*;

mod libjit;
mod opcode;
mod interpret;
mod jit;
mod analysis;

fn main() {

    // Computes the factorial of 10.
    let factorial = ~[
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
        Ifleq(16),          // ------   |
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
        Jmp(4),             // -----+----
                            //      |
        // return f         //      |
        Load(1),            // <-----
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
    let retval: @mut f32 = @mut 15.0;
    function.apply(args, retval);

    println("Returned:");
    println(fmt!("%?", *retval));

    println("");
    println("Closure factorial(10)...");

    let f: extern "C" unsafe fn() -> c_float = function.closure();
    let ret = unsafe {
        f()
    };
    
    println(fmt!("%?", ret));
}
