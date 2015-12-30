
use ::command::*;
use Literal;

/// Identifies non-comment characters
named!(pub legal_char, alt!(tag!(" ") | tag!("\t") | tag!("\n")));

/// Identifies characters in a literal
named!(pub literal_char<bool>, map!(
        alt!(tag!(" ") | tag!("\t")),
        |c: &[u8]| { c[0] == b"\t"[0] })
);

/// Identifies a literal. All literals are represented signed numbers 
/// of arbitrary width. We violate the spec a little here by imposing 
/// a maximum width of 64 characters.
named!(pub literal<Literal>, map!(
        terminated!(
            many1!(literal_char),
            tag!("\n")
        ),
        |mut c: Vec<bool>| {
            // Reverse the non-sign bits
            c[1..].reverse();
            let sign = if c[0] { -1 } else { 1 };
            let mut mantissa = 1;
            let mut value = 0;
            for bit in &c[1..] {
                value += if *bit { mantissa } else { 0 };
                mantissa *= 2;
            }
            value * sign
        })
);

/// Identifies an IMP.
named!(pub imp<IMP>, alt!(
    map!(tag!(" "), |_| IMP::Stack) |
    map!(tag!("\n"), |_| IMP::Flow) |
    map!(tag!("\t "), |_| IMP::Arithmetic) |
    map!(tag!("\t\t"), |_| IMP::Heap) |
    map!(tag!("\t\n"), |_| IMP::IO)
));

/// Identifies a stack instruction.
named!(pub stack<Stack>, alt!(
    map!(preceded!(tag!(" "), literal), |n| Stack::Push(n)) |
    map!(tag!("\n "), |_| Stack::Copy) |
    map!(tag!("\n\t"), |_| Stack::Swap) |
    map!(tag!("\n\n"), |_| Stack::Pop)
));

/// Identifies a arithmetic instruction.
named!(pub arithmetic<Arithmetic>, alt!(
    map!(tag!("  "), |_| Arithmetic::Add) |
    map!(tag!(" \t"), |_| Arithmetic::Subtract) |
    map!(tag!(" \n"), |_| Arithmetic::Multiply) |
    map!(tag!("\t "), |_| Arithmetic::Divide) |
    map!(tag!("\t\t"), |_| Arithmetic::Modulus)
));

/// Identifies a heap instruction.
named!(pub heap<Heap>, alt!(
    map!(tag!(" "), |_| Heap::Store) |
    map!(tag!("\t"), |_| Heap::Retrieve)
));

/// Identifies a flow control instruction.
named!(pub flow<Flow>, alt!(
    map!(preceded!(tag!("  "), literal), |n| Flow::Mark(n)) |
    map!(preceded!(tag!(" \t"), literal), |n| Flow::Call(n)) |
    map!(preceded!(tag!(" \n"), literal), |n| Flow::Jump(n)) |
    map!(preceded!(tag!("\t "), literal), |n| Flow::JumpZero(n)) |
    map!(preceded!(tag!("\t\t"), literal), |n| Flow::JumpNegative(n)) |
    map!(tag!("\t\n"), |_| Flow::Return) |
    map!(tag!("\n\n"), |_| Flow::Exit)
));

/// Identifies an IO instruction.
named!(pub io<IO>, alt!(
    map!(tag!("  "), |_| IO::OutputChar) |
    map!(tag!(" \t"), |_| IO::OutputNum) |
    map!(tag!("\t "), |_| IO::ReadChar) |
    map!(tag!("\t\t"), |_| IO::ReadNum)
));


#[cfg(test)]
mod tests {
    use nom::IResult;
    use super::*;
    use command::*;

    macro_rules! nom_match {
        ( $parser: ident, $test: expr, $err: expr ) => {
            match $parser($test) {
                IResult::Done(_, _) => {},
                _ => panic!($err),
            };
        };
        ( $parser: ident, $test: expr, $expected: expr, $err: expr ) => {
            assert_eq!($expected, match $parser($test) {
                IResult::Done(_, n) => n,
                _ => panic!($err),
            });
        };
    }
    macro_rules! nom_no_match {
        ( $parser: ident, $test: expr, $err: expr ) => {
            match $parser($test) {
                IResult::Done(_, _) => panic!($err),
                _ => {},
            };
        };
    }

