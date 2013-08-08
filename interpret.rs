use opcode::*;
use analysis::*;


pub struct Environment {
    bp: u32,
    ip: u32,
    end: u32
}



pub fn interpret(program: &[Opcode]) {
    let stack = &mut ~[];
    let environment = &mut Environment { bp: 0, ip: 0, end: program.len() as u32 };

    let local_count = local_count(program);
    environment.bp += local_count;
    stack.grow(local_count as uint, &0f32);

    while (environment.ip as uint) < program.len() {
        environment.ip = interpret_opcode(&program[environment.ip], stack, environment);
    }
}

fn interpret_opcode(opcode: &Opcode, stack: &mut ~[f32], environment: &mut Environment) -> u32 {
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
        Ret             => {
            println(fmt!("Returned: %?", stack.pop()));
            return environment.end;
        }
        Disp            => println(fmt!("%?", stack.pop())),
        Store(addr) => {
            stack[environment.bp - addr - 1] = stack.pop();
        }
        Load(addr) => {
            stack.push(stack[environment.bp - addr - 1]);
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
        Nop => { }
    }
    return environment.ip + 1;
}