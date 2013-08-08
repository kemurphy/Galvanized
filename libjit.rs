use std::libc::*;
use std::vec;
use std::ptr;
use std::io::*;


pub enum ABI {
    CDECL = 0
}

#[link_args = "-ljit"]
extern {
    fn jit_context_create() -> *c_void;
    fn jit_context_destroy(context: *c_void);
    fn jit_context_build_start(context: *c_void);
    fn jit_context_build_end(context: *c_void);
    fn jit_function_create(context: *c_void, signature: *c_void) -> *c_void;
    fn jit_function_compile(function: *c_void);
    fn jit_type_create_signature(abi: c_int, return_type: *c_void, params: **c_void, num_params: c_uint, incref: c_int) -> *c_void;
    fn jit_value_get_param(function: *c_void, param: c_uint) -> *c_void;
    fn jit_insn_return(function: *c_void, value: *c_void);
    fn jit_function_apply(function: *c_void, args: **c_void, return_area: *mut c_void);
    fn jit_insn_add(function: *c_void, v1: *c_void, v2: *c_void) -> *c_void;
    fn jit_insn_mul(function: *c_void, v1: *c_void, v2: *c_void) -> *c_void;
    fn jit_insn_sub(function: *c_void, v1: *c_void, v2: *c_void) -> *c_void;
    fn jit_insn_div(function: *c_void, v1: *c_void, v2: *c_void) -> *c_void;
    fn jit_insn_load(function: *c_void, value: *c_void) -> *c_void;
    fn jit_value_create(function: *c_void, value_type: *c_void) -> *c_void;
    fn jit_insn_label(function: *c_void, label: **c_void);
    fn jit_insn_branch(function: *c_void, label: **c_void);
    fn jit_insn_le(function: *c_void, v1: *c_void, v2: *c_void) -> *c_void;
    fn jit_insn_branch_if(function: *c_void, value: *c_void, label: **c_void);
    fn jit_insn_store(function: *c_void, dest: *c_void, src: *c_void);
    fn jit_dump_function (stream: *FILE, funcion: *c_void, name: *c_char);
    fn jit_value_create_float32_constant(function: *c_void, value_type: *c_void, value: c_float) -> *c_void;
    fn jit_function_to_closure(function: *c_void) -> extern "C" unsafe fn() -> c_float;

    static jit_type_void: *c_void;
    static jit_type_int: *c_void;
    static jit_type_float32: *c_void;
    static jit_type_float64: *c_void;
}


pub struct Context {
    priv _context: *c_void
}

impl Context {
    pub fn new() -> ~Context {
        unsafe {
            let context = jit_context_create();
            ~Context { _context: context }
        }
    }

    pub fn build_start(&self) {
        unsafe {
            jit_context_build_start(self._context);
        }
    }

    pub fn build_end(&self) {
        unsafe {
            jit_context_build_end(self._context);
        }
    }

    pub fn create_function(&self, signature: &Type) -> ~Function {
        unsafe {
            let function = jit_function_create(self._context, signature._type);
            ~Function { _context: self, _function: function }
        }
    }
}

impl Drop for Context {
    fn drop(&self) {
        unsafe {
            jit_context_destroy(self._context);
        }
    }
}

pub struct Type {
    priv _type: *c_void
}

// CRASHES compiler
//pub static void: ~Type = ~Type { _type: jit_type_void };

impl Type {
    pub fn create_signature(abi: ABI, return_type: &Type, params: &[&Type]) -> ~Type {
        unsafe {
            let mut ps: ~[*c_void] = ~[];

            for param in params.iter() {
                ps.push(param._type);
            }

            let params = if ps.len() > 0 { vec::raw::to_ptr(ps) } else { 0 as **c_void };

            let signature = jit_type_create_signature(abi as c_int, return_type._type, params, ps.len() as c_uint, 1);
            ~Type { _type: signature }
        }
    }
}

pub struct Function {
    priv _context: *Context,
    priv _function: *c_void
}


impl Function {
    priv fn insn_binop(&self, v1: &Value, v2: &Value, f: extern "C" unsafe fn(function: *c_void, v1: *c_void, v2: *c_void) -> *c_void) -> ~Value {
        unsafe {
            let value = f(self._function, v1._value, v2._value);
            ~Value { _value: value }
        }
    }

