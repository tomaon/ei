pub trait AsMutPtr<T: ?Sized> {
    fn as_mut_ptr(&mut self) -> *mut T;
}

impl AsMutPtr<u8> for u16 {
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut u16 as *mut u8
    }
}

impl AsMutPtr<u8> for u32 {
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut u32 as *mut u8
    }
}

impl AsMutPtr<u8> for u64 {
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut u64 as *mut u8
    }
}
