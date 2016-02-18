
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
mod tests {
    use std::collections::HashMap;
    use wsstd::Label;

    struct Output {
        stdout: Option<String>,
        stack: Option<Vec<i64>>,
        heap: Option<HashMap<Label, i64>>
    }

    macro_rules! out {
        ( $( $stack:expr),* ; $stdout:expr ) => {{
            let mut stack = Vec::new();
            $(
                stack.push($stack);
            )*
            $crate::tests::Output {
                stdout: Some($stdout.to_string()),
                stack: Some(stack),
                heap: None,
            }
        }};
        ( $( $stack:expr),* ) => {{
            let mut stack = Vec::new();
            $(
                stack.push($stack);
            )*
            $crate::tests::Output {
                stdout: None,
                stack: Some(stack),
                heap: None,
            }
        }};
    }

    macro_rules! gen_tests {
        ( $($pkg:ident: {
            $($name:ident: $program:expr => $output:expr;)*
        })*  ) => {

            $(mod $pkg {
                use parsers::program;
                use wsstd::Context;
                use std::rc::Rc;
                use std::cell::RefCell;
                use std::ops::Deref;
                use ::{parse, get_native_function};

                $(
                    #[test]
                    fn $name() {
                        let program = parse($program).expect("Parsing failed!");
                        let mut context = Context::new();
                        let stdout_actual = Rc::new(RefCell::new(Vec::new()));
                        // TODO solve lifetime problems better
                        context.capture_stdout(stdout_actual.clone());
                        {
                            let program = get_native_function(program, &mut context);
                            program.execute();
                        }
                        if let Some(stack) = $output.stack {
                            // we reverse the stack here so that the top of the stack
                            // is at the left in test cases, which is easier to read
                            context.stack.reverse();
                            assert_eq!(context.stack, stack);
                        }
                        if let Some(heap) = $output.heap {
                            assert_eq!(context.heap, heap);
                        }
                        if let Some(stdout) = $output.stdout {
                            assert_eq!(String::from_utf8(
                                    stdout_actual.borrow().deref().clone()).unwrap(),
                                stdout);
                        }
                    }
                )*
            })*
        };
    }

    gen_tests! {
        stack: {
            // push 1
            push:       b"    \t\n"                         => out!(1);
            // push 1, duplicate
            duplicate:  b"    \t\n \n "                     => out!(1, 1);
            // push 2, push 1, pop
            pop:        b"   \t \n    \t\n \n\n"            => out!(2);
            // push 1, push 0, swap
            swap:       b"    \t\n    \n \n\t"              => out!(1, 0);
            // push 0, push 1, copy 1
            copy:       b"   \n    \t\n \t   \t\n"          => out!(0, 1, 0);
        }

        io: {
            // push 65, out_char
            char_out:   b"   \t     \t\n\t\n  "             => out!(65; "A");
            // push 65, out_int
            int_out:    b"   \t     \t\n\t\n \t"            => out!(65; "65");
        }

        arithmetic: {
            // push 1, push 3, add
            add:        b"   \t\n   \t\t\n\t   "            => out!(4);
            // push -1, push 3, add
            add_neg:    b"  \t\t\n   \t\t\n\t   "           => out!(2);
            // push 3, push 1, subtract
            sub:        b"   \t\t\n   \t\n\t  \t"           => out!(2);
            // push 1, push 3, subtract
            sub_neg:    b"   \t\n   \t\t\n\t  \t"           => out!(-2);
            // push 2, push 3, multiply
            mul:        b"   \t \n   \t\t\n\t  \n"          => out!(6);
            // push -2, push 3, multiply
            mul_neg:    b"  \t\t \n   \t\t\n\t  \n"         => out!(-6);
            // push -2, push -3, multiply
            mul_neg_2:  b"  \t\t \n  \t\t\t\n\t  \n"        => out!(6);
            // push 4, push 2, divide
            div:        b"   \t  \n   \t \n\t \t "          => out!(2);
            // push 5, push 2, divide
            div_round:  b"   \t \t\n   \t \n\t \t "         => out!(2);
            // push -4, push 2, divide
            div_neg:    b"  \t\t  \n   \t \n\t \t "         => out!(-2);
            // push -4, push -2, divide
            div_neg_2:  b"  \t\t  \n  \t\t \n\t \t "        => out!(2);
            // push 5, push 2, modulo
            modulo:     b"   \t \t\n   \t \n\t \t\t"        => out!(1);
            // push 2, push 5, modulo
            modulo_2:   b"   \t \n   \t \t\n\t \t\t"        => out!(2);
        }
    }
}