    pub fn dump(&self, name: &str) {
        unsafe {
            name.as_c_str(|c_str| {
                jit_dump_function(rustrt::rust_get_stdout(), self._function, c_str);
            });
        }
    }

    pub fn compile(&self) {
        unsafe {
            jit_function_compile(self._function);
        }
    }

    pub fn get_param(&self, param: uint) -> Value {
        unsafe {
            let value = jit_value_get_param(self._function, param as c_uint);
            Value { _value: value }
        }
    }

    pub fn insn_return(&self, retval: &Value) {
        unsafe {
            jit_insn_return(self._function, retval._value);
        }
    }

    pub fn insn_mul(&self, v1: &Value, v2: &Value) -> ~Value {
        self.insn_binop(v1, v2, jit_insn_mul)
    }

    pub fn insn_add(&self, v1: &Value, v2: &Value) -> ~Value {
        self.insn_binop(v1, v2, jit_insn_add)
    }

    pub fn insn_sub(&self, v1: &Value, v2: &Value) -> ~Value {
        self.insn_binop(v1, v2, jit_insn_sub)
    }

    pub fn insn_div(&self, v1: &Value, v2: &Value) -> ~Value {
        self.insn_binop(v1, v2, jit_insn_div)
    }

    pub fn insn_leq(&self, v1: &Value, v2: &Value) -> ~Value {
        self.insn_binop(v1, v2, jit_insn_le)
    }

    pub fn insn_dup(&self, value: &Value) -> ~Value {
        unsafe {
            let dup_value = jit_insn_load(self._function, value._value);
            ~Value { _value: dup_value }
        }
    }

    pub fn insn_store(&self, dest: &Value, src: &Value) {
        unsafe {
            jit_insn_store(self._function, dest._value, src._value);
        }
    }

    pub fn insn_label(&self) -> ~Label {
        unsafe {
            let label = ~Label { _label: 0 as *c_void };
            jit_insn_label(self._function, &label._label as **c_void);
            label
        }
    }

    pub fn insn_set_label(&self, label: &Label) {
        unsafe {
            jit_insn_label(self._function, &label._label as **c_void);
        }
    }

    pub fn insn_branch(&self, label: &Label) {
        unsafe {
            jit_insn_branch(self._function, &label._label as **c_void);
        }
    }

    pub fn insn_branch_if(&self, value: &Value, label: &Label) {
        unsafe {
            jit_insn_branch_if(self._function, value._value, &label._label as **c_void);
        }
    }

    pub fn apply<T>(&self, args: &[*c_void], retval: &mut T) {
        unsafe {
            let pargs = vec::raw::to_ptr(args);
            jit_function_apply(self._function, pargs as **c_void, ptr::to_mut_unsafe_ptr(retval) as *mut c_void);
        }
    }

    pub fn execute(&self, args: &[*c_void]) {
        unsafe {
            let pargs = vec::raw::to_ptr(args);
            jit_function_apply(self._function, pargs as **c_void, ptr::mut_null());
        }
    }

    pub fn closure(&self) -> extern "C" unsafe fn() -> c_float {
        unsafe {
            jit_function_to_closure(self._function)
        }
    }

    pub fn constant_float32(&self, constant: f32) -> ~Value {
        unsafe {
            let value = jit_value_create_float32_constant(self._function, jit_type_float32, constant as c_float);
            ~Value { _value: value }
        }
    }

    pub fn create_value(&self, value_type: &Type) -> ~Value {
        unsafe {
            let value = jit_value_create(self._function, value_type._type);
            ~Value { _value: value }
        }
    }
}


pub struct Value {
    priv _value: *c_void
}

impl Value {

}


impl Clone for Value {
    pub fn clone(&self) -> Value {
        Value { _value: self._value }
    }
}


pub struct Label {
    priv _label: *c_void
}

impl Label {
    fn undefined() -> *c_void {
        !0u32 as *c_void
    }

    pub fn new() -> ~Label {
        ~Label { _label: Label::undefined() }
    }
}


priv struct Types;
impl Types {
    pub fn get_void() -> ~Type {
        ~Type { _type: jit_type_void }
    }

    pub fn get_int() -> ~Type {
        ~Type { _type: jit_type_int }   
    }

    pub fn get_float32() -> ~Type {
        ~Type { _type: jit_type_float32 }   
    }
}