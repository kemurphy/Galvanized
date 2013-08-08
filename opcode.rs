
pub enum Opcode {
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