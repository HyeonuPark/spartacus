use std::borrow::Borrow;
use std::marker::PhantomData;

use arena::Arena;
use tree::{Node, Indirect};
use tree::rule::Rule;

use super::node::Edge;

#[macro_export]
macro_rules! treemap {
    ($name:ident, $K:ty, $V:ty, $R:ty, $A:ident, $B:ident, $I:ident) => (
        treemap!{!_impl
            $name, $K, $V, $R, $A, $B, $I,
            $crate::tree::Node<$K, $V, $R, $I>
        }
    );
    (!_impl $name:ident, $K:ty, $V:ty, $R:ty, $A:ident, $B:ident, $I:ident, $Node:ty) => (
        type $name = TreeMap<$K, $V, $R, $A<$Node>, $I>;

        struct $I($B<$Node>);

        impl ::std::ops::Deref for $I {
            type Target = $Node;

            fn deref(&self) -> &$Node {
                &*self.0
            }
        }

        impl ::std::ops::DerefMut for $I {
            fn deref_mut(&mut self) -> &mut $Node {
                &mut *self.0
            }
        }

        impl $crate::arena::Boxed<$Node> for $I {
            type Unsafe = <$B<$Node> as $crate::arena::Boxed<$Node>>::Unsafe;

            fn unbox(boxed: Self) -> $Node {
                $B::<$Node>::unbox(boxed.0)
            }

            fn to_unsafe(boxed: &mut Self) -> Self::Unsafe {
                $B::<$Node>::to_unsafe(&mut boxed.0)
            }
        }

        impl $crate::tree::Indirect<$K, $V, $R> for $I {
            type Inner = $B<$Node>;

            fn new(b: $B<$Node>) -> Self {
                $I(b)
            }
        }
    );
}

pub struct TreeMap<K, V, R, A, I> where
    K: Ord,
    R: Rule,
    A: Arena<Node<K, V, R, I>, I::Inner>,
    I: Indirect<K, V, R>,
{
    arena: A,
    root: Option<I>,
    _marker: PhantomData<Node<K, V, R, I>>,
}

impl<K, V, R, A, I> TreeMap<K, V, R, A, I> where
    K: Ord,
    R: Rule,
    A: Arena<Node<K, V, R, I>, I::Inner>,
    I: Indirect<K, V, R>,
{
    pub fn new() -> Self {
        TreeMap {
            arena: A::default(),
            root: None,
            _marker: Default::default(),
        }
    }

    pub fn clear(&mut self) {
        self.root = None;
    }

    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_some()
    }

    pub fn get<'a, Q>(&self, key: &Q) -> Option<&V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key)
    }

    pub fn contains_key<'a, Q>(&self, key: &Q) -> bool where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key).is_some()
    }

    pub fn get_mut<'a, Q>(&mut self, key: &Q) -> Option<&mut V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get_mut(key)
    }

    pub fn remove<'a, Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.remove(key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let node = self.arena.alloc(Node::new(key, value));
        self.root.insert(I::new(node))
    }
}

impl<K, V, R, A, I> Default for TreeMap<K, V, R, A, I> where
    K: Ord,
    R: Rule,
    A: Arena<Node<K, V, R, I>, I::Inner>,
    I: Indirect<K, V, R>,
{
    fn default() -> Self {
        TreeMap::new()
    }
}
