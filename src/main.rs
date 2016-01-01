
extern crate libc;
#[macro_use]
extern crate nom;

extern crate wsstd;

mod jit;
mod parsers;
mod command;

use nom::IResult;

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
    println!("{:?}", Context::new());


    let input_file = unimplemented!();
    let input = unimplemented!();

    // println!("Parsing input...");
    let program = match parsers::program(input) {
        IResult::Done(_, mut program) => {
            program.insert(0, Command::Initialize);
            program
        }
        // TODO better error handling
        _ => panic!("Invalid program!"),
    };
    println!("Compiling...");
    let program = get_native_function(program);
    println!("Running:");
    program.execute(Context::new());
}
