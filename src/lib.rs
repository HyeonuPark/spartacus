#![feature(untagged_unions)]

extern crate rand;

mod alloc;
mod tree;

pub use self::alloc::{Alloc, Boxed, BoxAlloc};
pub use self::tree::*;
