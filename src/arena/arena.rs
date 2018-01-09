use std::ops::{Deref, DerefMut};
use std::marker::PhantomData;

/// Abstracted typed allocator
///
/// Similar to `std::heap::Alloc`, but more high-level and limited to single type
pub trait Arena<T, B>: Default where B: Boxed<T> {
    fn alloc(&self, value: T) -> B;
}

/// Abstracted allocated box
///
/// Similar to `Box`, this trait represents a wrapper type whose value is allocated "somewhere"
/// and release its memory when dropped.
pub trait Boxed<T>: Deref<Target=T> + DerefMut {
    type Unsafe: UnsafeBoxed<T>;

    fn unbox(boxed: Self) -> T;
    fn to_unsafe(boxed: &mut Self) -> Self::Unsafe;
}

/// Conceptually a raw pointer to allocated box.
pub trait UnsafeBoxed<T> {
    unsafe fn get(&self) -> &T;
    unsafe fn get_mut(&mut self) -> &mut T;
}

/// Simple typed allocator, just a wrapper around `Box`
/// This can be useful for comparison.
pub struct BoxArena<T>(PhantomData<*mut T>);

impl<T> Default for BoxArena<T> {
    fn default() -> Self {
        BoxArena(Default::default())
    }
}

impl<T> Arena<T, Box<T>> for BoxArena<T> {
    fn alloc(&self, value: T) -> Box<T> {
        Box::new(value)
    }
}

impl<T> Boxed<T> for Box<T> {
    type Unsafe = *mut T;

    fn unbox(boxed: Self) -> T {
        *boxed
    }

    fn to_unsafe(boxed: &mut Self) -> *mut T {
        &mut **boxed as *mut T
    }
}

impl<T> UnsafeBoxed<T> for *mut T {
    unsafe fn get(&self) -> &T {
        &**self
    }

    unsafe fn get_mut(&mut self) -> &mut T {
        &mut **self
    }
}
