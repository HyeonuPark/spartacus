use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

/// Abstracted typed allocator
///
/// Similar to `std::heap::Alloc`, but more high-level and limited to single type
pub trait Alloc<T: Sized>: Default {
    type Boxed: Boxed<T>;

    fn alloc(&self, value: T) -> Self::Boxed;
}

// Abstracted allocated box
//
// Similar to `Box`, this trait represents a wrapper type whose value is allocated "somewhere".
pub trait Boxed<T: Sized>: Drop + Deref<Target=T> + DerefMut {
    fn unbox(self) -> T;
}

pub struct BoxAlloc<T>(PhantomData<T>);

impl<T> Default for BoxAlloc<T> {
    fn default() -> Self {
        BoxAlloc(Default::default())
    }
}

impl<T> Alloc<T> for BoxAlloc<T> {
    type Boxed = Box<T>;

    fn alloc(&self, value: T) -> Box<T> {
        Box::new(value)
    }
}

impl<T> Boxed<T> for Box<T> {
    fn unbox(self) -> T {
        *self
    }
}
