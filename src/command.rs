
use {Number, Label};
use wsstd::Context;

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
    // Meta commands
    Initialize,
    Deinitialize,

    // Stack commands
    Push(Number),
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
    Mark(Label),
    Call(Label),
    Jump(Label),
    JumpZero(Label),
    JumpNegative(Label),
    Return,
    Exit,

    // IO commands
    OutputChar,
    OutputNum,
    ReadChar,
    ReadNum,
}

const RAX: u8 = 0xb8;
const RBX: u8 = 0xbb;
const RCX: u8 = 0xb9;
const RDX: u8 = 0xba;
const RDI: u8 = 0xbf;
const RSI: u8 = 0xbe;
/// little-endian move
fn mov_le(reg: u8, n: u64) -> Vec<u8> {
    vec![
        0x48,
        reg,
        ((n >> 0x00) as u8),
        ((n >> 0x08) as u8),
        ((n >> 0x10) as u8),
        ((n >> 0x18) as u8),
        ((n >> 0x20) as u8),
        ((n >> 0x28) as u8),
        ((n >> 0x30) as u8),
        ((n >> 0x38) as u8),
    ]
}

macro_rules! refint {
    ($x:expr) => ($x as *const _ as u64)
}

impl Command {
    /// Converts this command into assembly.
    /// TODO handle "linking"
    pub fn assemble(self, context: &Context) -> Vec<u8> {
        match self {
            Command::Initialize => vec![
                // push rbp
                0x55,
                // mov rbp, rsp
                0x48, 0x89, 0xe5,
            ],
            Command::Deinitialize => vec![
                // mov rsp, rbp
                0x48, 0x89, 0xec,
                // pop rbp
                0x5d,
                // ret
                0xc3
            ],
            Command::Push(n) => vec![
                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, n
                mov_le(RSI, n as u64),
                // mov rax, push_stack
                mov_le(RAX, Context::push_stack as u64),
                // call rax
                vec![0xff, 0xd0],
            ].concat(),
            Command::Copy => vec![
                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rcx, pop_stack
                mov_le(RCX, Context::pop_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],

                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, rax
                vec![0x48, 0x89, 0xc6],
                // mov rcx, push_stack
                mov_le(RCX, Context::push_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],

                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, rax
                vec![0x48, 0x89, 0xc6],
                // mov rc, push_stack
                mov_le(RCX, Context::push_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],
            ].concat(),
            Command::Swap => vec![
                // rax = __pop_stack()
                unimplemented!(),
                // mov rdx, rax ; TODO do I need to push/pop rdx? Should I use a different
                // register? Should I know more assembly before doing this? Yes hahaha :(
                0x48, 0x89, 0xC2,
                // rax = __pop_stack()
                unimplemented!(),
                // __push_stack(rax)
                unimplemented!(),
                // __push_stack(rdx)
                unimplemented!(),
            ],
            Command::Pop => vec![
                // __pop_stack()
                unimplemented!(),
            ],
            _ => unimplemented!(),
        }
    }
}
