use std::cmp::Ordering::{self, Less, Equal, Greater};
use std::ops::Deref;
use std::borrow::Borrow;

use arena::{Arena, Boxed};

pub struct TreeMap<K, V, A, T> where
    K: Ord,
    A: Arena<Boxed=T::Boxed>,
    T: Tree<Pair<K, V>>,
{
    arena: A,
    root: Option<A::Boxed>,
    _marker: ::std::marker::PhantomData<(T, K, V)>,
}

impl<K, V, A, T> TreeMap<K, V, A, T> where
    K: Ord,
    A: Arena<Boxed=T::Boxed>,
    T: Tree<Pair<K, V>>,
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
}

#[derive(Debug)]
pub struct Pair<K, V> {
    key: K,
    value: V,
}

impl<K: PartialEq, V> PartialEq for Pair<K, V> {
    fn eq(&self, rhs: &Self) -> bool {
        self.key.eq(&rhs.key)
    }
}

impl<K: PartialEq, V> PartialEq<K> for Pair<K, V> {
    fn eq(&self, rhs: &K) -> bool {
        self.key.eq(&rhs)
    }
}

impl<K: Eq, V> Eq for Pair<K, V> {}

impl<K: PartialOrd, V> PartialOrd for Pair<K, V> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&rhs.key)
    }
}

impl<K: PartialOrd, V> PartialOrd<K> for Pair<K, V> {
    fn partial_cmp(&self, rhs: &K) -> Option<Ordering> {
        self.key.partial_cmp(&rhs)
    }
}

impl<K: Ord, V> Ord for Pair<K, V> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.key.cmp(&rhs.key)
    }
}

pub trait Tree<D: Ord>: AsRef<D> + AsMut<D> {
    type Boxed: Boxed + Deref<Target=Self>;

    fn new(data: D) -> Self;

    fn into_data(self) -> D;

    fn children(&self) -> (&Option<Self::Boxed>, &Option<Self::Boxed>);

    fn children_mut(&mut self) -> (&mut Option<Self::Boxed>, &mut Option<Self::Boxed>);

    fn update(&mut self);
}

trait Link<B: Boxed + Deref<Target=T>, T: Tree<Pair<K, V>>, K: Ord, V> {
    fn len(&self) -> usize;

    fn cmp_key<Q>(&self, key: &Q) -> Option<Ordering> where
        K: Borrow<Q>, Q: Ord + ?Sized;

    fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, T: 'a;

    fn get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, T: 'a;

    fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized;
}

impl<B, T, K, V> Link<B, T, K, V> for Option<B> where
    B: Boxed + Deref<Target=T>, T: Tree<Pair<K, V>, Boxed=B>, K: Ord
{
    fn len(&self) -> usize {
        match *self {
            None => 0,
            Some(ref node) => {
                let (l, r): (&Option<B>, &Option<B>) = node.children();
                l.len() + r.len() + 1
            }
        }
    }

    fn cmp_key<Q>(&self, key: &Q) -> Option<Ordering> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        match *self {
            None => None,
            Some(ref node) => {
                if node.as_ref().key.borrow() == key {
                    Some(Equal)
                } else if node.as_ref().key.borrow() <= key {
                    Some(Less)
                } else {
                    Some(Greater)
                }
            }
        }
    }

    fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, T: 'a
    {
        match self.cmp_key(key) {
            None => None,
            Some(ord) => self.as_ref().and_then(|node| match ord {
                Equal => Some(&node.as_ref().value),
                Less => node.children().0.get(key),
                Greater => node.children().1.get(key),
            })
        }
    }

    fn get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, T: 'a
    {
        match self.cmp_key(key) {
            None => None,
            Some(ord) => self.as_mut().and_then(|node| match ord {
                Equal => Some(&mut node.as_mut().value),
                Less => node.children_mut().0.get_mut(key),
                Greater => node.children_mut().1.get_mut(key),
            })
        }
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        let child = match self.cmp_key(key) {
            None => return None,
            Some(Equal) => return self.take().map(|node| {
                node.unbox().into_data().value
            }),
            Some(ord) => self.as_mut().unwrap().children_mut().0.remove(key),
            Some(Greater) => self.as_mut().unwrap().children_mut().1.remove(key),
        };

        if child.is_some() {
            self.as_mut().unwrap().update();
        }

        child
    }
}
