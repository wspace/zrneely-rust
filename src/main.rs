
extern crate libc;
#[macro_use]
extern crate nom;

mod jit;
mod parsers;
mod command;

use parsers::*;
use command::*;

pub type Literal = i64;

fn main() {
    println!("hello, world");
}