    #[test]
    fn test_legal_char() {
        nom_match!(legal_char, b" ", "space not recognized");
        nom_match!(legal_char, b"\t", "tab not recognized");
        nom_match!(legal_char, b"\n", "newline not recognized");

        nom_no_match!(legal_char, b"a", "a mistakenly recognized");
    }

    #[test]
    fn test_literal_char() {
        nom_match!(literal_char, b"\t", true, "tab not recognized");
        nom_match!(literal_char, b" ", false, "space not recognized");

        nom_no_match!(literal_char, b"\n", "newline mistakenly recognized");
    }

    #[test]
    fn test_literal() {
        nom_match!(literal, b" \t  \n", 4, "string not parsed");
        nom_match!(literal, b" \t \t\n", 5, "string not parsed");
        nom_match!(literal, b"\t\t \t \t \n", -42, "string not parsed");

        nom_no_match!(literal, b"\n \n", "newline literal mistakenly recognized");
    }

    #[test]
    fn test_imp() {
        // Note: no negative tests possible since all combinations
        // of legal characters will match.
        nom_match!(imp, b"   \t\n", IMP::Stack, "string not parsed");
        nom_match!(imp, b"\n\n\n", IMP::Flow, "string not parsed");
        nom_match!(imp, b"\t   ", IMP::Arithmetic, "string not parsed");
        nom_match!(imp, b"\t\t  \t\n", IMP::Heap, "string not parsed");
        nom_match!(imp, b"\t\n  ", IMP::IO, "string not parsed");
    }

    #[test]
    fn test_stack() {
        nom_match!(stack,
                   b"  \t \t \t \n",
                   Stack::Push(42),
                   "string not parsed");
        nom_match!(stack, b"\n ", Stack::Copy, "string not parsed");
        nom_match!(stack, b"\n\t", Stack::Swap, "string not parsed");
        nom_match!(stack, b"\n\n", Stack::Pop, "string not parsed");

        nom_no_match!(stack, b" \t ", "\" \\t\" mistakenly identified as stack");
    }

    #[test]
    fn test_arithmetic() {
        nom_match!(arithmetic, b"  ", Arithmetic::Add, "string not parsed");
        nom_match!(arithmetic,
                   b" \t",
                   Arithmetic::Subtract,
                   "string not parsed");
        nom_match!(arithmetic,
                   b" \n",
                   Arithmetic::Multiply,
                   "string not parsed");
        nom_match!(arithmetic, b"\t ", Arithmetic::Divide, "string not parsed");
        nom_match!(arithmetic,
                   b"\t\t",
                   Arithmetic::Modulus,
                   "string not parsed");

        nom_no_match!(arithmetic,
                      b"\t\n",
                      "\"\\t\\n\" mistakenly identified as arithmetic");
    }

    #[test]
    fn test_heap() {
        nom_match!(heap, b" ", Heap::Store, "string not parsed");
        nom_match!(heap, b"\t", Heap::Retrieve, "string not parsed");

        nom_no_match!(heap, b"\n", "\"\\n\" mistakenly identified as heap");
    }

    #[test]
    fn test_flow() {
        nom_match!(flow, b"   \t\n", Flow::Mark(1), "string not parsed");
        nom_match!(flow, b" \t \t\n", Flow::Call(1), "string not parsed");
        nom_match!(flow, b" \n \t\n", Flow::Jump(1), "string not parsed");
        nom_match!(flow, b"\t  \t\n", Flow::JumpZero(1), "string not parsed");
        nom_match!(flow,
                   b"\t\t \t\n",
                   Flow::JumpNegative(1),
                   "string not parsed");
        nom_match!(flow, b"\t\n", Flow::Return, "string not parsed");
        nom_match!(flow, b"\n\n", Flow::Exit, "string not parsed");

        nom_no_match!(flow, b"\n ", "\"\\n \" mistakenly identified as flow");
    }

    #[test]
    fn test_io() {
        nom_match!(io, b"  ", IO::OutputChar, "string not parsed");
        nom_match!(io, b" \t", IO::OutputNum, "string not parsed");
        nom_match!(io, b"\t ", IO::ReadChar, "string not parsed");
        nom_match!(io, b"\t\t", IO::ReadNum, "string not parsed");

        nom_no_match!(io, b"\n\n", "\"\\n\\n\" mistakenly identified as io");
    }
}
