use std::rc::Rc;
use std::cell::RefCell;
use std::mem::ManuallyDrop as MD;
use std::ptr;
use std::ops::{Deref, DerefMut};
use std::fmt;

use alloc::{Alloc, Boxed};

pub struct Arena<T>(Rc<RefCell<ArenaData<T>>>);

pub struct ArenaData<T> {
    storage: Vec<Slot<T>>,
    empty: usize,
}

#[derive(Clone)]
pub struct Bucket<T> {
    arena: Arena<T>,
    index: usize,
}

union Slot<T> {
    data: MD<T>,
    empty: usize,
}

impl<T> Arena<T> {
    pub fn new() -> Self {
        Arena(Rc::new(RefCell::new(ArenaData {
            storage: vec![],
            empty: 0,
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

impl<T> Alloc for Arena<T> {
    type Boxed = Bucket<T>;

    fn alloc(&self, data: T) -> Bucket<T> {
        let mut arena = self.0.borrow_mut();

        if arena.empty == 0 {
            arena.empty = arena.storage.len();
            arena.storage.push(Slot { empty: 0 });
        }

        let index = arena.empty;
        let new_empty = unsafe {
            let loc = arena.storage.get_mut(index).unwrap();
            let new_empty = loc.empty;
            loc.data = MD::new(data);
            new_empty
        };
        arena.empty = new_empty;

        drop(arena);

        Bucket {
            arena: Arena(self.0.clone()),
            index,
        }
    }
}

impl<T> Default for Arena<T> {
    fn default() -> Self {
        Arena::new()
    }
}

impl<T> Clone for Arena<T> {
    fn clone(&self) -> Self {
        Arena(Rc::clone(&self.0))
    }
}

impl<T> fmt::Debug for Arena<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Arena {{ .. }}")
    }
}

impl<T> Boxed for Bucket<T> {
    fn unbox(self) -> T {
        self.arena.get_move(self.index)
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

impl<T> Drop for Bucket<T> {
    fn drop(&mut self) {
        self.arena.free(self.index)
    }
}

impl<T: fmt::Debug> fmt::Debug for Bucket<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.arena.get(self.index).fmt(f)
    }
}
