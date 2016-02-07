
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

// little-endian expansion
fn le_expand(n: u64) -> Vec<u8> {
    println!("n: 0x{:x}", n);
    vec![
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

impl Command {
    /// Converts this command into assembly.
    /// TODO handle "linking"
    pub fn assemble(self, context: &Context) -> Vec<u8> {
        match self {
            Command::Initialize => vec![
                // TODO what needs to happen?
                // unimplemented!(),
            ],
            Command::Deinitialize => vec![
                // ret
                0xC3
            ],
            Command::Push(n) => vec![
                    // push rbx     ; we need to save it
                vec![0x53,
                    //              ; start pushing function arguments
                    // movabs rax, &context
                     0x48, 0xB8], le_expand(context as *const _ as u64),
                    // push rax
                vec![0x50,
                    // movabs rax, n
                     0x48, 0xB8], le_expand(n as u64),
                    // push rax
                vec![0x50,
                    // mov rbx, context.fns[0]
                     0x48, 0xBB], le_expand(context.fns[0] as u64),
                    // call rbx
                vec![0xFF, 0xD3,
                    // pop rax
                     0x58,
                    // pop rbx
                     0x5B],
            ].concat(),
            Command::Copy => vec![
                // rax = __pop_stack()
                unimplemented!(),
                // __push_stack(rax)
                unimplemented!(),
                // __push_stack(rax)
                unimplemented!(),
            ],
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
