use std::borrow::Borrow;
use std::cmp::Ordering::{self, Less, Equal, Greater};
use std::mem::swap;

use arena::Boxed;
use tree::Regulator;

pub struct Node<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    key: K,
    value: V,
    pub up: Option<<I::Inner as Boxed<Node<K, V, R, I>>>::Unsafe>,
    pub left: Option<I>,
    pub right: Option<I>,
    pub regulator: R,
}

impl<K, V, R, I> Node<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    pub fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            up: None,
            left: None,
            right: None,
            regulator: R::default(),
        }
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

pub trait Edge<K, V, R, I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    fn len(&self) -> usize;
    fn cmp_key<Q>(&self, key: &Q) -> Option<Ordering> where
        K: Borrow<Q>, Q: Ord + ?Sized;
    fn update(&mut self);
    fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, V: 'a, R: 'a;
    fn get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, V: 'a, R: 'a;
    fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized;
    fn insert(&mut self, node: I) -> Option<V>;
}

impl<K, V, R, I> Edge<K, V, R, I> for Option<I> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    fn len(&self) -> usize {
        match *self {
            None => 0,
            Some(ref node) => node.left.len() + node.right.len() + 1,
        }
    }

    fn cmp_key<Q>(&self, key: &Q) -> Option<Ordering> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        match *self {
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
        if let Some(ref mut node) = *self {
            R::update(node);
        }
    }

    fn get<'a, Q>(&'a self, key: &Q) -> Option<&'a V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, V: 'a, R: 'a
    {
        match self.cmp_key(key) {
            None => None,
            Some(ord) => self.as_ref().and_then(|node| match ord {
                Equal => Some(&node.value),
                Less => node.left.get(key),
                Greater => node.right.get(key),
            }),
        }
    }

    fn get_mut<'a, Q>(&'a mut self, key: &Q) -> Option<&'a mut V> where
        K: Borrow<Q> + 'a, Q: Ord + ?Sized, V: 'a, R: 'a
    {
        match self.cmp_key(key) {
            None => None,
            Some(ord) => self.as_mut().and_then(|node| match ord {
                Equal => Some(&mut node.value),
                Less => node.left.get_mut(key),
                Greater => node.right.get_mut(key),
            }),
        }
    }

    fn remove<Q>(&mut self, key: &Q) -> Option<V> where
        K: Borrow<Q>, Q: Ord + ?Sized
    {
        let child = match self.cmp_key(key) {
            None => return None,
            Some(Equal) => {
                return self.take().map(|node| Boxed::unbox(node).value)
            }
            Some(ord) => self.as_mut().and_then(|node| match ord {
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

    fn insert(&mut self, mut newbie: I) -> Option<V> {
        let res = match *self {
            None => {
                *self = Some(newbie);
                return None
            }
            Some(ref mut node) => {
                if node.key == newbie.key {
                    swap(&mut node.key, &mut newbie.key);
                    swap(&mut node.value, &mut newbie.value);
                    Some(Boxed::unbox(newbie).value)
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
