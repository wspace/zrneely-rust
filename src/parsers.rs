
use command::*;
use {Label, Number};

// TODO properly handle "comments" aka non-legal characters, which should be
// ignored per the spec.


/// Identifies characters in a literal
named!(pub literal_char<bool>, map!(
        alt!(tag!(" ") | tag!("\t")),
        |c: &[u8]| { c[0] == b"\t"[0] })
);

/// Identifies a number. All numbers are represented as signed
/// integers of arbitrary width. We violate the spec a little here by
/// imposing a maximum width of 64 characters.
named!(pub number<Number>, map!(
        terminated!(
            many1!(literal_char),
            tag!("\n")
        ),
        |mut c: Vec<bool>| {
            // Reverse the non-sign bits
            c[1..].reverse();
            let mut mantissa = 1;
            let mut value = 0;
            for bit in &c[1..] {
                value += if *bit { mantissa } else { 0 };
                mantissa *= 2;
            }
            value * if c[0] { -1 } else { 1 }
        })
);

/// Identifies a label.
named!(pub label<Label>, map!(
    terminated!(
        many1!(literal_char),
        tag!("\n")
    ),
    |c: Vec<bool>| Label::Name(c)
));

/// Identifies an IMP.
named!(pub imp<IMP>, alt!(
    map!(tag!(" "), |_| IMP::Stack) |
    map!(tag!("\n"), |_| IMP::Flow) |
    map!(tag!("\t "), |_| IMP::Arithmetic) |
    map!(tag!("\t\t"), |_| IMP::Heap) |
    map!(tag!("\t\n"), |_| IMP::IO)
));

/// Identifies a stack instruction.
named!(pub stack<Command>, alt!(
    map!(preceded!(tag!(" "), number), |n| Command::Push(n)) |
    map!(tag!("\n "), |_| Command::Duplicate) |
    map!(preceded!(tag!("\t "), number), |n| Command::Copy(n)) |
    map!(tag!("\n\t"), |_| Command::Swap) |
    map!(tag!("\n\n"), |_| Command::Pop) |
    map!(preceded!(tag!("\t\n"), number), |n| Command::Slide(n))
));

/// Identifies a arithmetic instruction.
named!(pub arithmetic<Command>, alt!(
    map!(tag!("  "), |_| Command::Add) |
    map!(tag!(" \t"), |_| Command::Subtract) |
    map!(tag!(" \n"), |_| Command::Multiply) |
    map!(tag!("\t "), |_| Command::Divide) |
    map!(tag!("\t\t"), |_| Command::Modulus)
));

/// Identifies a heap instruction.
named!(pub heap<Command>, alt!(
    map!(tag!(" "), |_| Command::Store) |
    map!(tag!("\t"), |_| Command::Retrieve)
));

/// Identifies a flow control instruction.
named!(pub flow<Command>, alt!(
    map!(preceded!(tag!("  "), label), |n| Command::Mark(n)) |
    map!(preceded!(tag!(" \t"), label), |n| Command::Call(n)) |
    map!(preceded!(tag!(" \n"), label), |n| Command::Jump(n)) |
    map!(preceded!(tag!("\t "), label), |n| Command::JumpZero(n)) |
    map!(preceded!(tag!("\t\t"), label), |n| Command::JumpNegative(n)) |
    map!(tag!("\t\n"), |_| Command::Return) |
    map!(tag!("\n\n"), |_| Command::Exit)
));

/// Identifies an IO instruction.
named!(pub io<Command>, alt!(
    map!(tag!("  "), |_| Command::OutputChar) |
    map!(tag!(" \t"), |_| Command::OutputNum) |
    map!(tag!("\t "), |_| Command::ReadChar) |
    map!(tag!("\t\t"), |_| Command::ReadNum)
));

/// Identifies an entire command.
named!(pub command<Command>, switch!(imp,
    IMP::Stack => call!(stack) |
    IMP::Heap => call!(heap) |
    IMP::Arithmetic => call!(arithmetic) |
    IMP::Flow => call!(flow) |
    IMP::IO => call!(io)
));

/// Identifies an entire whitespace program.
named!(pub program<Vec<Command> >, many0!(command));

