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

    /*// Computes the factorial of 10.
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
    
    let context = Context::new();
    let function = compile(factorial, context);

    let args: ~[*c_void] = ~[];
    let retval: @mut f32 = @mut 15.0;
    function.apply(args, retval);
    println(fmt!("%?", *retval));
}
