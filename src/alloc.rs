use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

/// Abstracted typed allocator
///
/// Similar to `std::heap::Alloc`, but more high-level and limited to single type
pub trait Alloc: Default {
    type Boxed: Boxed;

    fn alloc(&self, value: <<Self as Alloc>::Boxed as Deref>::Target) -> Self::Boxed;
}

// Abstracted allocated box
//
// Similar to `Box`, this trait represents a wrapper type whose value is allocated "somewhere"
// and release its memory when dropped.
pub trait Boxed: Drop + Deref + DerefMut {
    fn unbox(self) -> <Self as Deref>::Target;
}

/// Simple typed allocator, just a wrapper around `Box`
/// This can be useful for comparison.
pub struct BoxAlloc<T>(PhantomData<T>);

impl<T> Default for BoxAlloc<T> {
    fn default() -> Self {
        BoxAlloc(Default::default())
    }
}

impl<T> Alloc for BoxAlloc<T> {
    type Boxed = Box<T>;

    fn alloc(&self, value: T) -> Box<T> {
        Box::new(value)
    }
}

impl<T> Boxed for Box<T> {
    fn unbox(self) -> T {
        *self
    }
}
