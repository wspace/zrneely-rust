
use Literal;

/// Marker trait for commands
pub trait Command {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum IMP {
    Stack,
    Arithmetic,
    Heap,
    Flow,
    IO,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Stack {
    Push(Literal),
    Copy,
    Swap,
    Pop,
}
impl Command for Stack {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}
impl Command for Arithmetic {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Heap {
    Store,
    Retrieve,
}
impl Command for Heap {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Flow {
    Mark(Literal),
    Call(Literal),
    Jump(Literal),
    JumpZero(Literal),
    JumpNegative(Literal),
    Return,
    Exit,
}
impl Command for Flow {}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum IO {
    OutputChar,
    OutputNum,
    ReadChar,
    ReadNum,
}
impl Command for IO {}
