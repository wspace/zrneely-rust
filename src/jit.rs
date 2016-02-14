
use libc as c;
use std::mem::{forget, transmute, uninitialized};
use std::ops::{Index, IndexMut};
use std::marker::PhantomData;

use wsstd::Context;

extern {
    fn memset(s: *mut c::c_void, c: c::uint32_t, n: c::size_t) -> *mut c::c_void;
}

pub struct JitMemory<'a> {
    contents: *mut u8,
    size: usize,
    phantom: PhantomData<&'a mut Context>,
}

pub struct JitFunction<'a> {
    contents: fn() -> i64,
    size: usize,
    phantom: PhantomData<&'a mut Context>,
}

impl<'a> JitMemory<'a> {

    pub fn get_page_size() -> usize {
        unsafe { c::sysconf(c::_SC_PAGESIZE) as usize }
    }

    pub fn new(num_pages: usize) -> Self {
        let page_size = JitMemory::get_page_size();

        unsafe {
            let size = num_pages * page_size;

            // Let's allocate space for the JIT function.
            // It has to be aligned...
            let mut page: *mut c::c_void = uninitialized();
            c::posix_memalign(&mut page, page_size, size);

            // ...and marked as writable.
            c::mprotect(page, size, c::PROT_READ | c::PROT_WRITE);

            // Fill it with "int" (0xCC) to avoid using uninitialized memory. This will
            // cause a SIGTRAP immediately on execution.
            memset(page, 0xcc, size);

            JitMemory {
                contents: transmute(page),
                size: size,
                phantom: PhantomData,
            }
        }
    }

    /// Copies a slice into memory
    pub fn copy_from(&mut self, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self[i] = *byte;
        }
    }
}

impl<'a> Drop for JitMemory<'a> {
    fn drop(&mut self) {
        unsafe {
            c::free(transmute(self.contents));
        }
    }
}

impl<'a> Drop for JitFunction<'a> {
    fn drop(&mut self) {
        unsafe {
            c::free(transmute(self.contents));
        }
    }
}

impl<'a> JitFunction<'a> {
    pub fn execute(self) -> i64 {
        (self.contents)()
    }
}

impl<'a> Into<JitFunction<'a>> for JitMemory<'a> {
    fn into(self) -> JitFunction<'a> {
        // Mark the function as executable, but not writable.
        unsafe {
            c::mprotect(transmute(self.contents),
                        self.size,
                        c::PROT_READ | c::PROT_EXEC);
            let function = JitFunction {
                contents: transmute(self.contents),
                size: self.size,
                phantom: PhantomData,
            };
            // Don't call the destructor
            forget(self);
            function
        }
    }
}

impl<'a> Into<JitMemory<'a>> for JitFunction<'a> {
    fn into(self) -> JitMemory<'a> {
        // Mark the function as writable, but not executable.
        unsafe {
            c::mprotect(transmute(self.contents),
                        self.size,
                        c::PROT_READ | c::PROT_WRITE);
            let memory = JitMemory {
                contents: transmute(self.contents),
                size: self.size,
                phantom: PhantomData,
            };
            // Don't call the destructor
            forget(self);
            memory
        }
    }
}

impl<'a> Index<usize> for JitMemory<'a> {
    type Output = u8;

    fn index(&self, _index: usize) -> &u8 {
        if _index > self.size {
            panic!("index {} out of bounds for JitMemory", _index);
        }
        unsafe { &*self.contents.offset(_index as isize) }
    }
}

impl<'a> IndexMut<usize> for JitMemory<'a> {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        if _index > self.size {
            panic!("index {} out of bounds for JitMemory", _index);
        }
        unsafe { &mut *self.contents.offset(_index as isize) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wsstd::Context;

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
