
extern crate libc;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::fmt;
use std::io::{self, BufRead, BufReader, Read, Write};

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
    pub heap: HashMap<Number, Number>,
    // maps literals to jump-to-able addresses in the function
    pub labels: HashMap<Label, Address>,

    stdin: BufReader<Box<dyn Read>>,
    stdout: Rc<RefCell<dyn Write>>,
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
            stdin: BufReader::new(Box::new(io::stdin())),
            stdout: Rc::new(RefCell::new(io::stdout())),
        }
    }

    /// Allows capturing stdout; very useful for test cases
    pub fn capture_stdout(&mut self, out: Rc<RefCell<dyn Write>>) {
        self.stdout = out;
    }

    /// Allows providing stdin; very useful for test cases
    pub fn provide_stdin(&mut self, inp: &'static str) {
        self.stdin = BufReader::new(Box::new(inp.as_bytes()));
    }

    // Marked as unsafe to indicate that they're not meant to be called
    // from Rust, but from the jit-d code.

    /// Called from jit-ed code. Pushes a value onto the stack, then
    /// returns that same value.
    pub unsafe extern "C" fn push_stack(&mut self, arg: Number) -> Number {
        self.stack.push(arg);
        arg
    }

    /// Called from jit-ed code. Pops a value off the stack, returning that value.
    /// If the stack is empty, returns 0. The situation where the stack is empty
    /// isn't defined in the spec, but it's hard to deal with panics across FFI boundaries.
    pub unsafe extern "C" fn pop_stack(&mut self) -> Number {
        self.stack.pop().unwrap_or_else(|| {
            Context::err("WS pop stack error!");
            0
        })
    }

    /// Called from jit-ed code. Reads a value from the n'th place in the stack, and
    /// returns it.
    pub unsafe extern "C" fn peek_stack(&self, arg: Number) -> Number {
        if let Some(val) = self.stack.get(self.stack.len() - arg as usize - 1) {
            *val
        } else {
            Context::err("WS peek stack error!");
            0
        }
    }

    /// Called from jit-ed code. Reads the two values on top of the heap and stores
    /// them.
    pub unsafe extern "C" fn store(&mut self) {
        let name = self.stack.get(self.stack.len() - 2).unwrap();
        let value = self.stack.get(self.stack.len() - 1).unwrap();
        self.heap.insert(*name, *value);
    }

    /// Called from jit-ed code. Retrieves data from the heap.
    pub unsafe extern "C" fn retrieve(&self) -> Number {
        *self.heap.get(&self.stack.get(self.stack.len() - 1).unwrap()).unwrap()
    }

    /// Called from jit-ed code. Displays data to stdout.
    pub unsafe extern "C" fn print(&mut self, is_char: bool) {
        let num = self.peek_stack(0);
        let mut out = self.stdout.borrow_mut();
        if is_char {
            write!(*out,
                   "{}",
                   String::from_utf8(vec![num as u8]).expect("Non-ascii print"))
                .unwrap();
        } else {
            write!(*out, "{}", num).unwrap();
        }
    }

    /// Called from jit-ed code. Reads data from stdin.
    pub unsafe extern "C" fn read(&mut self, is_char: bool) {
        let name = self.stack.get(self.stack.len() - 1).unwrap();
        let mut line = String::new();
        if is_char {
            self.stdin.read_line(&mut line).unwrap();
            self.heap.insert(*name, line.as_bytes()[0] as i64);
        } else {
            self.stdin.read_line(&mut line).unwrap();
            self.heap.insert(*name, i64::from_str_radix(&line, 10).unwrap());
        }
    }

    fn err(val: &'static str) {
        println!("{}", val);
    }
}
