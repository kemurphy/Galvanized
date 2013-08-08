
pub enum Opcode {
    Nop,
    Add,
    Subtract,
    Multiply,
    Divide,
    Constf(f32),
    Ret,
    Disp,
    Store(u32),
    Load(u32),
    Jmp(u32),
    Ifleq(u32)
}