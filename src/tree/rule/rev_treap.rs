use rand::random;

use super::prelude::*;

/// Like treap, but use tree-rotation to maintain balance.
#[derive(Debug, Clone, Copy)]
pub struct RevTreap(usize);

impl Default for RevTreap {
    fn default() -> Self {
        RevTreap(random())
    }
}

impl Rule for RevTreap {
    fn update<K, V, I>(node: &mut I) where
        K: Ord,
        I: Indirect<K, V, Self>,
    {
        let root = node.regulator.0;

        enum Dir {
            Left, Right, Nope
        }

        let mut dir = Dir::Nope;

        if let Some(ref left) = node.left {
            if left.regulator.0 > root {
                dir = Dir::Left;
            }
        }

        if let Some(ref right) = node.right {
            if right.regulator.0 > root {
                dir = Dir::Right;
            }
        }

        match dir {
            Dir::Left => node.rotate_right().unwrap(),
            Dir::Right => node.rotate_left().unwrap(),
            _ => {}
        }
    }
}
