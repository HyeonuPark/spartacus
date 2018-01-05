use std::ops::Deref;
use std::mem::replace;

use arena::Boxed;
use tree::Node;

#[derive(Debug)]
pub struct RotateEmptyLeg;

pub fn rotate_left<K, V, B, R>(root: &mut B) -> Result<(), RotateEmptyLeg> where
    K: Ord,
    B: Boxed + Deref<Target=Node<K, V, B, R>>,
    R: Regulator,
{
    //
    //     R            B
    //    / \          / \
    //   A   B   =>   R   D
    //      / \      / \
    //     C   D    A   C
    //

    let node_b = match root.right.node.take() {
        None => Err(RotateEmptyLeg)?,
        Some(node) => node,
    };

    let mut node_r = replace(root, node_b);

    let edge_c = root.left.node.take();

    node_r.right.node = edge_c;

    root.left.node = Some(node_r);

    Ok(())
}

pub fn rotate_right<K, V, B, R>(root: &mut B) -> Result<(), RotateEmptyLeg> where
    K: Ord,
    B: Boxed + Deref<Target=Node<K, V, B, R>>,
    R: Regulator,
{
    //
    //     R            A
    //    / \          / \
    //   A   B   =>   C   R
    //  / \              / \
    // C   D            D   B
    //

    let node_a = match root.left.node.take() {
        None => Err(RotateEmptyLeg)?,
        Some(node) => node,
    };

    let mut node_r = replace(root, node_a);

    let edge_d = root.right.node.take();

    node_r.left.node = edge_d;

    root.right.node = Some(node_r);

    Ok(())
}

pub trait Regulator: Default {
    fn update<K, V, B>(node: &mut B) where
        K: Ord,
        B: Boxed + Deref<Target=Node<K, V, B, Self>>;
}

#[derive(Default)]
pub struct NoopRegulator;

impl Regulator for NoopRegulator {
    fn update<K, V, B>(_node: &mut B) where
        K: Ord,
        B: Boxed + Deref<Target=Node<K, V, B, Self>>
    {}
}
