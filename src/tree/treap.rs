use std::mem::replace;
use std::borrow::Borrow;

use rand::random;

use super::arena::{Arena, Bucket};

#[derive(Debug)]
pub struct TreeMap<K, V> {
    arena: Arena<Node<K, V>>,
    root: Link<K, V>,
}

impl<K: Ord, V> TreeMap<K, V> {
    pub fn new() -> Self {
        TreeMap {
            arena: Arena::new(),
            root: Link(None),
        }
    }

    pub fn clear(&mut self) {
        self.root = Link(None);
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

    pub fn entry(&mut self, key: K) -> Entry<K, V> {
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

impl<K: Ord, V> Default for TreeMap<K, V> {
    fn default() -> Self {
        TreeMap::new()
    }
}

#[derive(Debug)]
struct Link<K, V>(Option<Bucket<Node<K, V>>>);

#[derive(Debug)]
struct Node<K, V> {
    weight: usize,
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
}

impl<K: Ord, V> Link<K, V> {
    fn set(&mut self, arena: &Arena<Node<K, V>>, key: K, value: V) {
        self.0 = Some(arena.alloc(Node {
            weight: random(),
            key,
            value,
            left: Link(None),
            right: Link(None),
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
            let node = self.0.take().unwrap();
            let node = Bucket::into_inner(node);
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

    fn entry(&mut self, arena: &Arena<Node<K, V>>, key: K) -> Entry<K, V> {
        if self.same(&key) {
            return Entry::Occupied(Occupied {
                link: self
            });
        }

        match self.0 {
            None => Entry::Vacant(Vacant {
                arena: arena.clone(),
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
pub enum Entry<'a, K: 'a, V: 'a> {
    Vacant(Vacant<'a, K, V>),
    Occupied(Occupied<'a, K, V>),
}

impl<'a, K: Ord, V> Entry<'a, K, V> {
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

impl<'a, K: Ord, V: Default> Entry<'a, K, V> {
    pub fn or_default(self) -> &'a mut V {
        self.or_insert_with(Default::default)
    }
}

#[derive(Debug)]
pub struct Vacant<'a, K: 'a, V: 'a> {
    arena: Arena<Node<K, V>>,
    key: K,
    // This link MUST NOT has a node
    link: &'a mut Link<K, V>,
}

impl<'a, K: Ord, V> Vacant<'a, K, V> {
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

#[derive(Debug)]
pub struct Occupied<'a, K: 'a, V: 'a> {
    // This link MUST has a node
    link: &'a mut Link<K, V>,
}

impl<'a, K: Ord, V> Occupied<'a, K, V> {
    pub fn key(&self) -> &K {
        &self.link.0.as_ref().unwrap().key
    }

    pub fn remove_entry(self) -> (K, V) {
        let node = self.link.0.take().unwrap();
        let node = Bucket::into_inner(node);

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
