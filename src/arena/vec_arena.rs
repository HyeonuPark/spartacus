use std::rc::Rc;
use std::cell::RefCell;
use std::mem::ManuallyDrop as MD;
use std::ptr;
use std::ops::{Deref, DerefMut};
use std::fmt;
use std::usize;

use super::{Arena, Boxed, UnsafeBoxed};

pub struct VecArena<T>(Rc<RefCell<ArenaData<T>>>);

pub struct ArenaData<T> {
    storage: Vec<Slot<T>>,
    empty: usize,
}

#[derive(Clone)]
pub struct Bucket<T> {
    arena: VecArena<T>,
    index: usize,
}

union Slot<T> {
    data: MD<T>,
    empty: usize,
}

pub struct UnsafeBucket<T> {
    arena: VecArena<T>,
    index: usize,
}

impl<T> VecArena<T> {
    pub fn new() -> Self {
        VecArena(Rc::new(RefCell::new(ArenaData {
            storage: vec![],
            empty: usize::MAX,
        })))
    }

    fn free(&self, index: usize) {
        let mut arena = self.0.borrow_mut();

        arena.storage[index].empty = arena.empty;
        arena.empty = index;
    }

    unsafe fn get_ptr(&self, index: usize) -> *mut T {
        assert!(index < self.0.borrow().storage.len());

        self.0.borrow_mut()
            .storage.as_mut_ptr()
            .offset(index as isize) as *mut T
    }

    fn get(&self, index: usize) -> &T {
        unsafe {
            &*self.get_ptr(index)
        }
    }

    fn get_mut(&self, index: usize) -> &mut T {
        unsafe {
            &mut *self.get_ptr(index)
        }
    }

    fn get_move(&self, index: usize) -> T {
        let res = unsafe {
            ptr::read(self.get_ptr(index))
        };

        self.free(index);

        res
    }
}

impl<T> Arena<T, Bucket<T>> for VecArena<T> {
    fn alloc(&self, data: T) -> Bucket<T> {
        let mut arena = self.0.borrow_mut();

        if arena.empty == usize::MAX {
            arena.storage.push(Slot { empty: 0 });
            arena.empty = 0;
        }

        let index = arena.empty;
        arena.empty = unsafe {
            let loc = arena.storage.get_mut(index)
                .expect("Failed to get arena storage");
            let new_empty = loc.empty;
            loc.data = MD::new(data);
            new_empty
        };

        drop(arena);

        Bucket {
            arena: VecArena(self.0.clone()),
            index,
        }
    }
}

impl<T> Default for VecArena<T> {
    fn default() -> Self {
        VecArena::new()
    }
}

impl<T> Clone for VecArena<T> {
    fn clone(&self) -> Self {
        VecArena(Rc::clone(&self.0))
    }
}

impl<T> fmt::Debug for VecArena<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Arena {{ .. }}")
    }
}

impl<T> Drop for Bucket<T> {
    fn drop(&mut self) {
        self.arena.free(self.index)
    }
}

impl<T> Deref for Bucket<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.arena.get(self.index)
    }
}

impl<T> DerefMut for Bucket<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.arena.get_mut(self.index)
    }
}

impl<T> Boxed<T> for Bucket<T> {
    type Unsafe = UnsafeBucket<T>;

    fn unbox(boxed: Self) -> T {
        boxed.arena.get_move(boxed.index)
    }

    fn to_unsafe(boxed: &Self) -> Self::Unsafe {
        UnsafeBucket {
            arena: boxed.arena.clone(),
            index: boxed.index,
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Bucket<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.arena.get(self.index).fmt(f)
    }
}

impl<T> UnsafeBoxed<T> for UnsafeBucket<T> {
    unsafe fn get(&self) -> &T {
        &*self.arena.get_ptr(self.index)
    }
}
