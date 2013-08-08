use opcode::*;
use interpret::*;
use libjit::*;
use analysis::*;

mod opcode;
mod interpret;
mod libjit;


pub fn compile(program: &[Opcode], context: &Context) -> ~Function {
    context.build_start();

    let return_type = Types::get_float32();
    let params: &[&Type] = &[];

    let signature = Type::create_signature(CDECL, return_type, params);

    let function = context.create_function(signature);

    let mut stack = ~[];
    let stack = &mut stack;
    let environment = &mut Environment { bp: 0, ip: 0 };

    let local_count = local_count(program);
    environment.bp += local_count;
    stack.grow(local_count as uint, ~None);

    for opcode in program.iter() {
        compile_opcode(opcode, function, stack, environment);
    }

    function.compile();
    context.build_end();

    return function;
}

fn compile_opcode(opcode: &Opcode, function: &Function, stack: &mut ~[Option<~Value>], environment: &mut Environment) {
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
        Jmp(n) => {

        }
        Ifleq(n) => {
            
        }
    }
}
