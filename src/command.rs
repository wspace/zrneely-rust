
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
    Slide(Number),

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
macro_rules! mov_le {
    ($x:ident <- $y:expr) => {
        {
            let n: u64 = $y;
            vec![
                0x48,
                $x,
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
    }
}

macro_rules! fn_call {
    ($x:ident : $y:expr, RSI: $z:expr) => {
        vec![
            mov_le!(RDI <- $y as *const _ as u64),
            mov_le!(RSI <- $z),
            mov_le!(RCX <- $crate::wsstd::Context::$x as u64),
            // call rcx
            vec![0xff, 0xd1],
        ].concat()
    };
    ($x:ident : $y:expr, RSI_setter: $z:expr) => {
        vec![
            mov_le!(RDI <- $y as *const _ as u64),
            $z,
            mov_le!(RCX <- $crate::wsstd::Context::$x as u64),
            // call rcx
            vec![0xff, 0xd1],
        ].concat()
    };
    ($x:ident : $y:expr) => {
        vec![
            mov_le!(RDI <- $y as *const _ as u64),
            mov_le!(RCX <- $crate::wsstd::Context::$x as u64),
            // call rcx
            vec![0xff, 0xd1],
        ].concat()
    };
}

macro_rules! arith {
    ($c:expr, $x:expr) => {
        vec![
            fn_call!(pop_stack: $c),
            // mov r12, rax
            vec![0x49, 0x89, 0xc4],
            fn_call!(pop_stack: $c),
            $x,
            fn_call!(push_stack: $c, RSI_setter: vec![0x48, 0x89, 0xc6]),
                                                     // mov rsi, rax
        ].concat()
    }
}

impl Command {
    /// Converts this command into assembly.
    /// TODO handle "linking"
    pub fn assemble(self, c: &Context) -> Vec<u8> {
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
            Command::Deinitialize | Command::Exit => vec![
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
            Command::Push(n) => fn_call!(push_stack: c, RSI: n as u64),
            Command::Duplicate => vec![
                fn_call!(peek_stack: c, RSI: 0),
                fn_call!(push_stack: c, RSI_setter: vec![0x48, 0x89, 0xc6]),
                                                           // mov rsi, rax
            ].concat(),
            Command::Swap => vec![
                fn_call!(pop_stack: c),
                // mov rbx, rax
                vec![0x48, 0x89, 0xc3],

                fn_call!(pop_stack: c),
                // mov r12, rax ; store returned value elsewhere
                vec![0x49, 0x89, 0xc4],

                fn_call!(push_stack: c, RSI_setter: vec![0x48, 0x89, 0xde]),
                                                         // mov rsi, rbx

                fn_call!(push_stack: c, RSI_setter: vec![0x4c, 0x89, 0xe6]),
                                                         // mov rsi, r12
            ].concat(),
            Command::Pop => fn_call!(pop_stack: c),
            Command::Copy(n) => vec![
                fn_call!(peek_stack: c, RSI: n as u64),
                fn_call!(push_stack: c, RSI_setter: vec![0x48, 0x89, 0xc6]),
                                                         // mov rsi, rax
            ].concat(),
            Command::Add => arith!(c, vec![0x4c, 0x01, 0xe0]),
                                           // add rax, r12
            Command::Subtract => arith!(c, vec![0x4c, 0x29, 0xe0]),
                                                // sub rax, r12
            Command::Multiply => arith!(c, vec![0x49, 0xf7, 0xe4]),
                                                // mul r12
            Command::Divide => arith!(c, vec![0x48, 0x99,
                                              // cqo    ; sign extend rax into rdx:rax
                                              0x49, 0xf7, 0xfc]),
                                              // idiv r12
            Command::Modulus => arith!(c, vec![0x48, 0x99,
                                               // cqo
                                               0x49, 0xf7, 0xfc,
                                               // idiv r12
                                               0x48, 0x89, 0xd0]),
                                               // mov rax, rdx
            Command::OutputChar => fn_call!(print: c, RSI: 1),
            Command::OutputNum => fn_call!(print: c, RSI: 0),
            _ => unimplemented!(),
        }
    }
}
