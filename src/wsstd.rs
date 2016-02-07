
extern crate libc;

use std::collections::HashMap;
use std::fmt;

pub type Number = i64;
pub type Address = usize;

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum Label {
    Name(Vec<bool>),
    Translated(Address),
}

impl Label {
    fn replace(self, mapping: &HashMap<Vec<bool>, Address>) -> Option<Label> {
        if let Label::Name(name) = self {
            mapping.get(&name)
                   .map(|&addr| Label::Translated(addr))
        } else {
            Some(self)
        }
    }
}

/// The context of a running program.
pub struct Context {
    // "exported" function pointers usable by the jit-ed code
    // offset + 0x00: push to stack
    // offset + 0x08: pop from stack
    // offset + 0x10: store in heap
    // offset + 0x18: retrieve from heap
    // offset + 0x20: store label
    // offset + 0x28: get pointer to label
    // offset + 0x30: print data
    // offset + 0x38: read data
    pub fns: [Address; 8],

    stack: Vec<Number>,
    heap: HashMap<Label, Number>,
    // maps literals to jump-to-able addresses in the function
    labels: HashMap<Label, Address>,
}

impl fmt::Debug for Context {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        try!(write!(formatter, "Context {{\n\t fns: ["));
        for ptr in self.fns.iter() {
            try!(write!(formatter, "0x{:x}, ", *ptr));
        }
        write!(formatter,
               "]\n\t stack: {:?}\
               \n\t heap: {:?}\
               \n\t labels: {:?}\
               \n}}",
               self.stack,
               self.heap,
               self.labels)
    }
}

impl Context {
    /// Create a new context.
    pub fn new() -> Self {
        Context {
            stack: Vec::new(),
            heap: HashMap::new(),
            labels: HashMap::new(),

            fns: [Context::__push_stack as Address,
                  Context::__pop_stack as Address,
                  Context::__store as Address,
                  Context::__retrieve as Address,
                  Context::__store_label as Address,
                  Context::__retrieve_label as Address,
                  Context::__print as Address,
                  Context::__read as Address],
        }
    }

    // Marked as unsafe to indicate that they're not meant to be called
    // from Rust, but from the jit-d code.

    /// Called from jit-ed code. Pushes a value onto the stack.
    pub unsafe extern "C" fn __push_stack(&mut self, arg: Number) {
        self.stack.push(arg);
    }

    /// Called from jit-ed code. Pops a value off the stack,
    /// returning that value.
    pub unsafe extern "C" fn __pop_stack(&mut self) -> Number {
        self.stack.pop().unwrap()
    }

    /// Called from jit-ed code. Puts data in the heap.
    pub unsafe extern "C" fn __store(&mut self, name: Label, value: Number) {
        self.heap.insert(name, value);
    }

    /// Called from jit-ed code. Retrieves data from the heap.
    pub unsafe extern "C" fn __retrieve(&self, name: Label) -> Number {
        *self.heap.get(&name).unwrap()
    }

    /// Called from jit-ed code. Stores a label.
    pub unsafe extern "C" fn __store_label(&mut self, name: Label, ptr: Address) {
        self.labels.insert(name, ptr);
    }

    /// Called from jit-ed code. Retrieves a label.
    pub unsafe extern "C" fn __retrieve_label(&self, name: Label) -> Address {
        *self.labels.get(&name).unwrap()
    }

    /// Called from jit-ed code. Displays data to stdout.
    pub unsafe extern "C" fn __print(data: Number, is_char: bool) {
        if is_char {
            // TODO
            unimplemented!()
        } else {
            println!("{}", data);
        }
    }

    /// Called from jit-ed code. Reads data from stdin.
    pub unsafe extern "C" fn __read(is_char: bool) -> Number {
        if is_char {
            // TODO
            unimplemented!()
        } else {
            // TODO
            unimplemented!()
        }
    }

}
