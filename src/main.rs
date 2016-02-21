
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

pub use wsstd::{Label, Number};

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
        _ => None,
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

    struct Input {
        program: String,
        stdin: Option<&'static str>,
    }

    macro_rules! inp {
        ( $program:expr ) => {
            $crate::tests::Input {
                program: $program.to_string(),
                stdin: None,
            }
        };
        ( $program:expr; $stdin:expr ) => {
            $crate::tests::Input {
                program: $program.to_string(),
                stdin: Some($stdin),
            }
        };
    }

    struct Output {
        stdout: String,
        stack: Vec<i64>,
        heap: HashMap<i64, i64>,
    }

    macro_rules! out {
        ( [ $( $stack:expr),* ]; $stdout:expr; { $( $key:expr => $value:expr )* } ) => {{
            let mut stack = Vec::new();
            $(
                stack.push($stack);
            )*
            let mut heap = HashMap::new();
            $(
                heap.insert($key, $value);
            )*
            $crate::tests::Output {
                stdout: $stdout.to_string(),
                stack: stack,
                heap: heap,
            }
        }};
    }

    macro_rules! gen_tests {
        ( $($pkg:ident: {
            $($name:ident: $input:expr => $output:expr;)*
        })*  ) => {

            $(mod $pkg {
                use parsers::program;
                use wsstd::Context;
                use std::rc::Rc;
                use std::cell::RefCell;
                use std::ops::Deref;
                use std::collections::HashMap;
                use ::{parse, get_native_function};

                $(
                    #[test]
                    #[allow(unused_mut)]
                    fn $name() {
                        let input = $input;
                        let output = $output;

                        let program = parse(input.program.as_bytes())
                            .expect("Parsing failed!");
                        let mut context = Context::new();

                        let stdout_actual = Rc::new(RefCell::new(Vec::new()));
                        context.capture_stdout(stdout_actual.clone());

                        context.provide_stdin(input.stdin.unwrap_or(""));

                        {
                            let program = get_native_function(program, &mut context);
                            program.execute();
                        }

                        // we reverse the stack here so that the top of the stack
                        // is at the left in test cases, which is easier to read
                        context.stack.reverse();
                        assert_eq!(context.stack, output.stack);

                        assert_eq!(context.heap, output.heap);

                        assert_eq!(String::from_utf8(
                                stdout_actual.borrow().deref().clone()).unwrap(),
                                output.stdout);
                    }
                )*
            })*
        };
    }

    gen_tests! {
        stack: {
            // push 1
            push:      inp!("    \t\n")                    => out!([1]; ""; {});
            // push 1, duplicate
            duplicate: inp!("    \t\n \n ")                => out!([1, 1]; ""; {});
            // push 2, push 1, pop
            pop:       inp!("   \t \n    \t\n \n\n")       => out!([2]; ""; {});
            // push 1, push 0, swap
            swap:      inp!("    \t\n    \n \n\t")         => out!([1, 0]; ""; {});
            // push 0, push 1, copy 1
            copy:      inp!("   \n    \t\n \t   \t\n")     => out!([0, 1, 0]; ""; {});
        }

        heap: {
            // push "1", push 5, store
            store:     inp!("   \t\n   \t \t\n\t\t ")      => out!([5, 1]; "";
                                                                   { 1 => 5 });
            // push "101", push 5, store, push "111", push 6, store
            store_2:   inp!("   \t \t\n   \t \t\n\t\t    \t\t\t\n   \t\t \n\t\t ")
                                                           => out!([6, 7, 5, 5]; ""; {
                                                                       5 => 5
                                                                       7 => 6
                                                                   });
            // push "101", push 3, store, push "101", retrieve
            ret:       inp!("   \t \t\n   \t\t\n\t\t    \t \t\n\t\t\t")
                                                           => out!([3, 5, 3, 5]; ""; {
                                                                       5 => 3
                                                                  });
        }

        io: {
            // push 65, out_char
            char_out:  inp!("   \t     \t\n\t\n  ")        => out!([65]; "A"; {});
            // push 65, out_int
            int_out:   inp!("   \t     \t\n\t\n \t")       => out!([65]; "65"; {});
            // push "101", in_char
            char_in:   inp!("   \t \t\n\t\n\t "; "A\n")    => out!([5]; ""; { 5 => 65 });
            // push "101", in_int
            int_in:    inp!("   \t \t\n\t\n\t\t"; "65")    => out!([5]; ""; { 5 => 65 });
        }

        arithmetic: {
            // push 1, push 3, add
            add:       inp!("   \t\n   \t\t\n\t   ")       => out!([4]; ""; {});
            // push -1, push 3, add
            add_neg:   inp!("  \t\t\n   \t\t\n\t   ")      => out!([2]; ""; {});
            // push 3, push 1, subtract
            sub:       inp!("   \t\t\n   \t\n\t  \t")      => out!([2]; ""; {});
            // push 1, push 3, subtract
            sub_neg:   inp!("   \t\n   \t\t\n\t  \t")      => out!([-2]; ""; {});
            // push 2, push 3, multiply
            mul:       inp!("   \t \n   \t\t\n\t  \n")     => out!([6]; ""; {});
            // push -2, push 3, multiply
            mul_neg:   inp!("  \t\t \n   \t\t\n\t  \n")    => out!([-6]; ""; {});
            // push -2, push -3, multiply
            mul_neg_2: inp!("  \t\t \n  \t\t\t\n\t  \n")   => out!([6]; ""; {});
            // push 4, push 2, divide
            div:       inp!("   \t  \n   \t \n\t \t ")     => out!([2]; ""; {});
            // push 5, push 2, divide
            div_round: inp!("   \t \t\n   \t \n\t \t ")    => out!([2]; ""; {});
            // push -4, push 2, divide
            div_neg:   inp!("  \t\t  \n   \t \n\t \t ")    => out!([-2]; ""; {});
            // push -4, push -2, divide
            div_neg_2: inp!("  \t\t  \n  \t\t \n\t \t ")   => out!([2]; ""; {});
            // push 5, push 2, modulo
            modulo:    inp!("   \t \t\n   \t \n\t \t\t")   => out!([1]; ""; {});
            // push 2, push 5, modulo
            modulo_2:  inp!("   \t \n   \t \t\n\t \t\t")   => out!([2]; ""; {});
        }

        flow: {
            // push 1, exit, push 2
            exit:      inp!("   \t\n\n\n\n   \t \n")       => out!([1]; "";  {});
        }
    }

}
