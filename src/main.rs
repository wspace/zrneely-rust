
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
mod runtest {
    macro_rules! gen_tests {
        ( $($pkg:ident: {
            $($name:ident: $program:expr => ($stack: expr, $heap: expr);)*
        })*  ) => {

            $(mod $pkg {
                use parsers::program;
                use wsstd::Context;
                use ::{parse, get_native_function};

                $(
                    #[test]
                    fn $name() {
                        let program = parse($program).expect("Parsing failed!");
                        let mut context = Context::new();
                        {
                            let program = get_native_function(program, &mut context);
                            program.execute();
                        }
                        if let Some(stack) = $stack {
                            // we reverse the stack here so that the top of the stack
                            // is at the left in test cases, which is easier to read
                            context.stack.reverse();
                            assert_eq!(context.stack, &stack);
                        }
                        if let Some(heap) = $heap {
                            assert_eq!(context.heap, heap);
                        }
                    }
                )*
            })*
        };
    }

    gen_tests! {
        stack: {
            // push 1
            push:       b"    \t\n"                       => (Some([1]),         None);
            // push 1, duplicate
            duplicate:  b"    \t\n \n "                   => (Some([1, 1]),      None);
            // push 1, pop
            pop:        b"    \t\n \n\n"                  => (Some([]),          None);
            // push 1, push 0, swap
            swap:       b"    \t\n    \n \n\t"            => (Some([1, 0]),      None);
            // push 0, push 1, copy 1
            copy:       b"   \n    \t\n \t   \t\n"        => (Some([0, 1, 0]),   None);
        }

        arithmetic: {
            // push 1, push 3, add
            add:        b"   \t\n   \t\t\n\t   "          => (Some([4]),         None);
            // push -1, push 3, add
            add_neg:    b"  \t\t\n   \t\t\n\t   "         => (Some([2]),         None);
            // push 3, push 1, subtract
            sub:        b"   \t\t\n   \t\n\t  \t"         => (Some([2]),         None);
            // push 1, push 3, subtract
            sub_neg:    b"   \t\n   \t\t\n\t  \t"         => (Some([-2]),        None);
        }
    }
}
