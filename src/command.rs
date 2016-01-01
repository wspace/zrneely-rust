
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

    // Meta commands
    Initialize,

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

//
// address argument to call is offset in elf (from beginning of
//  file) according to objdump. Is that just because the base
//  address happens to be zero? No, that doesn't even make sense - the
//  function pointer doesn't even end with that offset.
// how to call there from executable memory on the heap?
// does call take an offset, or can we know at compile time the address
// a function will be at in memory thanks to virtualized memory spaces?
// how to get that address in elf regardless? do we need it?
//  or do we need the address of it at runtime, where it's loaded
//  into memory?
//
// - open binary, read symbol table at runtime - blech, the hackiest
// version of reflection
//  - get info from linker at compile time - ideal, but how?
//  - read symbol table after compilation with post-build script
//      - store result in config file
//      - modify created binary (would require a not-optimized-away
//          constant)
//  - put a bunch of function pointers in the Context struct. Might be
//      the only way, especially if optimization strips the symbol
//      table
//
// still need to figure out calling conventions for x64. Used extern
// in wsstd, so we can use C ABI instead of Rust's non-standard one.
//

impl Command {
    /// Converts this command into assembly.
    /// TODO might have to make a separate "linking" step if
    /// we need to be able to jump to addresses that are marked later
    /// in the file
    pub fn assemble(self) -> Vec<u8> {
        match self {
            Command::Initialize => vec![
                // TODO what needs to happen?
                unimplemented!(),
            ],
            Command::Push(n) => vec![
                // __push_stack(n)
                unimplemented!(),
            ],
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
