use std::mem::replace;

use tree::Indirect;

#[derive(Debug)]
pub struct RotateEmptyLeg;

pub fn rotate_left<K, V, R, I>(root: &mut I) -> Result<(), RotateEmptyLeg> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    //
    //     R            B
    //    / \          / \
    //   A   B   =>   R   D
    //      / \      / \
    //     C   D    A   C
    //

    let node_b = match root.right.take() {
        None => Err(RotateEmptyLeg)?,
        Some(node) => node,
    };

    let mut node_r = replace(root, node_b);

    let edge_c = root.left.take();

    node_r.right = edge_c;

    root.left = Some(node_r);

    Ok(())
}

pub fn rotate_right<K, V, R, I>(root: &mut I) -> Result<(), RotateEmptyLeg> where
    K: Ord,
    R: Regulator,
    I: Indirect<K, V, R>,
{
    //
    //     R            A
    //    / \          / \
    //   A   B   =>   C   R
    //  / \              / \
    // C   D            D   B
    //

    let node_a = match root.left.take() {
        None => Err(RotateEmptyLeg)?,
        Some(node) => node,
    };

    let mut node_r = replace(root, node_a);

    let edge_d = root.right.take();

    node_r.left = edge_d;

    root.right = Some(node_r);

    Ok(())
}

pub trait Regulator: Default {
    fn update<K, V, I>(node: &mut I) where
        K: Ord,
        I: Indirect<K, V, Self>;
}

#[derive(Default)]
pub struct Noop;

impl Regulator for Noop {
    fn update<K, V, I>(_node: &mut I) where
        K: Ord,
        I: Indirect<K, V, Self>
    {}
}
