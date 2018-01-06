use rand::random;

use arena::Boxed;
use tree::{Regulator, Node, rotate_left, rotate_right};

/// Like treap, but use tree-rotation to maintain balance.
#[derive(Debug, Clone, Copy)]
pub struct RevTreap(usize);

impl Default for RevTreap {
    fn default() -> Self {
        RevTreap(random())
    }
}

impl Regulator for RevTreap {
    fn update<K, V, B>(node: &mut B) where
        K: Ord,
        B: Boxed<Node<K, V, B, Self>>
    {
        let root = node.regulator.0;

        enum Dir {
            Left, Right, Nope
        }

        let mut dir = Dir::Nope;

        if let Some(ref left) = node.left.node {
            if left.regulator.0 > root {
                dir = Dir::Left;
            }
        }

        if let Some(ref right) = node.right.node {
            if right.regulator.0 > root {
                dir = Dir::Right;
            }
        }

        match dir {
            Dir::Left => rotate_right(node).unwrap(),
            Dir::Right => rotate_left(node).unwrap(),
            _ => {}
        }
    }
}