#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::*;
    use command::*;
    use Label;

    const NP: &'static str = "string not parsed";

    macro_rules! nom_match {
        ($parser: ident, $test: expr, $err: expr) => {
            match $parser($test) {
                IResult::Done(_, _) => {},
                _ => panic!($err),
            };
        };
        ($parser: ident, $test: expr, $expected: expr, $err: expr) => {
            assert_eq!($expected, match $parser($test) {
                IResult::Done(_, n) => n,
                _ => panic!($err),
            });
        };
    }
    macro_rules! nom_no_match {
        ($parser: ident, $test: expr, $err: expr) => {
            match $parser($test) {
                IResult::Done(_, _) => panic!($err),
                _ => {},
            };
        };
    }

    #[test]
    fn test_literal_char() {
        nom_match!(literal_char, b"\t", true, "tab not recognized");
        nom_match!(literal_char, b" ", false, "space not recognized");

        nom_no_match!(literal_char, b"\n", "newline mistakenly recognized");
    }

    #[test]
    fn test_number() {
        nom_match!(number, b" \t  \n", 4, NP);
        nom_match!(number, b" \t \t\n", 5, NP);
        nom_match!(number, b"\t\t \t \t \n", -42, NP);

        nom_no_match!(number, b"\n \n", "newline literal mistakenly recognized");
    }

    #[test]
    fn test_imp() {
        // Note: no negative tests possible since all combinations
        // of legal characters will match.
        nom_match!(imp, b"   \t\n", IMP::Stack, NP);
        nom_match!(imp, b"\n\n\n", IMP::Flow, NP);
        nom_match!(imp, b"\t   ", IMP::Arithmetic, NP);
        nom_match!(imp, b"\t\t  \t\n", IMP::Heap, NP);
        nom_match!(imp, b"\t\n  ", IMP::IO, NP);
    }

    #[test]
    fn test_stack() {
        nom_match!(stack, b"  \t \t \t \n", Command::Push(42), NP);
        nom_match!(stack, b"\n ", Command::Duplicate, NP);
        nom_match!(stack, b"\n\t", Command::Swap, NP);
        nom_match!(stack, b"\n\n", Command::Pop, NP);

        nom_no_match!(stack, b" \t ", "\" \\t\" mistakenly identified as stack");
    }

    #[test]
    fn test_arithmetic() {
        nom_match!(arithmetic, b"  ", Command::Add, NP);
        nom_match!(arithmetic, b" \t", Command::Subtract, NP);
        nom_match!(arithmetic, b" \n", Command::Multiply, NP);
        nom_match!(arithmetic, b"\t ", Command::Divide, NP);
        nom_match!(arithmetic, b"\t\t", Command::Modulus, NP);

        nom_no_match!(arithmetic,
                      b"\t\n",
                      "\"\\t\\n\" mistakenly identified as arithmetic");
    }

    #[test]
    fn test_heap() {
        nom_match!(heap, b" ", Command::Store, NP);
        nom_match!(heap, b"\t", Command::Retrieve, NP);

        nom_no_match!(heap, b"\n", "\"\\n\" mistakenly identified as heap");
    }

    #[test]
    fn test_flow() {
        nom_match!(flow,
                   b"   \t\n",
                   Command::Mark(Label::Name(vec![false, true])),
                   NP);
        nom_match!(flow,
                   b" \t \t\n",
                   Command::Call(Label::Name(vec![false, true])),
                   NP);
        nom_match!(flow,
                   b" \n \t\n",
                   Command::Jump(Label::Name(vec![false, true])),
                   NP);
        nom_match!(flow,
                   b"\t  \t\n",
                   Command::JumpZero(Label::Name(vec![false, true])),
                   NP);
        nom_match!(flow,
                   b"\t\t \t\n",
                   Command::JumpNegative(Label::Name(vec![false, true])),
                   NP);
        nom_match!(flow, b"\t\n", Command::Return, NP);
        nom_match!(flow, b"\n\n", Command::Exit, NP);

        nom_no_match!(flow, b"\n ", "\"\\n \" mistakenly identified as flow");
    }

    #[test]
    fn test_io() {
        nom_match!(io, b"  ", Command::OutputChar, NP);
        nom_match!(io, b" \t", Command::OutputNum, NP);
        nom_match!(io, b"\t ", Command::ReadChar, NP);
        nom_match!(io, b"\t\t", Command::ReadNum, NP);

        nom_no_match!(io, b"\n\n", "\"\\n\\n\" mistakenly identified as io");
    }

    #[test]
    fn test_command() {
        // Test a few of the commands: if all the other tests pass, this should be
        // sufficient.
        nom_match!(command, b"\n\n\n", Command::Exit, NP);
        nom_match!(command, b"\t  \t", Command::Subtract, NP);
        nom_match!(command, b"   \t \t \n", Command::Push(10), NP);
        nom_match!(command,
                   b"\n   \t    \t\t\n",
                   Command::Mark(Label::Name(vec![false, true, false, false, false, false,
                                                  true, true])),
                   NP);
        nom_match!(command,
                   b"\n\t  \t   \t \t\n",
                   Command::JumpZero(Label::Name(vec![false, true, false, false, false, true,
                                                      false, true])),
                   NP);

        nom_no_match!(command,
                      b"\t\n \n",
                      "\"\\t\\n \\t\" mistakenly identified as command");
    }

    #[test]
    fn test_program() {
        nom_match!(program,
                   b"   \t\n\n   \t    \t\t\n \n  \n\n\n\n\n",
                   vec![Command::Push(1),
                        Command::Mark(Label::Name(vec![false, true, false, false, false,
                                                       false, true, true])),
                        Command::Duplicate,
                        Command::Pop,
                        Command::Exit],
                   NP);

        nom_match!(program,
                   b"\t\n \t   \t \t \n\t\n     \t\n\t   ",
                   vec![Command::OutputNum,
                        Command::Push(10),
                        Command::OutputChar,
                        Command::Push(1),
                        Command::Add],
                   NP);
    }
}
