
extern crate libc;

use libc as c;

use std::collections::HashMap;
use std::fmt;

pub type Literal = i64;

/// The context of a running program.
#[repr(C)]
pub struct Context {
    // "exported" function pointers usable by the jit-ed code
    pub fn0: *const c::c_void,
    pub fn1: *const c::c_void,
    pub fn2: *const c::c_void,
    pub fn3: *const c::c_void,

    stack: Vec<Literal>,
    heap: HashMap<Literal, Literal>,
    call_stack: Vec<Literal>,
    // maps literals to jump-to-able addresses in the function
    labels: HashMap<Literal, *const c::c_void>,
}

impl fmt::Debug for Context {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter,
               "Context {{\
               \n\t fns: ({:p}, {:p}, {:p}, {:p})\
               \n\t stack: {:?}\
               \n\t heap: {:?}\
               \n\t call stack: {:?}\
               \n\t labels: {:?}\
               \n}}",
               self.fn0,
               self.fn1,
               self.fn2,
               self.fn3,
               self.stack,
               self.heap,
               self.call_stack,
               self.labels)
    }
}

impl Context {
    /// Create a new context.
    pub fn new() -> Self {
        Context {
            stack: Vec::new(),
            heap: HashMap::new(),
            call_stack: Vec::new(),
            labels: HashMap::new(),

            fn0: Context::__push_stack as *const c::c_void,
            fn1: Context::__pop_stack as *const c::c_void,
            fn2: Context::__store as *const c::c_void,
            fn3: Context::__retrieve as *const c::c_void,
        }
    }

    // TODO the rest of these functions

    // Marked as unsafe to indicate that they're not meant to be called
    // from Rust, but from the jit-d code.

    /// Called from jit-ed code. Pushes a value onto the stack.
    #[no_mangle]
    pub unsafe extern "C" fn __push_stack(&mut self, arg: Literal) {
        self.stack.push(arg);
    }

    /// Called from jit-ed code. Pops a value off the stack,
    /// returning that value.
    #[no_mangle]
    pub unsafe extern "C" fn __pop_stack(&mut self) -> Literal {
        self.stack.pop().unwrap()
    }

    /// Called from jit-ed code. Puts data in the heap.
    #[no_mangle]
    pub unsafe extern "C" fn __store(&mut self, name: Literal, value: Literal) {
        self.heap.insert(name, value);
    }

    /// Called from jit-ed code. Retrieves data from the heap.
    #[no_mangle]
    pub unsafe extern "C" fn __retrieve(&self, name: Literal) -> Literal {
        *self.heap.get(&name).unwrap()
    }

}
