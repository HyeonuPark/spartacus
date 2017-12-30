use std::ops::Deref;
use std::borrow::Borrow;
use std::mem::replace;

use rand::random;

use arena::{Arena, Boxed};

#[derive(Debug)]
pub struct TreeMap<A, B, K, V> where
    A: Arena<Boxed=B>,
    B: Boxed + Deref<Target=Node<B, K, V>>,
{
    arena: A,
    root: Link<B>,
}

impl<A, B, K, V> TreeMap<A, B, K, V> where
    A: Arena<Boxed=B>,
    B: Boxed + Deref<Target=Node<B, K, V>>,
    K: Ord,
{
    pub fn new() -> Self {
        TreeMap {
            arena: Default::default(),
            root: Link::new(),
        }
    }

    pub fn clear(&mut self) {
        self.root = Link::new();
    }

    pub fn len(&self) -> usize {
        self.root.len()
    }

    pub fn is_empty(&self) -> bool {
        self.root.0.is_none()
    }

    pub fn get<Q>(&self, key: &Q) -> Option<&V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key).is_some()
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get_mut(key)
    }

    pub fn entry(&mut self, key: K) -> Entry<A, B, K, V> {
        self.root.entry(&self.arena, key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.root.entry(&self.arena, key) {
            Entry::Occupied(mut o) => Some(o.insert(value)),
            Entry::Vacant(v) => {
                v.insert(value);
                None
            }
        }
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.remove(key)
    }
}

#[derive(Debug)]
pub struct Link<B: Boxed>(Option<B>);

#[derive(Debug)]
pub struct Node<B: Boxed + Deref<Target=Node<B, K, V>>, K, V> {
    weight: usize,
    key: K,
    value: V,
    left: Link<B>,
    right: Link<B>,
}

impl<B: Boxed + Deref<Target=Node<B, K, V>>, K: Ord, V> Link<B> {
    fn new() -> Self {
        Link(None)
    }

    fn set<A: Arena<Boxed=B>>(&mut self, arena: &A, key: K, value: V) {
        self.0 = Some(arena.alloc(Node {
            weight: random(),
            key,
            value,
            left: Link::new(),
            right: Link::new(),
        }))
    }

    fn len(&self) -> usize {
        match self.0 {
            None => 0,
            Some(ref node) => node.left.len() + node.right.len(),
        }
    }

    fn same<Q>(&self, key: &Q) -> bool where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        match self.0 {
            None => false,
            Some(ref node) => node.key.borrow() == key,
        }
    }

    fn get<'s, Q>(&'s self, key: &Q) -> Option<&'s V> where
        K: Borrow<Q> + 's, Q: Ord + ?Sized
    {
        match self.0 {
            None => None,
            Some(ref node) => {
                if node.key.borrow() == key {
                    Some(&node.value)
                } else if node.key.borrow() <= key {
                    node.left.get(key)
                } else {
                    node.right.get(key)
                }
            }
        }
    }

    fn get_mut<'s, Q>(&'s mut self, key: &Q) -> Option<&'s mut V> where
        K: Borrow<Q> + 's, Q: Ord + ?Sized
    {
        match self.0 {
            None => None,
            Some(ref mut node) => {
                if node.key.borrow() == key {
                    Some(&mut node.value)
                } else if node.key.borrow() <= key {
                    node.left.get_mut(key)
                } else {
                    node.right.get_mut(key)
                }
            }
        }
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        if self.same(key) {
            let node = self.0.take().unwrap().unbox();
            return Some(node.value);
        }

        match self.0 {
            None => None,
            Some(ref mut node) => {
                if node.key.borrow() <= key {
                    node.left.remove(key)
                } else {
                    node.right.remove(key)
                }
            }
        }
    }

    fn entry<'a, 'l, A>(&'l mut self, arena: &'a A, key: K) -> Entry<'a, 'l, A, B, K, V> where
        'a: 'l, A: Arena<Boxed=B> + 'a, B: 'l, K: 'l, V: 'l
    {
        if self.same(&key) {
            return Entry::Occupied(Occupied { link: self });
        }

        match self.0 {
            None => Entry::Vacant(Vacant {
                arena,
                key,
                link: self,
            }),
            Some(ref mut node) => {
                if node.key <= key {
                    node.left.entry(arena, key)
                } else {
                    node.right.entry(arena, key)
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Entry<'a, 'l, A, B, K, V> where
    'a: 'l,
    A: Arena<Boxed=B> + 'a,
    B: Boxed + Deref<Target=Node<B, K, V>> + 'l,
    K: 'l,
{
    Vacant(Vacant<'a, 'l, A, B, K, V>),
    Occupied(Occupied<'l, B, K, V>),
}

impl<'a, 'l, A, B, K, V> Entry<'a, 'l, A, B, K, V> where
    'a: 'l,
    A: Arena<Boxed=B> + 'a,
    B: Boxed + Deref<Target=Node<B, K, V>> + 'l,
    K: Ord + 'l,
{
    pub fn or_insert(self, default: V) -> &'l mut V {
        match self {
            Entry::Vacant(v) => v.insert(default),
            Entry::Occupied(o) => o.into_mut(),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'l mut V {
        match self {
            Entry::Vacant(v) => v.insert(default()),
            Entry::Occupied(o) => o.into_mut(),
        }
    }

    pub fn key(&self) -> &K {
        match *self {
            Entry::Vacant(ref v) => v.key(),
            Entry::Occupied(ref o) => o.key(),
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut o) = self {
            f(o.get_mut());
        }

        self
    }
}

#[derive(Debug)]
pub struct Vacant<'a, 'l, A, B, K, V> where
    'a: 'l,
    A: Arena<Boxed=B> + 'a,
    B: Boxed + Deref<Target=Node<B, K, V>> + 'l,
{
    arena: &'a A,
    key: K,
    link: &'l mut Link<B>,
}

impl<'a, 'l, A, B, K, V> Vacant<'a, 'l, A, B, K, V> where
    'a: 'l,
    A: Arena<Boxed=B> + 'a,
    B: Boxed + Deref<Target=Node<B, K, V>> + 'l,
    K: Ord + 'l,
{
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn insert(self, value: V) -> &'l mut V {
        let Vacant { arena, key, link } = self;
        link.set(arena, key, value);
        &mut link.0.as_mut().unwrap().value
    }
}

#[derive(Debug)]
pub struct Occupied<'l, B, K, V> where
    B: Boxed + Deref<Target=Node<B, K, V>> + 'l,
{
    link: &'l mut Link<B>,
}

impl<'l, B, K, V> Occupied<'l, B, K, V> where
    B: Boxed + Deref<Target=Node<B, K, V>> + 'l,
    K: 'l,
{
    pub fn key(&self) -> &K {
        &self.link.0.as_ref().unwrap().key
    }

    pub fn remove_entry(self) -> (K, V) {
        let node = self.link.0.take().unwrap().unbox();

        (node.key, node.value)
    }

    pub fn get(&self) -> &V {
        &self.link.0.as_ref().unwrap().value
    }

    pub fn get_mut(&mut self) -> &mut V {
        &mut self.link.0.as_mut().unwrap().value
    }

    pub fn into_mut(self) -> &'l mut V {
        &mut self.link.0.as_mut().unwrap().value
    }

    pub fn insert(&mut self, value: V) -> V {
        replace(self.get_mut(), value)
    }

    pub fn remove(self) -> V {
        self.remove_entry().1
    }
}
