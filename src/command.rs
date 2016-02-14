
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
    Duplicate,
    Copy(Number),
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

                // we can use a few registers, but we have to restore them after
                // push rbx
                0x53,
                // push r12
                0x41, 0x54,
            ],
            Command::Deinitialize => vec![
                // pop r12
                0x41, 0x5c,
                // pop rbx
                0x5b,
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
            Command::Duplicate => vec![
                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, 0
                mov_le(RSI, 0),
                // mov rcx, peek_stack
                mov_le(RCX, Context::peek_stack as u64),
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
            ].concat(),
            Command::Swap => vec![
                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rcx, pop_stack
                mov_le(RCX, Context::pop_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],
                // mov rbx, rax ; store returned value elsewhere
                vec![0x48, 0x89, 0xc3],

                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rcx, pop_stack
                mov_le(RCX, Context::pop_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],
                // mov r12, rax ; store returned value elsewhere
                vec![0x49, 0x89, 0xc4],

                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, rbx
                vec![0x48, 0x89, 0xde],
                // mov rcx, push_stack
                mov_le(RCX, Context::push_stack as u64),
                // call rcx
                vec![0xff, 0xd1],

                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, r12
                vec![0x4c, 0x89, 0xe6],
                // mov rcx, push_stack
                mov_le(RCX, Context::push_stack as u64),
                // call rcx
                vec![0xff, 0xd1],
            ].concat(),
            Command::Pop => vec![
                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rcx, pop_stack
                mov_le(RCX, Context::pop_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],
            ].concat(),
            Command::Copy(n) => vec![
                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, n
                mov_le(RSI, n as u64),
                // mov rcx, peek_stack
                mov_le(RCX, Context::peek_stack as u64),
                // call rcx     ; result is in rax
                vec![0xff, 0xd1],

                // mov rdi, &context
                mov_le(RDI, refint!(context)),
                // mov rsi, rax
                vec![0x48, 0x89, 0xc6],
                // mov rcx, push_stack
                mov_le(RCX, Context::push_stack as u64),
                // call rcx
                vec![0xff, 0xd1],
            ].concat(),
            _ => unimplemented!(),
        }
    }
}
