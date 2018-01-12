use std::rc::Rc;
use std::cell::{RefCell, RefMut, UnsafeCell};
use std::ops::{Deref, DerefMut};
use std::{usize, mem, ptr};

use arena;

pub struct VecArena<T>(Rc<RefCell<ArenaData<T>>>);

pub struct Boxed<T> {
    arena: VecArena<T>,
    index: usize,
}

pub struct UnsafeBoxed<T> {
    arena: VecArena<T>,
    index: usize,
}

struct ArenaData<T> {
    storage: Vec<UnsafeCell<Slot<T>>>,
    empty: usize,
}

trait SlotPtrExt<T> {
    fn to_ref<'a, U>(self, life: &'a U) -> &'a T;
    fn to_mut<'a, U>(self, life: &'a mut U) -> &'a mut T;
    fn set_data(self, data: T) -> usize;
    fn set_empty(self, empty: usize) -> T;
}

impl<T> VecArena<T> {
    pub fn new() -> Self {
        VecArena(Rc::new(RefCell::new(ArenaData {
            storage: vec![],
            empty: usize::MAX,
        })))
    }

    fn get(&self) -> RefMut<ArenaData<T>> {
        self.0.borrow_mut()
    }
}

impl<T> Default for VecArena<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Clone for VecArena<T> {
    fn clone(&self) -> Self {
        VecArena(self.0.clone())
    }
}

impl<T> arena::Arena<T, Boxed<T>> for VecArena<T> {
    fn alloc(&self, data: T) -> Boxed<T> {
        let index = self.get().alloc(data);

        Boxed {
            arena: self.clone(),
            index,
        }
    }
}

impl<T> Deref for Boxed<T> {
    type Target = T;

    fn deref(&self) -> &T {
        let slot = self.arena.get().slot(self.index);
        slot.to_ref(self)
    }
}

impl<T> DerefMut for Boxed<T> {
    fn deref_mut(&mut self) -> &mut T {
        let slot = self.arena.get().slot(self.index);
        slot.to_mut(self)
    }
}

impl<T> arena::Boxed<T> for Boxed<T> {
    type Unsafe = UnsafeBoxed<T>;

    fn unbox(boxed: Self) -> T {
        let data = {
            let mut arena = boxed.arena.get();
            arena.free(boxed.index)
        };
        mem::forget(boxed);

        data
    }

    fn to_unsafe(boxed: &mut Self) -> Self::Unsafe {
        UnsafeBoxed {
            arena: boxed.arena.clone(),
            index: boxed.index,
        }
    }
}

impl<T> arena::UnsafeBoxed<T> for UnsafeBoxed<T> {
    unsafe fn get(&self) -> &T {
        let slot = self.arena.get().slot(self.index);
        slot.to_ref(self)
    }

    unsafe fn get_mut(&mut self) -> &mut T {
        let slot = self.arena.get().slot(self.index);
        slot.to_mut(self)
    }
}

impl<T> ArenaData<T> {
    fn slot(&self, index: usize) -> *mut Slot<T> {
        self.storage[index].get()
    }

    fn alloc(&mut self, data: T) -> usize {
        if mem::size_of::<T>() == 0 {
            return 0;
        }

        if self.empty == usize::MAX {
            self.empty = self.storage.len();
            self.storage.push(Slot::default().into());
        }

        let index = self.empty;
        self.empty = self.slot(index).set_data(data);

        index
    }

    fn free(&mut self, index: usize) -> T {
        let prev_empty = self.empty;
        self.empty = index;

        self.slot(index).set_empty(prev_empty)
    }
}

#[cfg(not(feature = "unions"))]
enum Slot<T> {
    Data(T),
    Empty(usize),
}

#[cfg(not(feature = "unions"))]
impl<T> Default for Slot<T> {
    fn default() -> Self {
        Slot::Empty(usize::MAX)
    }
}

#[cfg(not(feature = "unions"))]
impl<T> SlotPtrExt<T> for *mut Slot<T> {
    fn to_ref<'a, U>(self, _life: &'a U) -> &'a T {
        unsafe {
            match *self {
                Slot::Data(ref data) => {
                    mem::transmute::<&T, &'a T>(data)
                }
                _ => panic!("This slot is not data"),
            }
        }
    }

    fn to_mut<'a, U>(self, _life: &'a mut U) -> &'a mut T {
        unsafe {
            match *self {
                Slot::Data(ref mut data) => {
                    mem::transmute::<&mut T, &'a mut T>(data)
                }
                _ => panic!("This slot is not data"),
            }
        }
    }

    fn set_data(self, data: T) -> usize {
        unsafe {
            match ptr::read(self) {
                Slot::Empty(empty) => {
                    ptr::write(self, Slot::Data(data));
                    empty
                }
                _ => panic!("This slot is not empty"),
            }
        }
    }

    fn set_empty(self, empty: usize) -> T {
        unsafe {
            match ptr::read(self) {
                Slot::Data(data) => {
                    ptr::write(self, Slot::Empty(empty));
                    data
                }
                _ => panic!("This slot is not data"),
            }
        }
    }
}

#[cfg(feature = "unions")]
union Slot<T> {
    data: mem::ManuallyDrop<T>,
    empty: usize,
}

#[cfg(feature = "unions")]
impl<T> Default for Slot<T> {
    fn default() -> Self {
        Slot { empty: usize::MAX }
    }
}

#[cfg(feature = "unions")]
impl<T> SlotPtrExt<T> for *mut Slot<T> {
    fn to_ref<'a, U>(self, _life: &'a U) -> &'a T {
        unsafe {
            mem::transmute::<&T, &'a T>(&*(*self).data)
        }
    }

    fn to_mut<'a, U>(self, _life:&'a mut U) -> &'a mut T {
        unsafe {
            mem::transmute::<&mut T, &'a mut T>(&mut *(*self).data)
        }
    }

    fn set_data(self, data: T) -> usize {
        unsafe {
            let empty = ptr::read(self).empty;
            let data = mem::ManuallyDrop::new(data);
            ptr::write(self, Slot { data });
            empty
        }
    }

    fn set_empty(self, empty: usize) -> T {
        unsafe {
            let data = ptr::read(self).data;
            let data = mem::ManuallyDrop::into_inner(data);
            ptr::write(self, Slot { empty });
            data
        }
    }
}
