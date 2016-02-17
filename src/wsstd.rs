
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
    pub stack: Vec<Number>,
    pub heap: HashMap<Label, Number>,
    // maps literals to jump-to-able addresses in the function
    pub labels: HashMap<Label, Address>,
}

impl fmt::Debug for Context {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(formatter,
               "Context {{\n\t stack: {:?}\n\t heap: {:?}\n\t labels: {:?}\n}}",
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
        }
    }

    // Marked as unsafe to indicate that they're not meant to be called
    // from Rust, but from the jit-d code.

    /// Called from jit-ed code. Pushes a value onto the stack, then
    /// returns that same value.
    pub unsafe extern fn push_stack(&mut self, arg: Number) -> Number {
        self.stack.push(arg);
        arg
    }

    /// Called from jit-ed code. Pops a value off the stack, returning that value.
    /// If the stack is empty, returns 0. The situation where the stack is empty
    /// isn't defined in the spec, but it's hard to deal with panics across FFI boundaries.
    pub unsafe extern fn pop_stack(&mut self) -> Number {
        self.stack.pop().unwrap_or_else(|| { Context::err("WS pop stack error!"); 0 })
    }

    /// Called from jit-ed code. Reads a value from the n'th place in the stack, and
    /// returns it.
    pub unsafe extern fn peek_stack(&self, arg: Number) -> Number {
        if let Some(val) = self.stack.get(self.stack.len() - arg as usize - 1) {
            *val
        } else {
            Context::err("WS peek stack error!");
            0
        }
    }

    /// Called from jit-ed code. Puts data in the heap.
    pub unsafe extern fn store(&mut self, name: Label, value: Number) {
        self.heap.insert(name, value);
    }

    /// Called from jit-ed code. Retrieves data from the heap.
    pub unsafe extern fn retrieve(&self, name: Label) -> Number {
        *self.heap.get(&name).unwrap()
    }

    /// Called from jit-ed code. Stores a label.
    pub unsafe extern fn store_label(&mut self, name: Label, ptr: Address) {
        self.labels.insert(name, ptr);
    }

    /// Called from jit-ed code. Retrieves a label.
    pub unsafe extern fn retrieve_label(&self, name: Label) -> Address {
        *self.labels.get(&name).unwrap()
    }

    /// Called from jit-ed code. Displays data to stdout.
    pub unsafe extern fn print(&self, is_char: bool) {
        let num = self.peek_stack(0);
        if is_char {
            print!("{}", String::from_utf8(vec![num as u8]).expect("Non-ascii print"));
        } else {
            print!("{}", num);
        }
    }

    /// Called from jit-ed code. Reads data from stdin.
    pub unsafe extern fn read(is_char: bool) -> Number {
        if is_char {
            // TODO
            unimplemented!()
        } else {
            // TODO
            unimplemented!()
        }
    }

    fn err(val: &'static str) {
        println!("{}", val);
    }
}
