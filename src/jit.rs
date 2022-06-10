
use libc as c;
use memmap2::{Mmap, MmapMut};
use std::mem::transmute;
use std::ops::{Index, IndexMut};
use std::marker::PhantomData;

use wsstd::Context;

pub struct JitMemory<'a> {
    contents: MmapMut,
    size: usize,
    phantom: PhantomData<&'a mut Context>,
}

pub struct JitFunction<'a> {
    contents: Mmap,
    size: usize,
    phantom: PhantomData<&'a mut Context>,
}

impl<'a> JitMemory<'a> {

    pub fn get_page_size() -> usize {
        unsafe { c::sysconf(c::_SC_PAGESIZE) as usize }
    }

    pub fn new(num_pages: usize) -> Self {
        let page_size = JitMemory::get_page_size();
        let size = num_pages * page_size;

        // Let's allocate space for the JIT function.
        let mut page = MmapMut::map_anon(size).unwrap();
        // Fill it with "int" (0xCC) to avoid using uninitialized memory.
        page.fill(0xcc);

        JitMemory {
            contents: page,
            size: size,
            phantom: PhantomData,
        }
    }

    /// Copies a slice into memory
    pub fn copy_from(&mut self, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self[i] = *byte;
        }
    }
}

impl<'a> JitFunction<'a> {
    pub fn execute(self) -> i64 {
        let f: fn() -> i64 = unsafe {
            transmute(self.contents.as_ptr())
        };
        f()
    }
}

impl<'a> Into<JitFunction<'a>> for JitMemory<'a> {
    fn into(self) -> JitFunction<'a> {
        // Mark the function as executable, but not writable.
        JitFunction {
            contents: self.contents.make_exec().unwrap(),
            size: self.size,
            phantom: PhantomData,
        }
    }
}

impl<'a> Into<JitMemory<'a>> for JitFunction<'a> {
    fn into(self) -> JitMemory<'a> {
        // Mark the function as writable, but not executable.
        JitMemory {
            contents: self.contents.make_mut().unwrap(),
            size: self.size,
            phantom: PhantomData,
        }
    }
}

impl<'a> Index<usize> for JitMemory<'a> {
    type Output = u8;

    fn index(&self, _index: usize) -> &u8 {
        if _index > self.size {
            panic!("index {} out of bounds for JitMemory", _index);
        }
        &self.contents[_index]
    }
}

impl<'a> IndexMut<usize> for JitMemory<'a> {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        if _index > self.size {
            panic!("index {} out of bounds for JitMemory", _index);
        }
        &mut self.contents[_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check_output(program: &[u8], output: i64) {
        let mut memory = JitMemory::new(program.len() / JitMemory::get_page_size() + 1);

        memory.copy_from(program);

        let function: JitFunction = memory.into();
        assert_eq!(output, function.execute());
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn jit() {
        check_output(&[0x48, 0xC7, 0xC0, 0x20, 0x00, 0x00, 0x00,    // mov rax, 0x20
                       0x48, 0x83, 0xC0, 0x0A,                      // add rax, 0x0A
                       0x48, 0x83, 0xE8, 0x0A,                      // sub rax, 0x0A
                       0xC3], 32);                                  // ret
        check_output(&[0x48, 0xC7, 0xC0, 0x20, 0x00, 0x00, 0x00,    // mov rax, 0x20
                       0x48, 0x83, 0xC0, 0x0A,                      // add rax, 0x0A
                       0xC3], 42);                                  // ret
    }
}
