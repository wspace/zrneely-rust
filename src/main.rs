
extern crate libc;
#[macro_use]
extern crate nom;

extern crate wsstd;

mod jit;
mod parsers;
mod command;

use command::Command;
use jit::{JitFunction, JitMemory};
use wsstd::Context;

pub use wsstd::Literal;

fn get_native_function(program: Vec<Command>) -> JitFunction {
    let machine_code = program.into_iter()
                              .map(|x| x.assemble())
                              .collect::<Vec<Vec<u8>>>()
                              .concat();

    let pages = (machine_code.len() / JitMemory::get_page_size()) + 1;
    let mut memory = JitMemory::new(pages);

    memory.copy_from(&machine_code[..]);
    memory.into()
}

fn main() {
    // This should be all that really needs to happen:

    // println!("Parsing input...");
    // let program = parsers::program(read_input_file());
    // println!("Compiling...");
    // let program = get_native_function(program);
    // println!("Running:");
    // program.execute(Context::new());

    println!("{:?}", Context::new());
}
