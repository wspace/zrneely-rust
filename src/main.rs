
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

pub use wsstd::{Number, Label};

fn get_native_function<'a>(program: Vec<Command>, context: &'a mut Context) -> JitFunction<'a> {
    let machine_code = program.into_iter()
                              .map(|x| x.assemble(context))
                              .collect::<Vec<Vec<u8>>>()
                              .concat();

    // TODO something with labels

    let pages = (machine_code.len() / JitMemory::get_page_size()) + 1;
    let mut memory = JitMemory::new(pages);

    memory.copy_from(&machine_code[..]);
    memory.into()
}

fn parse(program: &[u8]) -> Option<Vec<Command>> {
    match parsers::program(program) {
        IResult::Done(_, mut program) => {
            program.insert(0, Command::Initialize);
            program.push(Command::Deinitialize);
            Some(program)
        }
        _ => None
    }
}

fn main() {

    let input_file = unimplemented!();
    let input = unimplemented!();

    let program = parse(input).expect("Invalid program!");

    let mut context = Context::new();
    {
        let program = get_native_function(program, &mut context);

        program.execute();
    }
    println!("Done!\n{:?}", context);
}

#[cfg(test)]
mod test {
    use parsers::program;
    use wsstd::Context;
    use super::{parse, get_native_function};

    #[test]
    fn test_push() {
        let input = b"    \t\n";
        let program = parse(input).expect("Parsing failed!");
        let mut context = Context::new();
        {
            let program = get_native_function(program, &mut context);

            program.execute();
        }

        assert_eq!(context.stack, [1]);
    }

    #[test]
    fn test_copy() {
        let input = b"    \t\n \n ";
        let program = parse(input).expect("Parsing failed!");
        let mut context = Context::new();
        {
            let program = get_native_function(program, &mut context);

            program.execute();
        }

        assert_eq!(context.stack, [1, 1]);
    }

    #[test]
    fn test_pop() {
        let input = b"    \t\n \n\n";
        let program = parse(input).expect("Parsing failed!");
        let mut context = Context::new();
        {
            let program = get_native_function(program, &mut context);

            program.execute();
        }

        assert_eq!(context.stack, []);
    }
}
