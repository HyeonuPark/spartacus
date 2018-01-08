use std::borrow::Borrow;
use std::cmp::Ordering::{self, Less, Equal, Greater};
use std::mem::swap;
use std::marker::PhantomData;

use arena::{Arena, Boxed};
use tree::Regulator;

#[macro_export]
macro_rules! treemap {
    ($name:ident, $K:ty, $V:ty, $R:ty, $A:ident, $B:ident, $I:ident) => (
        type $name = TreeMap<$K, $V, $R, $A<$crate::tree::Node<$K, $V, $R, $I>>, $I>;

        struct $I($B<$crate::tree::Node<$K, $V, $R, $I>>);

        impl ::std::ops::Deref for $I {
            type Target = $crate::tree::Node<$K, $V, $R, $I>;

            fn deref(&self) -> &$crate::tree::Node<$K, $V, $R, $I> {
                &*self.0
            }
        }

        impl ::std::ops::DerefMut for $I {
            fn deref_mut(&mut self) -> &mut $crate::tree::Node<$K, $V, $R, $I> {
                &mut *self.0
            }
        }

        impl $crate::arena::Boxed<$crate::tree::Node<$K, $V, $R, $I>> for $I {
            fn unbox(boxed: Self) -> $crate::tree::Node<$K, $V, $R, $I> {
                type B = $B<$crate::tree::Node<$K, $V, $R, $I>>;

                B::unbox(boxed.0)
            }
        }

        impl $crate::tree::Indirect<$K, $V, $R> for $I {
            type Inner = $B<$crate::tree::Node<$K, $V, $R, $I>>;

            fn new(b: Self::Inner) -> Self {
                $I(b)
            }
        }
    );
}

pub struct TreeMap<K, V, R, A, I> where
    K: Ord,
    R: Regulator,
    A: Arena<Node<K, V, R, I>, I::Inner>,
    I: Indirect<K, V, R>,
{
    arena: A,
    root: Edge<K, V, R, I>,
}

impl<K, V, R, A, I> TreeMap<K, V, R, A, I> where
    K: Ord,
    R: Regulator,
    A: Arena<Node<K, V, R, I>, I::Inner>,
    I: Indirect<K, V, R>,
{
    pub fn new() -> Self {
        TreeMap {
            arena: A::default(),
            root: Edge::new(),
        }
    }

    pub fn clear(&mut self) {
        self.root = Edge::new();
    }

    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.node.is_some()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key).map(|node| &node.value)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key).is_some()
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get_mut(key).map(|node| &mut node.value)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.remove(key).map(|node| Boxed::unbox(node).value)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let node = self.arena.alloc(Node::new(key, value));
        self.root.insert(I::new(node)).map(|node| Boxed::unbox(node).value)
    }
}

impl<K, V, R, A, I> Default for TreeMap<K, V, R, A, I> where
    K: Ord,
    R: Regulator,
    A: Arena<Node<K, V, R, I>, I::Inner>,
    I: Indirect<K, V, R>,
{
    fn default() -> Self {
        TreeMap::new()
    }
}

pub trait Indirect<K, V, R>: Boxed<Node<K, V, R, Self>> where
    Self: Sized,
    K: Ord,
    R: Regulator,
{
    type Inner: Boxed<Node<K, V, R, Self>>;

    fn new(inner: Self::Inner) -> Self;
}

pub struct Node<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    key: K,
    value: V,
    pub left: Edge<K, V, R, I>,
    pub right: Edge<K, V, R, I>,
    pub regulator: R,
}

impl<K, V, R, I> Node<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            left: Edge::new(),
            right: Edge::new(),
            regulator: R::default(),
        }
    }
}

pub struct Edge<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    pub node: Option<I>,
    _marker: PhantomData<(K, V, R)>,
}

impl<K, V, R, I> Edge<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    fn new() -> Self {
        Edge {
            node: None,
            _marker: Default::default(),
        }
    }

    fn len(&self) -> usize {
        match self.node {
            None => 0,
            Some(ref node) => node.left.len() + node.right.len() + 1,
        }
    }

    fn cmp_key<Q>(&self, key: &Q) -> Option<Ordering> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        match self.node {
            None => None,
            Some(ref node) => {
                if node.key.borrow() == key {
                    Some(Equal)
                } else if node.key.borrow() <= key {
                    Some(Less)
                } else {
                    Some(Greater)
                }
            }
        }
    }

    fn update(&mut self) {
        if let Some(ref mut node) = self.node {
            R::update(node);
        }
    }

    fn get<Q>(&self, key: &Q) -> Option<&I> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        match self.cmp_key(key) {
            None => None,
            Some(ord) => self.node.as_ref().and_then(|node| match ord {
                Equal => Some(node),
                Less => node.left.get(key),
                Greater => node.right.get(key),
            }),
        }
    }

    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut I> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        match self.cmp_key(key) {
            None => None,
            Some(ord) => self.node.as_mut().and_then(|node| match ord {
                Equal => Some(node),
                Less => node.left.get_mut(key),
                Greater => node.right.get_mut(key),
            }),
        }
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<I> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        let child = match self.cmp_key(key) {
            None => return None,
            Some(Equal) => {
                return self.node.take()
            }
            Some(ord) => self.node.as_mut().and_then(|node| match ord {
                Equal => unreachable!(),
                Less => node.left.remove(key),
                Greater => node.right.remove(key),
            }),
        };

        if child.is_some() {
            self.update();
        }

        child
    }

    fn insert(&mut self, mut newbie: I) -> Option<I> {
        let res = match self.node {
            None => {
                self.node = Some(newbie);
                return None
            }
            Some(ref mut node) => {
                if node.key == newbie.key {
                    swap(&mut node.key, &mut newbie.key);
                    swap(&mut node.value, &mut newbie.value);
                    Some(newbie)
                } else if node.key <= newbie.key {
                    node.left.insert(newbie)
                } else {
                    node.right.insert(newbie)
                }
            }
        };

        // when a new node is added to the subtree
        if res.is_none() {
            self.update();
        }

        res
    }
}
