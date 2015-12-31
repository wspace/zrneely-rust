
use Literal;

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum IMP {
    Stack,
    Arithmetic,
    Heap,
    Flow,
    IO,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Command {
    // Stack commands
    Push(Literal),
    Copy,
    Swap,
    Pop,

    // Arithmetic commands
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,

    // Heap commands
    Store,
    Retrieve,

    // Flow control commands
    Mark(Literal),
    Call(Literal),
    Jump(Literal),
    JumpZero(Literal),
    JumpNegative(Literal),
    Return,
    Exit,

    // IO commands
    OutputChar,
    OutputNum,
    ReadChar,
    ReadNum,
}
