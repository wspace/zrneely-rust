
use ::Literal;

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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Arithmetic {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulus,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Heap {
    Store,
    Retrieve,
}

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

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum IO {
    OutputChar,
    OutputNum,
    ReadChar,
    ReadNum,
}
