use opcode::*;
use libjit::*;
use analysis::*;
use std::vec;
//use std::io::*;


struct AnnotatedOpcode {
    opcode: Opcode,
    jmp_from: Option<u32>,
    jmp_to: Option<u32>,
    label: ~Label,
}


pub fn compile(program: &[Opcode], context: &Context) -> ~Function {
    context.build_start();

    let return_type = Types::get_float32();
    let params: &[&Type] = &[];

    let signature = Type::create_signature(CDECL, return_type, params);

    let function = context.create_function(signature);

    let mut stack = ~[];
    let mut locals = reserve_locals(program, function);

    let mut annotated_program = annotate_labels(program);
    annotated_program.push(~AnnotatedOpcode { opcode: Nop, jmp_from: None, jmp_to: None, label: Label::new() });

    let mut index = 0;
    while index < annotated_program.len() - 1 {
        compile_opcode(&mut annotated_program, index, function, &mut stack, &mut locals);      
        index += 1;
    }

    function.insn_set_label(annotated_program[index].label);

    //function.dump("factorial");

    function.compile();
    context.build_end();

    return function;
}

fn compile_opcode(annotated_program: &mut ~[~AnnotatedOpcode], index: uint, function: &Function, stack: &mut ~[Option<~Value>], locals: &mut ~[~Value]) {
    match annotated_program[index].jmp_from {
        Some(_) => {
            function.insn_set_label(annotated_program[index].label);
        }
        _ => ()
    }

    let opcode = annotated_program[index].opcode;
    match opcode {
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
            function.insn_return(*temp);
        },
        Disp            => {} //println(fmt!("%?", stack.pop()))
        Store(addr) => {
            let v = stack.pop();
            function.insn_store(locals[addr], *v.get_ref());
        }
        Load(addr) => {
            let v = locals[addr].clone();
            let new_value = Some(function.insn_dup(v));
            stack.push(new_value);
        }
        Jmp(n) => {
            function.insn_branch(annotated_program[n].label);
        }
        Ifleq(n) => {
            let v1 = stack.pop();
            let temp1 = v1.get_ref();
            let v2 = stack.pop();
            let temp2 = v2.get_ref();
            let temp_result = function.insn_leq(*temp2, *temp1);
            function.insn_branch_if(temp_result, annotated_program[n].label);
        }
        Nop => { }
    }
}

fn reserve_locals(program: &[Opcode], function: &Function) -> ~[~Value] {
    let local_count = local_count(program) as uint;
    let locals: ~[~Value] = vec::from_fn(local_count, |_| {
        function.create_value(Types::get_float32())
    });
    return locals;
}

fn annotate_labels(program: &[Opcode]) -> ~[~AnnotatedOpcode] {
    let mut annotated_program: ~[~AnnotatedOpcode] = vec::from_fn(program.len(), |i| {
        ~AnnotatedOpcode { opcode: program[i], jmp_to: None, jmp_from: None, label: Label::new()  }
    });
    let mut index = 0;
    for opcode in program.iter() {
        match *opcode {
            Jmp(n) => {
                annotated_program[index].jmp_to = Some(n);
                annotated_program[n].jmp_from = Some(index);
            }
            Ifleq(n) => {
                annotated_program[index].jmp_to = Some(n);
                annotated_program[n].jmp_from = Some(index);
            }
            _ => ()
        }
        index += 1;
    }
    return annotated_program;
}
