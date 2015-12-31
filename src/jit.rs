
use libc as c;
use std::mem::{forget, transmute, uninitialized};
use std::ops::{Index, IndexMut};

extern {
    fn memset(s: *mut c::c_void, c: c::uint32_t, n: c::size_t) -> *mut c::c_void;
}

pub struct JitMemory {
    contents: *mut u8,
    size: usize,
}

pub struct JitFunction {
    contents: fn() -> i64,
    size: usize,
}

impl JitMemory {

    pub fn get_page_size() -> usize {
        unsafe { c::sysconf(c::_SC_PAGESIZE) as usize }
    }

    pub fn new(num_pages: usize) -> Self {
        let page_size = JitMemory::get_page_size();

        unsafe {
            let size = num_pages * page_size;

            // Let's allocate space for the JIT function.
            let mut page: *mut c::c_void = uninitialized();

            // It has to be aligned...
            c::posix_memalign(&mut page, page_size, size);

            // ...and marked as writable.
            c::mprotect(page, size, c::PROT_READ | c::PROT_WRITE);

            // Fill it with "int" (0xCC) to avoid using uninitialized memory. This will
            // cause a SIGTRAP immediately on execution.
            memset(page, 0xCC, size);

            JitMemory {
                contents: transmute(page),
                size: size,
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

impl Drop for JitMemory {
    fn drop(&mut self) {
        unsafe {
            c::free(transmute(self.contents));
        }
    }
}

impl Drop for JitFunction {
    fn drop(&mut self) {
        unsafe {
            c::free(transmute(self.contents));
        }
    }
}

impl JitFunction {
    pub fn execute(&self) -> i64 {
        (self.contents)()
    }
}

impl Into<JitFunction> for JitMemory {
    fn into(self) -> JitFunction {
        // Mark the function as executable, but not writable.
        unsafe {
            c::mprotect(transmute(self.contents),
                        self.size,
                        c::PROT_READ | c::PROT_EXEC);
            let function = JitFunction {
                contents: transmute(self.contents),
                size: self.size,
            };
            // Don't call the destructor
            forget(self);
            function
        }
    }
}

impl Into<JitMemory> for JitFunction {
    fn into(self) -> JitMemory {
        // Mark the function as writable, but not executable.
        unsafe {
            c::mprotect(transmute(self.contents),
                        self.size,
                        c::PROT_READ | c::PROT_WRITE);
            let memory = JitMemory {
                contents: transmute(self.contents),
                size: self.size,
            };
            // Don't call the destructor
            forget(self);
            memory
        }
    }
}

impl Index<usize> for JitMemory {
    type Output = u8;

    fn index(&self, _index: usize) -> &u8 {
        if _index > self.size {
            panic!("index {} out of bounds for JitMemory", _index);
        }
        unsafe { &*self.contents.offset(_index as isize) }
    }
}

impl IndexMut<usize> for JitMemory {
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

    fn check_output(program: &[u8], output: i64) {
        let mut memory = JitMemory::new(program.len() / JitMemory::get_page_size() + 1);

        memory.copy_from(program);

        let function: JitFunction = memory.into();
        assert_eq!(output, function.execute());
    }

    #[test]
    #[cfg_attr(rustfmt, rustfmt_skip)]
    fn test_jit() {
        check_output(&[0x48, 0xC7, 0xC0, 0x20, 0x00, 0x00, 0x00,    // mov rax, 0x20
                       0x48, 0x83, 0xC0, 0x0A,                      // add rax, 0x0A
                       0x48, 0x83, 0xE8, 0x0A,                      // sub rax, 0x0A
                       0xC3], 32);                                  // ret
        check_output(&[0x48, 0xC7, 0xC0, 0x20, 0x00, 0x00, 0x00,    // mov rax, 0x20
                       0x48, 0x83, 0xC0, 0x0A,                      // add rax, 0x0A
                       0xC3], 42);                                  // ret
    }

}
