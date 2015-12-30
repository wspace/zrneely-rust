
use libc as c;
use std::mem::{uninitialized, transmute, forget};
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
    pub fn new(num_pages: usize) -> Self {
        unsafe {
            let page_size = c::sysconf(c::_SC_PAGESIZE) as usize;
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
            c::mprotect(transmute(self.contents), self.size, c::PROT_READ | c::PROT_EXEC);
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
            c::mprotect(transmute(self.contents), self.size, c::PROT_READ | c::PROT_WRITE);
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
        unsafe {
            &*self.contents.offset(_index as isize)
        }
    }
}

impl IndexMut<usize> for JitMemory {
    fn index_mut(&mut self, _index: usize) -> &mut u8 {
        if _index > self.size {
            panic!("index {} out of bounds for JitMemory", _index);
        }
        unsafe {
            &mut *self.contents.offset(_index as isize)
        }
    }
}
