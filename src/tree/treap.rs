use std::mem::replace;
use std::borrow::Borrow;
use std::marker::PhantomData;

use rand::random;

use alloc::{Alloc, Boxed};

pub struct TreeMap<A: Alloc<Node<A, K, V>>, K, V> {
    arena: A,
    root: Link<A, K, V>,
    _phantom: PhantomData<(K, V)>,
}

impl<A: Alloc<Node<A, K, V>>, K: Ord, V> TreeMap<A, K, V> {
    pub fn new() -> Self {
        TreeMap {
            arena: Default::default(),
            root: Link::new(),
            _phantom: Default::default(),
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

    pub fn get<Q>(&self, key: &Q) -> Option<&V>
        where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key)
    }

    pub fn contains_key<Q>(&self, key: &Q) -> bool
        where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get(key).is_some()
    }

    pub fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
        where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.get_mut(key)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
        where K: Borrow<Q>, Q: Ord + ?Sized
    {
        self.root.remove(key)
    }

    pub fn entry(&mut self, key: K) -> Entry<A, K, V> {
        self.root.entry(&self.arena, key)
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        match self.root.entry(&self.arena, key) {
            Entry::Occupied(mut occupied) => Some(occupied.insert(value)),
            Entry::Vacant(vacant) => {
                vacant.insert(value);
                None
            }
        }
    }
}

impl<A: Alloc<Node<A, K, V>>, K: Ord, V> Default for TreeMap<A, K, V> {
    fn default() -> Self {
        TreeMap::new()
    }
}

struct Link<A: Alloc<Node<A, K, V>>, K, V>(Option<A::Boxed>, PhantomData<(K, V)>);

struct Node<A: Alloc<Node<A, K, V>>, K, V> {
    weight: usize,
    key: K,
    value: V,
    left: Link<A, K, V>,
    right: Link<A, K, V>,
}

impl<A: Alloc<Node<A, K, V>>, K: Ord, V> Link<A, K, V> {
    fn new() -> Self {
        Link(None, Default::default())
    }

    fn set(&mut self, arena: &A, key: K, value: V) {
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

    fn same<Q>(&self, key: &Q) -> bool
        where K: Borrow<Q>, Q: Ord + ?Sized
    {
        match self.0 {
            None => false,
            Some(ref node) => node.key.borrow() == key,
        }
    }

    fn get<Q>(&self, key: &Q) -> Option<&V>
        where K: Borrow<Q>, Q: Ord + ?Sized
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

    fn get_mut<Q>(&mut self, key: &Q) -> Option<&mut V>
        where K: Borrow<Q>, Q: Ord + ?Sized
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

    fn remove<Q>(&mut self, key: &Q) -> Option<V>
        where K: Borrow<Q>, Q: Ord + ?Sized
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

    fn entry<'a, 'b>(&'a mut self, arena: &'b A, key: K) -> Entry<'a, 'b, A, K, V> {
        if self.same(&key) {
            return Entry::Occupied(Occupied {
                link: self
            });
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

pub enum Entry<'a, 'b, A: Alloc<Node<A, K, V>> + 'a + 'b, K: 'a, V: 'a> {
    Vacant(Vacant<'a, 'b, A, K, V>),
    Occupied(Occupied<'a, A, K, V>),
}

impl<'a, 'b, A: Alloc<Node<A, K, V>>, K: Ord, V> Entry<'a, 'b, A, K, V> {
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Vacant(vacant) => vacant.insert(default),
            Entry::Occupied(occupied) => occupied.into_mut(),
        }
    }

    pub fn or_insert_with<F: FnOnce() -> V>(self, default: F) -> &'a mut V {
        match self {
            Entry::Vacant(vacant) => vacant.insert(default()),
            Entry::Occupied(occupied) => occupied.into_mut(),
        }
    }

    pub fn key(&self) -> &K {
        match *self {
            Entry::Vacant(ref vacant) => vacant.key(),
            Entry::Occupied(ref occupied) => occupied.key(),
        }
    }

    pub fn and_modify<F: FnOnce(&mut V)>(mut self, f: F) -> Self {
        if let Entry::Occupied(ref mut occupied) = self {
            f(occupied.get_mut());
        }

        self
    }
}

impl<'a, 'b, A: Alloc<Node<A, K, V>> + 'a + 'b, K: Ord, V: Default> Entry<'a, 'b, A, K, V> {
    pub fn or_default(self) -> &'a mut V {
        self.or_insert_with(Default::default)
    }
}

pub struct Vacant<'a, 'b, A: Alloc<Node<A, K, V>> + 'a + 'b, K: 'a, V: 'a> {
    arena: &'b A,
    key: K,
    // This link MUST NOT has a node
    link: &'a mut Link<A, K, V>,
}

impl<'a, 'b, A: Alloc<Node<A, K, V>>, K: Ord, V> Vacant<'a, 'b, A, K, V> {
    pub fn key(&self) -> &K {
        &self.key
    }

    pub fn into_key(self) -> K {
        self.key
    }

    pub fn insert(self, value: V) -> &'a mut V {
        self.link.set(&self.arena, self.key, value);
        &mut self.link.0.as_mut().unwrap().value
    }
}

pub struct Occupied<'a, A: Alloc<Node<A, K, V>> + 'a, K: 'a, V: 'a> {
    // This link MUST has a node
    link: &'a mut Link<A, K, V>,
}

impl<'a, A: Alloc<Node<A, K, V>>, K: Ord, V> Occupied<'a, A, K, V> {
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

    pub fn into_mut(self) -> &'a mut V {
        &mut self.link.0.as_mut().unwrap().value
    }

    pub fn insert(&mut self, value: V) -> V {
        replace(self.get_mut(), value)
    }

    pub fn remove(self) -> V {
        self.remove_entry().1
    }
}
