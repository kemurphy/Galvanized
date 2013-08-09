/**
 * The VM instruction opcodes.
 */
pub enum Opcode {
    /// No operation
    Nop,
    
    /// Binary opcodes - pop 2 values from
    /// the stack and push the result.
    Add,
    Subtract,
    Multiply,
    Divide,
    And,
    Or,
    Xor,
    Eq,
    Neq,
    Leq,
    Geq,
    Lt,
    Gt,

    /// Unary opcodes -- pop a value from
    /// the stack and push the result.
    Negate,
    Not,

    /// Load constant opcodes - pushes the
    /// specified constant value on the stack.
    Constf32(f32),
    Consti32(i32),

    /// Pops a value from the stack and returns it.
    Ret,

    /// Pops a value from the stack and displays it.
    Disp,

    /// Pops a value from the stack and stores it
    /// in the specified local variable location.
    Store(u32),

    /// Loads a value from the specified local 
    /// variable location and pushes it on the stack.
    Loadf32(u32),
    Loadi32(u32),

    /// Jumps to the instruction at the 
    /// specified address.
    Jmp(u32),

    /// Conditional branches - pops 2 values from
    /// the stack and jumps to the specified 
    /// address if the condition is satisfied.
    Iftrue(u32),
    Iffalse(u32)
}