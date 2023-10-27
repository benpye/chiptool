use core::marker::PhantomData;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RW;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct R;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct W;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct URUW;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct RUW;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct UR;
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct UW;

mod sealed {
    use super::*;
    pub trait Access {}
    impl Access for R {}
    impl Access for W {}
    impl Access for RW {}
    impl Access for UR {}
    impl Access for UW {}
    impl Access for RUW {}
    impl Access for URUW {}
}

pub trait Access: sealed::Access + Copy {}
impl Access for R {}
impl Access for W {}
impl Access for RW {}
impl Access for UR {}
impl Access for UW {}
impl Access for RUW {}
impl Access for URUW {}

pub trait Read: Access {}
impl Read for RUW {}
impl Read for RW {}
impl Read for R {}

pub trait Write: Access {}
impl Write for RW {}
impl Write for W {}

pub trait UnsafeRead: Access {}
impl UnsafeRead for URUW {}
impl UnsafeRead for RUW {}
impl UnsafeRead for UR {}
impl UnsafeRead for RW {}
impl UnsafeRead for R {}

pub trait UnsafeWrite: Access {}
impl UnsafeWrite for URUW {}
impl UnsafeWrite for UW {}
impl UnsafeWrite for RW {}
impl UnsafeWrite for W {}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Reg<T: Copy, A: Access> {
    ptr: *mut u8,
    phantom: PhantomData<*mut (T, A)>,
}
unsafe impl<T: Copy, A: Access> Send for Reg<T, A> {}
unsafe impl<T: Copy, A: Access> Sync for Reg<T, A> {}

impl<T: Copy, A: Access> Reg<T, A> {
    #[inline(always)]
    pub const unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self {
            ptr: ptr as _,
            phantom: PhantomData,
        }
    }

    #[inline(always)]
    pub const fn as_ptr(&self) -> *mut T {
        self.ptr as _
    }
}

impl<T: Copy, A: Read> Reg<T, A> {
    #[inline(always)]
    pub fn read(&self) -> T {
        unsafe { (self.ptr as *mut T).read_volatile() }
    }
}

impl<T: Copy, A: UnsafeRead> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn unsafe_read(&self) -> T {
        unsafe { (self.ptr as *mut T).read_volatile() }
    }
}

impl<T: Copy, A: Write> Reg<T, A> {
    #[inline(always)]
    pub fn write_value(&self, val: T) {
        unsafe { (self.ptr as *mut T).write_volatile(val) }
    }
}

impl<T: Copy, A: UnsafeWrite> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn unsafe_write_value(&self, val: T) {
        unsafe { (self.ptr as *mut T).write_volatile(val) }
    }
}

impl<T: Default + Copy, A: Write> Reg<T, A> {
    #[inline(always)]
    pub fn write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

impl<T: Default + Copy, A: UnsafeWrite> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn unsafe_write<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = Default::default();
        let res = f(&mut val);
        unsafe { self.unsafe_write_value(val) };
        res
    }
}

impl<T: Copy, A: Read + Write> Reg<T, A> {
    #[inline(always)]
    pub fn modify<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = self.read();
        let res = f(&mut val);
        self.write_value(val);
        res
    }
}

impl<T: Copy, A: UnsafeRead + UnsafeWrite> Reg<T, A> {
    #[inline(always)]
    pub unsafe fn unsafe_modify<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut val = unsafe { self.unsafe_read() };
        let res = f(&mut val);
        unsafe { self.unsafe_write_value(val) };
        res
    }
}
