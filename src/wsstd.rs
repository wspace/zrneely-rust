
extern crate libc;

use libc as c;

use std::collections::HashMap;
use std::fmt;

pub type Literal = i64;

/// The context of a running program.
#[repr(C)]
pub struct Context {
    // "exported" function pointers usable by the jit-ed code
    // offset + 0x00: push to stack
    // offset + 0x08: pop from stack
    // offset + 0x10: store in heap
    // offset + 0x18: retrieve from heap
    // offset + 0x20: store label
    // offset + 0x28: get pointer to label
    pub fns: [*const c::c_void; 6],

    stack: Vec<Literal>,
    heap: HashMap<Literal, Literal>,
    call_stack: Vec<Literal>, // TODO can we use the native call/return functionality and stack?
    // maps literals to jump-to-able addresses in the function
    labels: HashMap<Literal, *const c::c_void>,
}

impl fmt::Debug for Context {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(formatter, "Context {{\n\t fns: ("));
        for ptr in self.fns.iter() {
            try!(write!(formatter, "{:p}, ", ptr));
        }
        write!(formatter,
               ")\n\t stack: {:?}\
               \n\t heap: {:?}\
               \n\t call stack: {:?}\
               \n\t labels: {:?}\
               \n}}",
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

            fns: [Context::__push_stack as *const c::c_void,
                  Context::__pop_stack as *const c::c_void,
                  Context::__store as *const c::c_void,
                  Context::__retrieve as *const c::c_void,
                  Context::__store_label as *const c::c_void,
                  Context::__retrieve_label as *const c::c_void],
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

    /// Called from jit-ed code. Stores a label.
    #[no_mangle]
    pub unsafe extern "C" fn __store_label(&mut self, name: Literal, ptr: *const c::c_void) {
        self.labels.insert(name, ptr);
    }

    /// Called from jit-ed code. Retrieves a label.
    #[no_mangle]
    pub unsafe extern "C" fn __retrieve_label(&self, name: Literal) -> *const c::c_void {
       *self.labels.get(&name).unwrap()
    }

}
